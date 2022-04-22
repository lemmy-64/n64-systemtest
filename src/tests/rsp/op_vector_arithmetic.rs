use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;

use crate::math::vector::Vector;
use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{CP2FlagsRegister, E, Element, GPR, RSPAssembler, VR, VSARAccumulator};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq_vector};

fn run_test<F: Fn(&mut RSPAssembler)>(
    vco: u16, vcc: u16, vce: u8,
    emitter: F,
    vector1: Vector, vector2: Vector,

    expected_vco: u16, expected_vcc: u16, expected_vce: u8,
    expected_result: Vector, expected_acc_low: Vector) -> Result<(), String> {

    // Two vectors to multiply upfront. That sets the accumulator register
    SPMEM::write_vector_into_dmem(0x00, &Vector::from_u16([0x7FFF, 0x7FFF, 0x7FFF, 0x0000, 0x0001, 0xFFFF, 0x7FFF, 0x8000]));
    SPMEM::write_vector_into_dmem(0x10, &Vector::from_u16([0x7FFF, 0xFFFF, 0x0010, 0x0000, 0xFFFF, 0xFFFF, 0x7FFF, 0x8000]));

    // The actual input data for the instruction
    SPMEM::write_vector_into_dmem(0x20, &vector1);
    SPMEM::write_vector_into_dmem(0x30, &vector2);

    // Assemble RSP program
    let mut assembler = RSPAssembler::new(0);

    // Do a multiplication to ensure that the accumulator bits are set
    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);
    assembler.write_vmudh(VR::V2, VR::V0, VR::V1, Element::All);
    assembler.write_vmadn(VR::V2, VR::V0, VR::V1, Element::All);

    // The accumulators will now be as follows:
    //    high  mid  low
    // 0: 3FFF 4000 0001
    // 1: FFFF FFFF 8001
    // 2: 0007 FFF7 FFF0
    // 3: 0000 0000 0000
    // 4: FFFF FFFF FFFF
    // 5: 0000 0000 0001
    // 6: 3FFF 4000 0001
    // 7: 3FFF C000 0000
    let acc_high = Vector::from_u16([0x3FFF, 0xFFFF, 0x0007, 0x0000, 0xFFFF, 0x0000, 0x3FFF, 0x3FFF]);
    let acc_mid = Vector::from_u16([0x4000, 0xFFFF, 0xFFF7, 0x0000, 0xFFFF, 0x0000, 0x4000, 0xC000]);

    // Set flags
    assembler.write_li(GPR::AT, vco as u32);
    assembler.write_ctc2(CP2FlagsRegister::VCO, GPR::AT);
    assembler.write_li(GPR::AT, vcc as u32);
    assembler.write_ctc2(CP2FlagsRegister::VCC, GPR::AT);
    assembler.write_li(GPR::AT, vce as u32);
    assembler.write_ctc2(CP2FlagsRegister::VCE, GPR::AT);

    // Load the actual input
    assembler.write_lqv(VR::V0, E::_0, 0x020, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x030, GPR::R0);

    // Perform the calculation
    emitter(&mut assembler);

    // Get flags and accumulators
    assembler.write_cfc2(CP2FlagsRegister::VCO, GPR::S0);
    assembler.write_cfc2(CP2FlagsRegister::VCC, GPR::S1);
    assembler.write_cfc2(CP2FlagsRegister::VCE, GPR::S2);
    assembler.write_vsar(VR::V3, VSARAccumulator::High);
    assembler.write_vsar(VR::V4, VSARAccumulator::Mid);
    assembler.write_vsar(VR::V5, VSARAccumulator::Low);

    assembler.write_sw(GPR::S0, GPR::R0, 0x90);
    assembler.write_sw(GPR::S1, GPR::R0, 0x94);
    assembler.write_sw(GPR::S2, GPR::R0, 0x98);
    assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
    assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V5, E::_0, 0x130, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq_vector(SPMEM::read_vector_from_dmem(0x100), expected_result, || "Output register (main calculation result)".to_string())?;
    soft_assert_eq(SPMEM::read(0x90) as u16, expected_vco, "VCO after calculation")?;
    soft_assert_eq(SPMEM::read(0x94) as u16, expected_vcc, "VCC after calculation")?;
    soft_assert_eq(SPMEM::read(0x98) as u8, expected_vce, "VCE after calculation")?;
    soft_assert_eq_vector(SPMEM::read_vector_from_dmem(0x130), expected_acc_low, || "Acc[0..8] after calculation".to_string())?;
    soft_assert_eq_vector(SPMEM::read_vector_from_dmem(0x120), acc_mid, || "Acc[16..32] after calculation".to_string())?;
    soft_assert_eq_vector(SPMEM::read_vector_from_dmem(0x110), acc_high, || "Acc[32..48] after calculation".to_string())?;

    Ok(())
}

