use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
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

    // This is what the resulting vector will be filled with before the instruction runs
    SPMEM::write_vector_into_dmem(0x40, &Vector::from_u16([0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF]));

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
    assembler.write_lqv(VR::V4, E::_0, 0x020, GPR::R0);
    assembler.write_lqv(VR::V5, E::_0, 0x030, GPR::R0);

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

/// A couple of instructions add up the input vectors, put that on the accumulator and otherwise zero out
/// the target register
fn run_vzero<F: Fn(&mut RSPAssembler)>(emitter: F) -> Result<(), String> {
    // VCE, VCC and VCO are ignored and left alone. Put some random stuff in there
    // The target register is cleared
    // The accumulator register is set to the sum of the two input registers
    // The upper bits of VCO are ignored but then cleared. Fill them with random stuff as well
    run_test(
        0x8E11,
        0x1234,
        0x89,
        emitter,
        Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0xFFFF]),
        Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0xFFFF]),
        0x8E11,
        0x1234,
        0x89,
        Vector::from_u16([0, 0, 0, 0, 0, 0, 0, 0]),
        Vector::from_u16([0, 3, 0x800F, 0x7FFE, 0x7FFF, 0x7FFE, 0x7FFD, 0xFFFE]))
}

/// Some instructions do absolutely nothing
fn run_noop<F: Fn(&mut RSPAssembler)>(emitter: F) -> Result<(), String> {
    run_test(
        0x8E11,
        0x1234,
        0x89,
        emitter,
        Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0xFFFF]),
        Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0xFFFF]),
        0x8E11,
        0x1234,
        0x89,
        Vector::from_u16([0xFFFF, 0x8001, 0xFFFF, 0, 0xFFFF, 0x0001, 0xFFFF, 0xFFFF]),
        Vector::from_u16([0x0001, 0x8001, 0xFFF0, 0, 0xFFFF, 0x0001, 0x0001, 0x0000]))
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
            |assembler| { assembler.write_vadd(VR::V2, VR::V4, VR::V5, Element::All); },
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
            |assembler| { assembler.write_vadd(VR::V2, VR::V4, VR::V5, Element::All); },
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
            |assembler| { assembler.write_vadd(VR::V2, VR::V4, VR::V5, Element::H1); },
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
            |assembler| { assembler.write_vsub(VR::V2, VR::V4, VR::V5, Element::All); },
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
            |assembler| { assembler.write_vsub(VR::V2, VR::V4, VR::V5, Element::Q0); },
            Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x8000]),
            Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0x7FFF]),
            0,
            0x1234,
            0x89,
            Vector::from_u16([0, 1, 0x7FEF, 0x7FEE, 0x8001, 0x8000, 0x8000, 0xFFFF]),
            Vector::from_u16([0, 1, 0x7FEF, 0x7FEE, 0x8001, 0x7FFF, 0x7FFF, 0xFFFF]))
    }
}

pub struct VABS {}

impl Test for VABS {
    fn name(&self) -> &str { "RSP VABS" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE, VCC and VCO are ignored and left alone. Put some random stuff in there
        run_test(
            0x8E11,
            0x1234,
            0x89,
            |assembler| { assembler.write_vabs(VR::V2, VR::V4, VR::V5, Element::All); },
            Vector::from_u16([0x1234, 0x1234, 0x8765, 0x0001, 0xFFFF, 0x0000, 0x7FFF, 0x8000]),
            Vector::from_u16([0x0000, 0x0002, 0x0002, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF]),
            0x8E11,
            0x1234,
            0x89,
            Vector::from_u16([0x0000, 0x1234, 0x8765, 0xFFFF, 0x0001, 0x0000, 0x8001, 0x7FFF]),
            Vector::from_u16([0x0000, 0x1234, 0x8765, 0xFFFF, 0x0001, 0x0000, 0x8001, 0x8000]))
    }
}

pub struct VABSQ1 {}

