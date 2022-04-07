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
    SPMEM::write_vector_16(0x00, &[0x0000, 0x0000, 0x0000, 0xE000, 0x8001, 0x8000, 0x7FFF, 0x8000]);
    SPMEM::write_vector_16(0x10, &[0x0000, 0x0001, 0xFFFF, 0xFFFF, 0x8000, 0x7FFF, 0x7FFF, 0x8000]);

    // Assemble RSP program. First use VMULF to set accumulator to something known, then use VMADM
    let mut assembler = RSPAssembler::new(0);

    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);

    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, Element::All);
    assembler.write_vmadm(VR::V2, VR::V0, VR::V1, e);

    assembler.write_vsar(VR::V3, VR::V0, VR::V0, E::_8);
    assembler.write_vsar(VR::V4, VR::V0, VR::V0, E::_9);
    assembler.write_vsar(VR::V5, VR::V0, VR::V0, E::_10);

    assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
    assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V5, E::_0, 0x130, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq(SPMEM::read_vector_16(0x100), expected_result, "VMADM result")?;
    soft_assert_eq(SPMEM::read_vector_16(0x110), expected_acc_top, "VMADM Acc[32..48]")?;
    soft_assert_eq(SPMEM::read_vector_16(0x120), expected_acc_mid, "VMADM Acc[16..32]")?;
    soft_assert_eq(SPMEM::read_vector_16(0x130), expected_acc_low, "VMADM Acc[0..8]")?;

    Ok(())
}

pub struct VMADMAll {}

impl Test for VMADMAll {
    fn name(&self) -> &str { "RSP VMADM" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::All,
            [0, 0, 0, 0xffff, 0x3fff, 0xc001, 0x7fff, 0x4000],
            [0, 0, 0, 0xffff, 0, 0xffff, 0, 0],
            [0, 0, 0, 0xffff, 0x3fff, 0xc001, 0xbffd, 0x4000],
            [0x8000, 0x8000, 0x8000, 0xe000, 0, 0, 0x8003, 0x8000],
        )
    }
}

pub struct VMADM4 {}

impl Test for VMADM4 {
    fn name(&self) -> &str { "RSP VMADM (e=_4)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::_4,
            [0, 1, 0xffff, 0, 0x3fff, 0xc001, 0x7fff, 0x4000],
            [0, 0, 0xffff, 0, 0, 0xffff, 0, 0],
            [0, 1, 0xffff, 0, 0x3fff, 0xc001, 0xbffe, 0x4000],
            [0x8000, 1, 0xffff, 0x3fff, 0, 0x7fff, 0x8001, 0],
        )
    }
}

pub struct VMADMAccumulatorOverflowed {}

impl Test for VMADMAccumulatorOverflowed {
    fn name(&self) -> &str { "RSP VMADM (accumulator itself overflowed)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Prepare input data
        SPMEM::write_vector_16(0x00, &[0x8000, 0x7FFF, 0, 0, 0, 0, 0, 0]);
        SPMEM::write_vector_16(0x10, &[0x7FFF, 0x8000, 0, 0, 0, 0, 0, 0]);

        // Assemble RSP program. First use VMULF to set accumulator to something known, then use VMADM repeatedly
        let mut assembler = RSPAssembler::new(0);

        assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
        assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);

        assembler.write_vmulf(VR::V2, VR::V0, VR::V1, Element::All);

        // Loop begin. After 131075 iterations the second element will overflow
        assembler.write_li(GPR::A0, 131075);
        assembler.write_vmadm(VR::V2, VR::V0, VR::V1, Element::All);
        assembler.write_addiu(GPR::A0, GPR::A0, -1);
        assembler.write_bgtz(GPR::A0, -3);
        assembler.write_nop();  // delay

        assembler.write_vsar(VR::V3, VR::V0, VR::V0, E::_8);
        assembler.write_vsar(VR::V4, VR::V0, VR::V0, E::_9);
        assembler.write_vsar(VR::V5, VR::V0, VR::V0, E::_10);

        assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
        assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
        assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);
        assembler.write_sqv(VR::V5, E::_0, 0x130, GPR::R0);

        // After 4 more iterations the first element will overflow
        assembler.write_li(GPR::A0, 4);
        assembler.write_vmadm(VR::V2, VR::V0, VR::V1, Element::All);
        assembler.write_addiu(GPR::A0, GPR::A0, -1);
        assembler.write_bgtz(GPR::A0, -3);
        assembler.write_nop();  // delay

        assembler.write_vsar(VR::V3, VR::V0, VR::V0, E::_8);
        assembler.write_vsar(VR::V4, VR::V0, VR::V0, E::_9);
        assembler.write_vsar(VR::V5, VR::V0, VR::V0, E::_10);

        assembler.write_sqv(VR::V2, E::_0, 0x140, GPR::R0);
        assembler.write_sqv(VR::V3, E::_0, 0x150, GPR::R0);
        assembler.write_sqv(VR::V4, E::_0, 0x160, GPR::R0);
        assembler.write_sqv(VR::V5, E::_0, 0x170, GPR::R0);

        assembler.write_break();

        RSP::run_and_wait(0);

        // After first loop
        soft_assert_eq(SPMEM::read_vector_16(0x100), [0x7fff, 0x7fff, 0, 0, 0, 0, 0, 0], "Result after accumulator overflow in [1]")?;
        soft_assert_eq(SPMEM::read_vector_16(0x110), [0x7fff, 0x7fff, 0, 0, 0, 0, 0, 0], "Acc[32..48] after accumulator overflow in [1]")?;
        soft_assert_eq(SPMEM::read_vector_16(0x120), [0x4000, 0xc003, 0, 0, 0, 0, 0, 0], "Acc[16..32] after accumulator overflow in [1]")?;
        soft_assert_eq(SPMEM::read_vector_16(0x130), [0, 0, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000], "Acc[0..8] after accumulator overflow in [1]")?;

        // After second loop
        soft_assert_eq(SPMEM::read_vector_16(0x140), [0x8000, 0x7fff, 0, 0, 0, 0, 0, 0], "Result after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector_16(0x150), [0x8000, 0x7ffe, 0, 0, 0, 0, 0, 0], "Acc[32..48] after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector_16(0x160), [0x3ffe, 0xc005, 0, 0, 0, 0, 0, 0], "Acc[16..32] after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector_16(0x170), [0, 0, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000], "Acc[0..8] after accumulator overflow in [0]")?;

        Ok(())
    }
}
