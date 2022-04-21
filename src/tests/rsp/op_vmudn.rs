use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, Element, GPR, RSPAssembler, VR};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

fn run_test(e: Element, expected_result: [u16; 8], expected_acc_top: [u16; 8], expected_acc_mid: [u16; 8], expected_acc_low: [u16; 8]) -> Result<(), String> {
    // Prepare input data
    SPMEM::write_vector16_into_dmem(0x00, &[0x0000, 0x8000, 0xFFFF, 0x8000, 0x8001, 0x8000, 0x7FFF, 0x8000]);
    SPMEM::write_vector16_into_dmem(0x10, &[0x0000, 0x0001, 0xFFFF, 0xFFFF, 0x8000, 0x7FFF, 0x7FFF, 0x8000]);

    // Assemble RSP program. First use VMULF to set accumulator to something known, then use VMUDN
    let mut assembler = RSPAssembler::new(0);

    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);

    assembler.write_vmudn(VR::V2, VR::V0, VR::V1, e);

    assembler.write_vsar(VR::V3, VR::V0, VR::V0, E::_8);
    assembler.write_vsar(VR::V4, VR::V0, VR::V0, E::_9);
    assembler.write_vsar(VR::V5, VR::V0, VR::V0, E::_10);

    assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
    assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V5, E::_0, 0x130, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), expected_result, "VMUDN result")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), expected_acc_top, "VMUDN Acc[32..48]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), expected_acc_mid, "VMUDN Acc[16..32]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), expected_acc_low, "VMUDN Acc[0..8]")?;

    Ok(())
}

pub struct VMUDNAll {}

impl Test for VMUDNAll {
    fn name(&self) -> &str { "RSP VMUDN" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::All,
            [0, 0x8000, 1, 0x8000, 0x8000, 0x8000, 1, 0],
            [0, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0, 0xffff],
            [0, 0xffff, 0xffff, 0x8000, 0xc000, 0xc000, 0x3fff, 0xc000],
            [0, 0x8000, 1, 0x8000, 0x8000, 0x8000, 1, 0],
        )
    }
}

pub struct VMUDNH2 {}

impl Test for VMUDNH2 {
    fn name(&self) -> &str { "RSP VMUDN (e=H2)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::H2,
            [0, 0xffff, 1, 1, 0x8000, 1, 1, 0x8000],
            [0, 0xffff, 0xffff, 0xffff, 0, 0, 0, 0],
            [0, 0xffff, 0xffff, 0xffff, 0x3fff, 0x3fff, 0x3fff, 0x3fff],
            [0, 0xffff, 1, 1, 0x8000, 1, 1, 0x8000],
        )
    }
}

pub struct VMUDN7 {}

impl Test for VMUDN7 {
    fn name(&self) -> &str { "RSP VMUDN (e=_7)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::_7,
            [0, 0x8000, 0x8000, 0x8000, 0, 0x8000, 0x8000, 0],
            [0, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff],
            [0, 0xffff, 0x8000, 0x8000, 0xc000, 0xc000, 0xc000, 0xc000],
            [0, 0x8000, 0x8000, 0x8000, 0, 0x8000, 0x8000, 0],
        )
    }
}