pub struct VADD {}

impl Test for VADD {
    fn name(&self) -> &str { "RSP VADD" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE and VCC are ignore and left alone, so put some random stuff in there
        // The upper bits of VCO are ignored but then cleared. Fill them with random stuff as well
        run_test(
            0x8E00,
            0x1234,
            0x89,
            |assembler| { assembler.write_vadd(VR::V2, VR::V0, VR::V1, Element::All); },
            Vector::from_u16([0, 1, 0x8000, 0xFFFF, 0x7fff, 0x8001, 0x8000, 0x0001]),
            Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x7fff, 0x8001, 0xFFFF, 0xFFFF]),
            0,
            0x1234,
            0x89,
            Vector::from_u16([0, 3, 0xFFFF, 0x7FFE, 0x7FFF, 0x8000, 0x8000, 0]),
            Vector::from_u16([0, 3, 0xFFFF, 0x7FFE, 0xFFFE, 0x0002, 0x7FFF, 0]))
    }
}

pub struct VADDWithVCO {}

impl Test for VADDWithVCO {
    fn name(&self) -> &str { "RSP VADD (with VCO set)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE and VCC are ignore and left alone, so put some random stuff in there
        // For VCO, the upper bits are zeroed out
        // VCO lower (which actually changes) the result: Every odd bit is set
        run_test(
            0xFFAA,
            0x1234,
            0x89,
            |assembler| { assembler.write_vadd(VR::V2, VR::V0, VR::V1, Element::All); },
            Vector::from_u16([1, 1, 0x8000, 0x8000, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF]),
            Vector::from_u16([2, 2, 0xFFFF, 0xFFFF, 0x0001, 0x0001, 0xFFFF, 0xFFFF]),
            0x0000,
            0x1234,
            0x89,
            Vector::from_u16([3, 4, 0x8000, 0x8000, 0x7FFF, 0x7FFF, 0x7FFE, 0x7FFF]),
            Vector::from_u16([3, 4, 0x7FFF, 0x8000, 0x8000, 0x8001, 0x7FFE, 0x7FFF]))
    }
}

pub struct VADDWithVCOAndElementSpecifier {}

impl Test for VADDWithVCOAndElementSpecifier {
    fn name(&self) -> &str { "RSP VADD (with Element specifier)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE and VCC are ignore and left alone, so put some random stuff in there
        // For VCO, the upper bits are zeroed out
        // VCO lower (which actually changes) the result: Every odd bit is set
        run_test(
            0xFFAA,
            0x1234,
            0x89,
            |assembler| { assembler.write_vadd(VR::V2, VR::V0, VR::V1, Element::H1); },
            Vector::from_u16([1, 1, 0x8000, 0x8000, 0x7FFF, 0x7FFF, 0x7FFF, 0x7FFF]),
            Vector::from_u16([2, 2, 0xFFFF, 0xFFFF, 0x0001, 0x0001, 0xFFFF, 0xFFFF]),
            0x0000,
            0x1234,
            0x89,
            Vector::from_u16([3, 4, 0, 1, 0x7FFF, 0x7FFF, 0x7FFE, 0x7FFF]),
            Vector::from_u16([3, 4, 0, 1, 0x8000, 0x8001, 0x7FFE, 0x7FFF]))
    }
}