impl Test for VABSQ1 {
    fn name(&self) -> &str { "RSP VABS (Q1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE, VCC and VCO are ignored and left alone. Put some random stuff in there
        run_test(
            0x8E11,
            0x1234,
            0x89,
            |assembler| { assembler.write_vabs(VR::V2, VR::V4, VR::V5, Element::Q1); },
            Vector::from_u16([0x1234, 0x1234, 0x8765, 0x0001, 0xFFFF, 0x0000, 0x7FFF, 0x8000]),
            Vector::from_u16([0x0000, 0x0002, 0x0002, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF]),
            0x8E11,
            0x1234,
            0x89,
            Vector::from_u16([0x0000, 0x1234, 0x0001, 0xFFFF, 0x0000, 0x0000, 0x7FFF, 0x7FFF]),
            Vector::from_u16([0x0000, 0x1234, 0x0001, 0xFFFF, 0x0000, 0x0000, 0x8000, 0x8000]))
    }
}

pub struct VADDC {}

impl Test for VADDC {
    fn name(&self) -> &str { "RSP VADDC" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE and VCC are ignored and left alone. Put some random stuff in there
        run_test(
            0x8E11,
            0x1234,
            0x89,
            |assembler| { assembler.write_vaddc(VR::V2, VR::V4, VR::V5, Element::All); },
            Vector::from_u16([0x0001, 0x7FFF, 0xF000, 0xF000, 0xFFFF, 0x8000, 0xFFFF, 0xFFFF]),
            Vector::from_u16([0x0001, 0x7FFF, 0x1000, 0xF001, 0xFFFF, 0xFFFF, 0x8000, 0x0001]),
            0x00FC,
            0x1234,
            0x89,
            Vector::from_u16([0x0002, 0xFFFE, 0x0000, 0xE001, 0xFFFE, 0x7FFF, 0x7FFF, 0x0000]),
            Vector::from_u16([0x0002, 0xFFFE, 0x0000, 0xE001, 0xFFFE, 0x7FFF, 0x7FFF, 0x0000]))
    }
}

pub struct VADDCH3 {}

impl Test for VADDCH3 {
    fn name(&self) -> &str { "RSP VADDC (H3)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE and VCC are ignored and left alone. Put some random stuff in there
        run_test(
            0x8E11,
            0x1234,
            0x89,
            |assembler| { assembler.write_vaddc(VR::V2, VR::V4, VR::V5, Element::H3); },
            Vector::from_u16([0x0001, 0x7FFF, 0xF000, 0xF000, 0xFFFF, 0x8000, 0xFFFF, 0xFFFF]),
            Vector::from_u16([0x0001, 0x7FFF, 0x1000, 0xF001, 0xFFFF, 0xFFFF, 0x8000, 0x0001]),
            0x00FE,
            0x1234,
            0x89,
            Vector::from_u16([0xF001, 0x6FFF, 0x0000, 0xE001, 0xFFFE, 0xFFFE, 0x7FFF, 0x0000]),
            Vector::from_u16([0xF001, 0x6FFF, 0x0000, 0xE001, 0xFFFE, 0xFFFE, 0x7FFF, 0x0000]))
    }
}

pub struct VSUBC {}

impl Test for VSUBC {
    fn name(&self) -> &str { "RSP VSUBC" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE and VCC are ignored and left alone. Put some random stuff in there
        // VCO is not read but written to, based on the sign of the result:
        // - 0:   high: 0, low: 0
        // - >0:  high: 1, low: 0
        // - <0:  high: 1, low: 1
        run_test(
            0x8E11,
            0x1234,
            0x89,
            |assembler| { assembler.write_vsubc(VR::V2, VR::V4, VR::V5, Element::All); },
            Vector::from_u16([0x0001, 0x0002, 0xFFFF, 0x0000, 0xFFFF, 0x0050, 0x0050, 0x0050]),
            Vector::from_u16([0x0003, 0x0003, 0x0000, 0xFFFF, 0xFFFF, 0x004F, 0x0050, 0x0051]),
            0xAF24,
            0x1234,
            0x89,
            Vector::from_u16([0x0002, 0x0001, 0x0001, 0xFFFF, 0x0000, 0xFFFF, 0x0000, 0x0001]),
            Vector::from_u16([0x0002, 0x0001, 0x0001, 0xFFFF, 0x0000, 0xFFFF, 0x0000, 0x0001]))
    }
}

pub struct VSUBCE1 {}

