use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::sync::atomic::{AtomicU16, Ordering};

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, Element, GPR, RSPAssembler, VR, VSARAccumulator};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

static COUNTER: AtomicU16 = AtomicU16::new(13);

fn rng() -> u16 {
    COUNTER.fetch_add(1234, Ordering::Relaxed)
}

fn run_test(e: Element, vs: VR, vt: VR, expected_result: [u16; 8], expected_acc_top: [u16; 8], expected_acc_mid: [u16; 8], expected_acc_low: [u16; 8]) -> Result<(), String> {
    // Data to pre-set accumulator
    SPMEM::write_vector16_into_dmem(0x00, &[0x0000, 0x0001, 0x0001, 0x7FFF, 0xFFFF, 0x7FFF, 0x3FFF, 0x8000]);
    SPMEM::write_vector16_into_dmem(0x10, &[0x0000, 0x0001, 0xFFFF, 0xFFFF, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF]);

    // Data for input. The second value is ignored, so we'll fill it with garbage
    SPMEM::write_vector16_into_dmem(0x20, &[0x0000, 0x0001, 0x0002, 0x7FFF, 0xFFFF, 0x8000, 0x8001, 0x8002]);
    SPMEM::write_vector16_into_dmem(0x30, &[rng(), rng(), rng(), rng(), rng(), rng(), rng(), rng()]);

    // Assemble RSP program
    let mut assembler = RSPAssembler::new(0);

    // Ensure the accumulator is filled
    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);

    assembler.write_vmudh(VR::V2, VR::V0, VR::V1, Element::All);
    assembler.write_vmadl(VR::V2, VR::V0, VR::V1, Element::All);

    // Load input data for actual test
    assembler.write_lqv(vt, E::_0, 0x020, GPR::R0);
    if vs != vt {
        assembler.write_lqv(vs, E::_0, 0x030, GPR::R0);
    }

    assembler.write_vrndp(VR::V2, vt, vs, e);

    assembler.write_vsar(VR::V3, VSARAccumulator::High);
    assembler.write_vsar(VR::V4, VSARAccumulator::Mid);
    assembler.write_vsar(VR::V5, VSARAccumulator::Low);

    assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
    assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V5, E::_0, 0x130, GPR::R0);
    assembler.write_sqv(VR::V6, E::_0, 0x140, GPR::R0);
    assembler.write_sqv(VR::V7, E::_0, 0x150, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), expected_result, "Result")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), expected_acc_top, "Acc[32..48]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), expected_acc_mid, "Acc[16..32]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), expected_acc_low, "Acc[0..16]")?;

    Ok(())
}

pub struct VRNDPWithEvenVS {}

impl Test for VRNDPWithEvenVS {
    fn name(&self) -> &str { "RSP VRNDP (even vs)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for i in (0..32).step_by(2) {
            let vt = if i == 0 { VR::V1 } else { VR::V0 };
            run_test(
                Element::All,
                VR::from_index(i).unwrap(),
                vt,
                [0, 1, 0xFFFF, 0x8001, 1, 0x7FFF, 0x7fff, 0x8000],
                [0, 0, 0xffff, 0xffff, 0, 0x3fff, 0x1fff, 0xc000],
                [0, 1, 0xffff, 0x8001, 1, 0, 0x4000, 0x8000],
                [0, 1, 0, 0x7ffe, 0xfffd, 0xbfff, 0xa000, 0x3fff],
            )?;
        }

        Ok(())
    }
}

pub struct VRNDPWithOddVS {}

impl Test for VRNDPWithOddVS {
    fn name(&self) -> &str { "RSP VRNDP (odd vs)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for i in (1..32).step_by(2) {
            let vt = VR::V0;
            run_test(
                Element::All,
                VR::from_index(i).unwrap(),
                vt,
                [0, 2, 0xFFFF, 0x8001, 0, 0x7FFF, 0x7fff, 0x8000],
                [0, 0, 0xffff, 0xffff, 0, 0x3ffe, 0x1ffe, 0xc000],
                [0, 2, 0xffff, 0x8001, 0, 0x8001, 0xc002, 0x8000],
                [0, 0, 0, 0x7ffe, 0xfffe, 0x3fff, 0x1fff, 0x3fff],
            )?;
        }

        Ok(())
    }
}

pub struct VRNDPOverwriteItselfWithElement {}

