use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, Element, GPR, RSPAssembler, VR, VSARAccumulator};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

fn run_test(e: Element, expected_result: [u16; 8], expected_acc_top: [u16; 8], expected_acc_mid: [u16; 8], expected_acc_low: [u16; 8]) -> Result<(), String> {
    // Prepare input data
    SPMEM::write_vector16_into_dmem(0x00, &[0x0000, 0x0000, 0x0000, 0xE000, 0x8001, 0x8000, 0x7FFF, 0x8000]);
    SPMEM::write_vector16_into_dmem(0x10, &[0x0000, 0x0001, 0xFFFF, 0xFFFF, 0x8000, 0x7FFF, 0x7FFF, 0x8000]);

    // Assemble RSP program. First use VMULF to set accumulator to something known, then use VMADL
    let mut assembler = RSPAssembler::new(0);

    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);
    assembler.write_lqv(VR::V6, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V7, E::_0, 0x010, GPR::R0);

    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, Element::All);
    assembler.write_vmadl(VR::V2, VR::V0, VR::V1, e);

    assembler.write_vsar(VR::V3, VSARAccumulator::High);
    assembler.write_vsar(VR::V4, VSARAccumulator::Mid);
    assembler.write_vsar(VR::V5, VSARAccumulator::Low);

    assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
    assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V5, E::_0, 0x130, GPR::R0);

    // again but this time destructive by overwriting a source reg
    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, Element::All);
    assembler.write_vmadl(VR::V6, VR::V6, VR::V1, e);
    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, Element::All);
    assembler.write_vmadl(VR::V7, VR::V0, VR::V7, e);

    assembler.write_sqv(VR::V6, E::_0, 0x140, GPR::R0);
    assembler.write_sqv(VR::V7, E::_0, 0x150, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), expected_result, "Result")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), expected_acc_top, "Acc[32..48]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), expected_acc_mid, "Acc[16..32]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), expected_acc_low, "Acc[0..16]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x140), expected_result, "Result when doing VMADL V6, V6, V1")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x150), expected_result, "Result when doing VMADL V7, V0, V7")?;

    Ok(())
}

pub struct VMADLAll {}

impl Test for VMADLAll {
    fn name(&self) -> &str { "RSP VMADL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::All,
            [0x8000, 0x8000, 0x8000, 0x9fff, 0xc000, 0xbfff, 0xc001, 0xffff],
            [0, 0, 0, 0, 0, 0xffff, 0, 0],
            [0, 0, 0, 1, 0x7fff, 0x8001, 0x7ffe, 0x8000],
            [0x8000, 0x8000, 0x8000, 0x9fff, 0xc000, 0xbfff, 0xc001, 0xc000],
        )
    }
}

pub struct VMADL4 {}

impl Test for VMADL4 {
    fn name(&self) -> &str { "RSP VMADL (e=_4)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::_4,
            [0x8000, 0x8000, 0, 0x4000, 0xc000, 0xbfff, 0xc001, 0xffff],
            [0, 0, 0, 0, 0, 0xffff, 0, 0],
            [0, 0, 1, 1, 0x7fff, 0x8001, 0x7ffe, 0x8000],
            [0x8000, 0x8000, 0, 0x4000, 0xc000, 0xbfff, 0xc001, 0xc000],
        )
    }
}

pub struct VMADLAccumulatorOverflowed {}

impl Test for VMADLAccumulatorOverflowed {
    fn name(&self) -> &str { "RSP VMADL (accumulator itself overflowed)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Prepare input data
        SPMEM::write_vector16_into_dmem(0x00, &[0x7FFF, 0, 0, 0, 0, 0, 0, 0]);
        SPMEM::write_vector16_into_dmem(0x10, &[0x7FFF, 0, 0, 0, 0, 0, 0, 0]);

        // Assemble RSP program.
        // Using VMADL alone it takes very long to overflow the accumulator, so let's use
        // other instruction to get it close to the maximum positive value
        let mut assembler = RSPAssembler::new(0);

        assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
        assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);

        // This sets Accumulator #0 to 3FFF 0001 0000
        assembler.write_vmudh(VR::V2, VR::V0, VR::V1, Element::All);
        // Increment accumulator #0 to 7FFE 0002 0000
        assembler.write_vmadh(VR::V2, VR::V0, VR::V1, Element::All);
        // Increment accumulator #0 to 7FFF FFFA 0008
        for _ in 0..8 {
            assembler.write_vmadm(VR::V2, VR::V0, VR::V1, Element::All);
        }

        // Loop begin. After 24 iterations the first element will overflow
        assembler.write_li(GPR::A0, 25);
        assembler.write_vmadl(VR::V2, VR::V0, VR::V1, Element::All);
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

        assembler.write_break();

        RSP::run_and_wait(0);

        // After positive overflow (accumulator is now negative)
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), [0, 0, 0, 0, 0, 0, 0, 0], "Result after accumulator overflow")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), [0x8000, 0, 0, 0, 0, 0, 0, 0], "Acc[32..48] after accumulator overflow")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), [0, 0, 0, 0, 0, 0, 0, 0], "Acc[16..32] after accumulator overflow")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), [0x3fef, 0, 0, 0, 0, 0, 0, 0], "Acc[0..16] after accumulator overflow")?;

        Ok(())
    }
}