impl Test for VSUBCE1 {
    fn name(&self) -> &str { "RSP VSUBC (e=1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE and VCC are ignored and left alone. Put some random stuff in there
        // VCO is not read but written to, based on the sign of the result:
        // - 0:   high: 0, low: 0
        // - >0:  high: 1, low: 0
        // - <0:  high: 1, low: 1
        run_test(
            0x8E11,
            0x1234,
            0x89,
            |assembler| { assembler.write_vsubc(VR::V2, VR::V4, VR::V5, Element::All1); },
            Vector::from_u16([0x0001, 0x0002, 0xFFFF, 0x0000, 0xFFFF, 0x0050, 0x0050, 0x0050]),
            Vector::from_u16([0x0003, 0x0003, 0x0000, 0xFFFF, 0xFFFF, 0x004F, 0x0050, 0x0051]),
            0xAF24,
            0x1234,
            0x89,
            Vector::from_u16([0x0002, 0x0001, 0x0001, 0xFFFF, 0x0000, 0xFFFF, 0x0000, 0x0001]),
            Vector::from_u16([0x0002, 0x0001, 0x0001, 0xFFFF, 0x0000, 0xFFFF, 0x0000, 0x0001]))
    }
}

pub struct VSUT {}

impl Test for VSUT {
    fn name(&self) -> &str { "RSP VSUT" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vsut(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VSUTH1 {}

impl Test for VSUTH1 {
    fn name(&self) -> &str { "RSP VSUT (H1)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VCE, VCC and VCO are ignored and left alone. Put some random stuff in there
        // The target register is cleared to 0
        // The accumulator register is set to the sum of the two input registers
        // The upper bits of VCO are ignored but then cleared. Fill them with random stuff as well
        run_test(
            0x8E11,
            0x1234,
            0x89,
            |assembler| { assembler.write_vsut(VR::V2, VR::V4, VR::V5, Element::H1); },
            Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x8000]),
            Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0x7FFF]),
            0x8E11,
            0x1234,
            0x89,
            Vector::from_u16([0, 0, 0, 0, 0, 0, 0, 0]),
            Vector::from_u16([1, 3, 0x8000, 0x8000, 0x7FFF, 0x7FFE, 0x7FFD, 0xFFFE]))
    }
}

pub struct VADDB {}