pub struct VSUB {}

impl Test for VSUB {
    fn name(&self) -> &str { "RSP VSUB" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE and VCC are ignore and left alone, so put some random stuff in there
        // The upper bits of VCO are ignored but then cleared. Fill them with random stuff as well
        run_test(
            0x8E00,
            0x1234,
            0x89,
            |assembler| { assembler.write_vsub(VR::V2, VR::V0, VR::V1, Element::All); },
            Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x8000]),
            Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0x7FFF]),
            0,
            0x1234,
            0x89,
            Vector::from_u16([0, 1, 0x7FEF, 0x7FFF, 0x8001, 0x8000, 0x8000, 0x7FFF]),
            Vector::from_u16([0, 1, 0x7FEF, 0x8000, 0x8001, 0x8000, 0x7FFF, 0xFFFF]))
    }
}

pub struct VSUBWithVCO {}

impl Test for VSUBWithVCO {
    fn name(&self) -> &str { "RSP VSUBWithVCO" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE and VCC are ignored and left alone, so put some random stuff in there
        // The upper bits of VCO are ignored but then cleared. Fill them with random stuff as well
        run_test(
            0xFFAA,
            0x1234,
            0x89,
            |assembler| { assembler.write_vsub(VR::V2, VR::V0, VR::V1, Element::Q0); },
            Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x8000]),
            Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0x7FFF]),
            0,
            0x1234,
            0x89,
            Vector::from_u16([0, 1, 0x7FEF, 0x7FEE, 0x8001, 0x8000, 0x8000, 0xFFFF]),
            Vector::from_u16([0, 1, 0x7FEF, 0x7FEE, 0x8001, 0x7FFF, 0x7FFF, 0xFFFF]))
    }
}

pub struct VSUT {}

impl Test for VSUT {
    fn name(&self) -> &str { "RSP VSUT" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE, VCC and VCO are ignored and left alone. Put some random stuff in there
        // The target register is cleared
        // The accumulator register is set to the sum of the two input registers
        // The upper bits of VCO are ignored but then cleared. Fill them with random stuff as well
        run_test(
            0x8E11,
            0x1234,
            0x89,
            |assembler| { assembler.write_vsut(VR::V2, VR::V0, VR::V1, Element::All); },
            Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x8000]),
            Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0x7FFF]),
            0x8E11,
            0x1234,
            0x89,
            Vector::from_u16([0, 0, 0, 0, 0, 0, 0, 0]),
            Vector::from_u16([0, 3, 0x800F, 0x7FFE, 0x7FFF, 0x7FFE, 0x7FFD, 0xFFFF]))
    }
}

pub struct VSUTH1 {}

impl Test for VSUTH1 {
    fn name(&self) -> &str { "RSP VSUT (H1)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE, VCC and VCO are ignored and left alone. Put some random stuff in there
        // The target register is cleared
        // The accumulator register is set to the sum of the two input registers
        // The upper bits of VCO are ignored but then cleared. Fill them with random stuff as well
        run_test(
            0x8E11,
            0x1234,
            0x89,
            |assembler| { assembler.write_vsut(VR::V2, VR::V0, VR::V1, Element::H1); },
            Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x8000]),
            Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0x7FFF]),
            0x8E11,
            0x1234,
            0x89,
            Vector::from_u16([0, 0, 0, 0, 0, 0, 0, 0]),
            Vector::from_u16([1, 3, 0x8000, 0x8000, 0x7FFF, 0x7FFE, 0x7FFD, 0xFFFE]))
    }
}

