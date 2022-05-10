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

    // Assemble RSP program
    let mut assembler = RSPAssembler::new(0);

    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);

    assembler.write_lqv(VR::V6, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V7, E::_0, 0x010, GPR::R0);

    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, e);

    assembler.write_vsar(VR::V3, VSARAccumulator::High);
    assembler.write_vsar(VR::V4, VSARAccumulator::Mid);
    assembler.write_vsar(VR::V5, VSARAccumulator::Low);

    // again but this time destructive by overwriting a source reg
    assembler.write_vmulf(VR::V6, VR::V6, VR::V1, e);
    assembler.write_vmulf(VR::V7, VR::V0, VR::V7, e);

    assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
    assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V5, E::_0, 0x130, GPR::R0);
    assembler.write_sqv(VR::V6, E::_0, 0x140, GPR::R0);
    assembler.write_sqv(VR::V7, E::_0, 0x150, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), expected_result, "VMULF result")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), expected_acc_top, "VMULF Acc[32..48]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), expected_acc_mid, "VMULF Acc[16..32]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), expected_acc_low, "VMULF Acc[0..16]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x140), expected_result, "VMULF result when doing VMULF V6, V6, V1")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x150), expected_result, "VMULF result when doing VMULF V7, V0, V7")?;

    Ok(())
}

pub struct VMULFAll {}

impl Test for VMULFAll {
    fn name(&self) -> &str { "RSP VMULF" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::All,
            [0, 0, 0, 0, 0x7fff, 0x8001, 0x7ffe, 0x7fff],
            [0, 0, 0, 0, 0, 0xffff, 0, 0],
            [0, 0, 0, 0, 0x7fff, 0x8001, 0x7ffe, 0x8000],
            [0x8000, 0x8000, 0x8000, 0xc000, 0x8000, 0x8000, 0x8002, 0x8000],
        )
    }
}

pub struct VMULFAll1 {}

impl Test for VMULFAll1 {
    fn name(&self) -> &str { "RSP VMULF (e={1})" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::All1,
            [0, 0, 0, 0, 0x7fff, 0x8001, 0x7ffe, 0x7fff],
            [0, 0, 0, 0, 0, 0xffff, 0, 0],
            [0, 0, 0, 0, 0x7fff, 0x8001, 0x7ffe, 0x8000],
            [0x8000, 0x8000, 0x8000, 0xc000, 0x8000, 0x8000, 0x8002, 0x8000],
        )
    }
}

pub struct VMULFH0 {}

impl Test for VMULFH0 {
    fn name(&self) -> &str { "RSP VMULF (e=H0)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::H0,
            [0, 0, 0, 0, 0x7fff, 0x8002, 0x8002, 0x7fff],
            [0, 0, 0, 0, 0, 0xffff, 0xffff, 0],
            [0, 0, 0, 0, 0x7fff, 0x8002, 0x8002, 0x7fff],
            [0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x7ffe, 0x7ffe, 0x8000],
        )
    }
}

pub struct VMULFH1 {}

impl Test for VMULFH1 {
    fn name(&self) -> &str { "RSP VMULF (e=H1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::H1,
            [0, 0, 0, 0, 0x7fff, 0x8001, 0x8001, 0x7fff],
            [0, 0, 0, 0, 0, 0xffff, 0xffff, 0],
            [0, 0, 0, 0, 0x8000, 0x8001, 0x8001, 0x8000],
            [0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000],
        )
    }
}