impl Test for VADDB {
    fn name(&self) -> &str { "RSP VADDB" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vaddb(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VSUBB {}

impl Test for VSUBB {
    fn name(&self) -> &str { "RSP VSUBB" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vsubb(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VACCB {}

impl Test for VACCB {
    fn name(&self) -> &str { "RSP VACCB" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vaccb(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VSUCB {}

impl Test for VSUCB {
    fn name(&self) -> &str { "RSP VSUCB" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vsucb(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VSAD {}

impl Test for VSAD {
    fn name(&self) -> &str { "RSP VSAD" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vsad(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VSAC {}

impl Test for VSAC {
    fn name(&self) -> &str { "RSP VSAC" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vsac(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}


pub struct VSUM {}

impl Test for VSUM {
    fn name(&self) -> &str { "RSP VSUM" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| {
            // Use fewer than 3 NOPs here and the test will fail on hardware - it seems that one
            // of the previous multiplications will still be able to write to the accumulator.
            // See test below
            assembler.write_nop();
            assembler.write_nop();
            assembler.write_nop();
            assembler.write_vsum(VR::V2, VR::V4, VR::V5, Element::All);
        })
    }
}

pub struct VSUMNoNops {}

impl Test for VSUMNoNops {
    fn name(&self) -> &str { "RSP VSUM (without NOPs before)" }

    fn level(&self) -> Level { Level::TooWeird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VSUM seems to broken - if it runs after a multiplication, the multiplication might still
        // be able to change (some) of the accumulator - the result is deterministic, so we'll keep
        // the test but this sounds like a bug that no one would probably ever need,
        // so the test it marked as TooWeird to prevent it from running
        run_test(
            0x8E11,
            0x1234,
            0x89,
            |assembler| { assembler.write_vsum(VR::V2, VR::V4, VR::V5, Element::All); },
            Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0xFFFF]),
            Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0xFFFF]),
            0x8E11,
            0x1234,
            0x89,
            Vector::from_u16([0, 0, 0, 0, 0, 0, 0, 0]),
            Vector::from_u16([0x4000, 0x0002, 0x8006, 0x7FFE, 0x7FFE, 0x7FFE, 0xBFFD, 0xBFFE]))
    }
}

pub struct VLT {}

impl Test for VLT {
    fn name(&self) -> &str { "RSP VLT" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // This instruction picks the smaller of the corresponding u16 in the vector. It also
        // writes the pick (0 or 1) into the lower half of VCC (while clearing the upper half)

        // VCE is ignored and not modified

        // In the case of equality, VCC picks the winner: If both the lower and upper bit
        // are set, vs is picked. Otherwise vt.

        run_test(
            0x8E00,
            0x1200,
            0xDE,
            |assembler| { assembler.write_vlt(VR::V2, VR::V4, VR::V5, Element::All); },
            Vector::from_u16([0x1234, 0x1234, 0x1234, 0xF234, 0xF234, 0xF234, 0xF234, 0x1234]),
            Vector::from_u16([0x1233, 0x1234, 0x1235, 0xF233, 0xF234, 0xF235, 0x1234, 0xF234]),
            0x0000,
            0x0089,
            0xDE,
            Vector::from_u16([0x1233, 0x1234, 0x1234, 0xF233, 0xF234, 0xF234, 0xF234, 0xF234]),
            Vector::from_u16([0x1233, 0x1234, 0x1234, 0xF233, 0xF234, 0xF234, 0xF234, 0xF234]))
    }
}

pub struct VLTAllEqualAndFlags {}

impl Test for VLTAllEqualAndFlags {
    fn name(&self) -> &str { "RSP VLT (all equal)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vcc) = value.downcast_ref::<u16>() {
            // VCO (high), VCO (low) decide whether to pick
            // VCE and VCC are ignored
            run_test(
                0b00001111_00110011,
                vcc,
                0b10101001,
                |assembler| { assembler.write_vlt(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                0x0000,
                0b00000011,
                0b10101001,
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VLTAllSmallerAndFlags {}

impl Test for VLTAllSmallerAndFlags {
    fn name(&self) -> &str { "RSP VLT (all smaller)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vcc) = value.downcast_ref::<u16>() {
            // VCO (high), VCO (low) decide whether to pick
            // VCE and VCC are ignored
            run_test(
                0b00001111_00110011,
                vcc,
                0b10101001,
                |assembler| { assembler.write_vlt(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                Vector::from_u16([0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233]),
                0x0000,
                0b11111111,
                0b10101001,
                Vector::from_u16([0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233]),
                Vector::from_u16([0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VLTAllLargerAndFlags {}

impl Test for VLTAllLargerAndFlags {
    fn name(&self) -> &str { "RSP VLT (all larger)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vcc) = value.downcast_ref::<u16>() {
            // VCO (high), VCO (low) decide whether to pick
            // VCE and VCC are ignored
            run_test(
                0b00001111_00110011,
                vcc,
                0b10101001,
                |assembler| { assembler.write_vlt(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233]),
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                0x0000,
                0b00000000,
                0b10101001,
                Vector::from_u16([0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233]),
                Vector::from_u16([0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VEQ {}

impl Test for VEQ {
    fn name(&self) -> &str { "RSP VEQ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // This instruction copies vt into the target register and sets VCC if rs and rt
        // are equal AND the upper bit in VCO is cleared. It will also clear the upper half of VCC

        // VCE is ignored and not modified
        // VCO is ignored and cleared

        // In the case of equality, VCC picks the winner: If both the lower and upper bit
        // are set, vs is picked. Otherwise vt.

        run_test(
            0x0012,
            0x1200,
            0xDE,
            |assembler| { assembler.write_veq(VR::V2, VR::V4, VR::V5, Element::All); },
            Vector::from_u16([0x1234, 0x1234, 0x1234, 0xF234, 0xF234, 0xF234, 0xF234, 0x1234]),
            Vector::from_u16([0x1233, 0x1234, 0x1235, 0xF233, 0xF234, 0xF235, 0x1234, 0xF234]),
            0x0000,
            0b00010010,
            0xDE,
            Vector::from_u16([0x1234, 0x1234, 0x1234, 0xF234, 0xF234, 0xF234, 0xF234, 0x1234]),
            Vector::from_u16([0x1234, 0x1234, 0x1234, 0xF234, 0xF234, 0xF234, 0xF234, 0x1234]))
    }
}

pub struct VEQAllDifferent {}

impl Test for VEQAllDifferent {
    fn name(&self) -> &str { "RSP VEQ (all different)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vcc) = value.downcast_ref::<u16>() {
            run_test(
                0b00001111_00110011,
                vcc,
                0b10101001,
                |assembler| { assembler.write_veq(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
                Vector::from_u16([0xAAAA, 0xBBBB, 0xCCCC, 0xDDDD, 0xEEEE, 0xFFFF, 0xEFEF, 0xEFEF]),
                0x0000,
                0x0000,
                0b10101001,
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]))?;
        }
        Ok(())
    }
}

pub struct VEQAllEqual {}

impl Test for VEQAllEqual {
    fn name(&self) -> &str { "RSP VEQ (all equal)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vcc) = value.downcast_ref::<u16>() {
            run_test(
                0b00001111_00110011,
                vcc,
                0b10101001,
                |assembler| { assembler.write_veq(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
                0x0000,
                0x00F0,
                0b10101001,
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]))?;
        }
        Ok(())
    }
}

pub struct VNE {}

impl Test for VNE {
    fn name(&self) -> &str { "RSP VNE" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // This instruction copies vt into the target register and sets VCC if rs and rt
        // are different OR the upper bit in VCO is set. It will also clear the upper half of VCC

        // VCE is ignored and not modified
        // VCO is ignored and cleared

        // In the case of equality, VCC picks the winner: If both the lower and upper bit
        // are set, vs is picked. Otherwise vt.

        run_test(
            0x0012,
            0x1200,
            0xDE,
            |assembler| { assembler.write_vne(VR::V2, VR::V4, VR::V5, Element::All); },
            Vector::from_u16([0x1234, 0x1234, 0x1234, 0xF234, 0xF234, 0xF234, 0xF234, 0x1234]),
            Vector::from_u16([0x1233, 0x1234, 0x1235, 0xF233, 0xF234, 0xF235, 0x1234, 0xF234]),
            0x0000,
            0b11101101,
            0xDE,
            Vector::from_u16([0x1233, 0x1234, 0x1235, 0xF233, 0xF234, 0xF235, 0x1234, 0xF234]),
            Vector::from_u16([0x1233, 0x1234, 0x1235, 0xF233, 0xF234, 0xF235, 0x1234, 0xF234]))
    }
}


pub struct VNEAllDifferent {}

impl Test for VNEAllDifferent {
    fn name(&self) -> &str { "RSP VNE (all different)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vcc) = value.downcast_ref::<u16>() {
            run_test(
                0b00001111_00110011,
                vcc,
                0b10101001,
                |assembler| { assembler.write_vne(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
                Vector::from_u16([0xAAAA, 0xBBBB, 0xCCCC, 0xDDDD, 0xEEEE, 0xFFFF, 0xEFEF, 0xEFEF]),
                0x0000,
                0x00FF,
                0b10101001,
                Vector::from_u16([0xAAAA, 0xBBBB, 0xCCCC, 0xDDDD, 0xEEEE, 0xFFFF, 0xEFEF, 0xEFEF]),
                Vector::from_u16([0xAAAA, 0xBBBB, 0xCCCC, 0xDDDD, 0xEEEE, 0xFFFF, 0xEFEF, 0xEFEF]))?;
        }
        Ok(())
    }
}

pub struct VNEAllEqual {}

impl Test for VNEAllEqual {
    fn name(&self) -> &str { "RSP VNE (all equal)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vcc) = value.downcast_ref::<u16>() {
            run_test(
                0b00001111_00110011,
                vcc,
                0b10101001,
                |assembler| { assembler.write_vne(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
                0x0000,
                0x000F,
                0b10101001,
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]))?;
        }
        Ok(())
    }
}

pub struct VGE {}

impl Test for VGE {
    fn name(&self) -> &str { "RSP VGE" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // This instruction picks the larger of the corresponding u16 in the vector. It also
        // writes the pick (0 or 1) into the lower half of VCC (while clearing the upper half)

        // VCE is ignored and not modified

        // In the case of equality, VCC picks the winner: If neither the lower nor the upper bit
        // are set, vs is picked. Otherwise vt.

        run_test(
            0x8E00,
            0x1200,
            0xDE,
            |assembler| { assembler.write_vge(VR::V2, VR::V4, VR::V5, Element::All); },
            Vector::from_u16([0x1234, 0x1234, 0x1234, 0xF234, 0xF234, 0xF234, 0xF234, 0x1234]),
            Vector::from_u16([0x1233, 0x1234, 0x1235, 0xF233, 0xF234, 0xF235, 0x1234, 0xF234]),
            0x0000,
            0x0076,
            0xDE,
            Vector::from_u16([0x1234, 0x1234, 0x1235, 0xF234, 0xF234, 0xF235, 0x1234, 0x1234]),
            Vector::from_u16([0x1234, 0x1234, 0x1235, 0xF234, 0xF234, 0xF235, 0x1234, 0x1234]))
    }
}

pub struct VGEAllEqualAndFlags {}

impl Test for VGEAllEqualAndFlags {
    fn name(&self) -> &str { "RSP VGE (all equal)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vcc) = value.downcast_ref::<u16>() {
            // VCO (high), VCO (low) decide whether to pick
            // VCE and VCC are ignored
            run_test(
                0b00001111_00110011,
                vcc,
                0b10101001,
                |assembler| { assembler.write_vge(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                0x0000,
                0b11111100,
                0b10101001,
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VGEAllSmallerAndFlags {}

impl Test for VGEAllSmallerAndFlags {
    fn name(&self) -> &str { "RSP VGE (all smaller)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vcc) = value.downcast_ref::<u16>() {
            // VCO (high), VCO (low) decide whether to pick
            // VCE and VCC are ignored
            run_test(
                0b00001111_00110011,
                vcc,
                0b10101001,
                |assembler| { assembler.write_vge(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                Vector::from_u16([0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233]),
                0x0000,
                0b00000000,
                0b10101001,
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VGEAllLargerAndFlags {}

impl Test for VGEAllLargerAndFlags {
    fn name(&self) -> &str { "RSP VGE (all larger)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vcc) = value.downcast_ref::<u16>() {
            // VCO (high), VCO (low) decide whether to pick
            // VCE and VCC are ignored
            run_test(
                0b00001111_00110011,
                vcc,
                0b10101001,
                |assembler| { assembler.write_vge(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233, 0x1233]),
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                0x0000,
                0b11111111,
                0b10101001,
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]),
                Vector::from_u16([0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VMRG {}

impl Test for VMRG {
    fn name(&self) -> &str { "RSP VMRG" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vco) = value.downcast_ref::<u16>() {
            run_test(
                vco,
                0b00001111_00110011,
                0b10101001,
                |assembler| { assembler.write_vmrg(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
                Vector::from_u16([0xAAAA, 0xBBBB, 0xCCCC, 0xDDDD, 0xEEEE, 0xFFFF, 0xEFEF, 0xEFEF]),
                0,
                0b00001111_00110011,
                0b10101001,
                Vector::from_u16([0xAAAA, 0xBBBB, 0x3333, 0x4444, 0xEEEE, 0xFFFF, 0x7777, 0x8888]),
                Vector::from_u16([0xAAAA, 0xBBBB, 0x3333, 0x4444, 0xEEEE, 0xFFFF, 0x7777, 0x8888]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VAND {}

impl Test for VAND {
    fn name(&self) -> &str { "RSP VAND" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vco) = value.downcast_ref::<u16>() {
            run_test(
                vco,
                0b00001111_00110011,
                0b10101001,
                |assembler| { assembler.write_vand(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
                Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]),
                vco,
                0b00001111_00110011,
                0b10101001,
                Vector::from_u16([0x1101, 0x0200, 0x0000, 0x4444, 0x3000, 0x0000, 0x0000, 0xEFEF]),
                Vector::from_u16([0x1101, 0x0200, 0x0000, 0x4444, 0x3000, 0x0000, 0x0000, 0xEFEF]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VNAND {}

impl Test for VNAND {
    fn name(&self) -> &str { "RSP VNAND" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vco) = value.downcast_ref::<u16>() {
            run_test(
                vco,
                0b00001111_00110011,
                0b10101001,
                |assembler| { assembler.write_vnand(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
                Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]),
                vco,
                0b00001111_00110011,
                0b10101001,
                Vector::from_u16([0xEEFE, 0xFDFF, 0xFFFF, 0xBBBB, 0xCFFF, 0xFFFF, 0xFFFF, 0x1010]),
                Vector::from_u16([0xEEFE, 0xFDFF, 0xFFFF, 0xBBBB, 0xCFFF, 0xFFFF, 0xFFFF, 0x1010]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VOR {}

impl Test for VOR {
    fn name(&self) -> &str { "RSP VOR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vco) = value.downcast_ref::<u16>() {
            run_test(
                vco,
                0b00001111_00110011,
                0b10101001,
                |assembler| { assembler.write_vor(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
                Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]),
                vco,
                0b00001111_00110011,
                0b10101001,
                Vector::from_u16([0xFF1F, 0xFF65, 0x3333, 0xFFFF, 0xB3D7, 0x6666, 0xEFEF, 0xFFFF]),
                Vector::from_u16([0xFF1F, 0xFF65, 0x3333, 0xFFFF, 0xB3D7, 0x6666, 0xEFEF, 0xFFFF]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VNOR {}

impl Test for VNOR {
    fn name(&self) -> &str { "RSP VNOR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vco) = value.downcast_ref::<u16>() {
            run_test(
                vco,
                0b00001111_00110011,
                0b10101001,
                |assembler| { assembler.write_vnor(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
                Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]),
                vco,
                0b00001111_00110011,
                0b10101001,
                Vector::from_u16([0x00E0, 0x009A, 0xCCCC, 0x0000, 0x4C28, 0x9999, 0x1010, 0x0000]),
                Vector::from_u16([0x00E0, 0x009A, 0xCCCC, 0x0000, 0x4C28, 0x9999, 0x1010, 0x0000]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VXOR {}

impl Test for VXOR {
    fn name(&self) -> &str { "RSP VXOR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vco) = value.downcast_ref::<u16>() {
            run_test(
                vco,
                0b00001111_00110011,
                0b10101001,
                |assembler| { assembler.write_vxor(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
                Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]),
                vco,
                0b00001111_00110011,
                0b10101001,
                Vector::from_u16([0xEE1E, 0xFD65, 0x3333, 0xBBBB, 0x83D7, 0x6666, 0xEFEF, 0x1010]),
                Vector::from_u16([0xEE1E, 0xFD65, 0x3333, 0xBBBB, 0x83D7, 0x6666, 0xEFEF, 0x1010]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VNXOR {}

impl Test for VNXOR {
    fn name(&self) -> &str { "RSP VNXOR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x0000u16),
            Box::new(0xFF00u16),
            Box::new(0x00FFu16),
            Box::new(0xFFFFu16),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some(&vco) = value.downcast_ref::<u16>() {
            run_test(
                vco,
                0b00001111_00110011,
                0b10101001,
                |assembler| { assembler.write_vnxor(VR::V2, VR::V4, VR::V5, Element::All); },
                Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
                Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]),
                vco,
                0b00001111_00110011,
                0b10101001,
                Vector::from_u16([0x11E1, 0x029A, 0xCCCC, 0x4444, 0x7C28, 0x9999, 0x1010, 0xEFEF]),
                Vector::from_u16([0x11E1, 0x029A, 0xCCCC, 0x4444, 0x7C28, 0x9999, 0x1010, 0xEFEF]))?;
            Ok(())
        } else {
            panic!("Invalid value")
        }
    }
}

pub struct VNOP {}

impl Test for VNOP {
    fn name(&self) -> &str { "RSP VNOP" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_noop(|assembler| { assembler.write_vnop(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VEXTT {}

impl Test for VEXTT {
    fn name(&self) -> &str { "RSP VEXTT" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vextt(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VEXTQ {}

impl Test for VEXTQ {
    fn name(&self) -> &str { "RSP VEXTQ" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vextq(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VEXTN {}

impl Test for VEXTN {
    fn name(&self) -> &str { "RSP VEXTN" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vextn(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VINST {}

impl Test for VINST {
    fn name(&self) -> &str { "RSP VINST" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vinst(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VINSQ {}

impl Test for VINSQ {
    fn name(&self) -> &str { "RSP VINSQ" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vinsq(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VINSN {}

impl Test for VINSN {
    fn name(&self) -> &str { "RSP VINSN" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(|assembler| { assembler.write_vinsn(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}

pub struct VNULL {}

impl Test for VNULL {
    fn name(&self) -> &str { "RSP VNULL" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_noop(|assembler| { assembler.write_vnull(VR::V2, VR::V4, VR::V5, Element::All); })
    }
}