impl Test for VRNDPOverwriteItselfWithElement {
    fn name(&self) -> &str { "RSP VRNDP (overwrite itself with element specifier)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // V2 is the output, so also use it for input
        run_test(
            Element::H0,
            VR::V1,
            VR::V2,
            [0, 1, 0xffff, 0x8001, 0, 0x7fff, 0x7fff, 0x8000],
            [0, 0, 0xffff, 0xffff, 0, 0x3fff, 0x1fff, 0xc000],
            [0, 1, 0xffff, 0x8001, 0, 0, 0x4000, 0x8000],
            [0, 0, 0, 0x7ffe, 0xfffe, 0x3fff, 0x1fff, 0x3fff],
        )?;

        Ok(())
    }
}

pub struct VRNDPAccumulatorOverflowed {}

impl Test for VRNDPAccumulatorOverflowed {
    fn name(&self) -> &str { "RSP VRNDP (accumulator itself overflowed)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Prepare input data
        SPMEM::write_vector16_into_dmem(0x00, &[0x8000, 0x7FFF, 0x7FFF, 0, 0, 0, 0, 0]);
        SPMEM::write_vector16_into_dmem(0x10, &[0x8000, 0x8000, 0x7FFF, 0, 0, 0, 0, 0]);

        // Assemble RSP program. First use VMULF to set accumulator to something known, then use VMACU
        let mut assembler = RSPAssembler::new(0);

        assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
        assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);

        assembler.write_vmudh(VR::V2, VR::V0, VR::V1, Element::All);

        // Loop begin. After 32769 iterations the first element in the accumulator will flip from 0x0000 to 0xFFFF
        assembler.write_li(GPR::A0, 32769);
        assembler.write_vrndp(VR::V2, VR::V0, VR::V1, Element::All);
        assembler.write_addiu(GPR::A0, GPR::A0, -1);
        assembler.write_bgtz(GPR::A0, -3);
        assembler.write_nop();  // delay

        assembler.write_vsar_any_index(VR::V3, VR::V0, VR::V0, E::_8);
        assembler.write_vsar_any_index(VR::V4, VR::V0, VR::V0, E::_9);
        assembler.write_vsar_any_index(VR::V5, VR::V0, VR::V0, E::_10);

        assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
        assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
        assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);
        assembler.write_sqv(VR::V5, E::_0, 0x130, GPR::R0);

        // Keep going for another 3 iterations so that the third element flips from 0x7FFF to 0x8000
        assembler.write_li(GPR::A0, 3);
        assembler.write_vrndp(VR::V2, VR::V0, VR::V1, Element::All);
        assembler.write_addiu(GPR::A0, GPR::A0, -1);
        assembler.write_bgtz(GPR::A0, -3);
        assembler.write_nop();  // delay

        assembler.write_vsar_any_index(VR::V3, VR::V0, VR::V0, E::_8);
        assembler.write_vsar_any_index(VR::V4, VR::V0, VR::V0, E::_9);
        assembler.write_vsar_any_index(VR::V5, VR::V0, VR::V0, E::_10);

        assembler.write_sqv(VR::V2, E::_0, 0x140, GPR::R0);
        assembler.write_sqv(VR::V3, E::_0, 0x150, GPR::R0);
        assembler.write_sqv(VR::V4, E::_0, 0x160, GPR::R0);
        assembler.write_sqv(VR::V5, E::_0, 0x170, GPR::R0);

        assembler.write_break();

        RSP::run_and_wait(0);

        // After first overflow
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), [0x8000, 0x8000, 0x7FFF, 0, 0, 0, 0, 0], "Result after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), [0xffff, 0xc000, 0x7fff, 0, 0, 0, 0, 0], "Acc[32..48] after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), [0x8000, 0x8000, 0, 0, 0, 0, 0, 0], "Acc[16..32] after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), [0, 0, 0, 0, 0, 0, 0, 0], "Acc[0..16] after accumulator overflow in [0]")?;

        // After second overflow
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x140), [0x8000, 0x8000, 0x8000, 0, 0, 0, 0, 0], "Result after accumulator overflow in [3]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x150), [0xffff, 0xc000, 0x8000, 0, 0, 0, 0, 0], "Acc[32..48] after accumulator overflow in [3]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x160), [0x8000, 0x8000, 0x7ffd, 0, 0, 0, 0, 0], "Acc[16..32] after accumulator overflow in [3]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x170), [0, 0, 0, 0, 0, 0, 0, 0], "Acc[0..16] after accumulator overflow in [3]")?;

        Ok(())
    }
}
