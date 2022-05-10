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
    SPMEM::write_vector16_into_dmem(0x00, &[0x0000, 0x8000, 0xFFFF, 0x8000, 0x8001, 0x8000, 0x7FFF, 0x8000]);
    SPMEM::write_vector16_into_dmem(0x10, &[0x0000, 0x0001, 0xFFFF, 0xFFFF, 0x8000, 0x7FFF, 0x7FFF, 0x8000]);

    // Assemble RSP program. First use VMULF to set accumulator to something known, then use VMADN
    let mut assembler = RSPAssembler::new(0);

    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);
    assembler.write_lqv(VR::V6, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V7, E::_0, 0x010, GPR::R0);

    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, e);
    assembler.write_vmadn(VR::V2, VR::V0, VR::V1, e);

    assembler.write_vsar(VR::V3, VSARAccumulator::High);
    assembler.write_vsar(VR::V4, VSARAccumulator::Mid);
    assembler.write_vsar(VR::V5, VSARAccumulator::Low);

    assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
    assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V5, E::_0, 0x130, GPR::R0);

    // again but this time destructive by overwriting a source reg
    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, e);
    assembler.write_vmadn(VR::V6, VR::V6, VR::V1, e);
    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, e);
    assembler.write_vmadn(VR::V7, VR::V0, VR::V7, e);

    assembler.write_sqv(VR::V6, E::_0, 0x140, GPR::R0);
    assembler.write_sqv(VR::V7, E::_0, 0x150, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), SPMEM::read_vector16_from_dmem(0x140), "temp check")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), SPMEM::read_vector16_from_dmem(0x150), "temp check")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), expected_result, "VMADN result")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), expected_acc_top, "VMADN Acc[32..48]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), expected_acc_mid, "VMADN Acc[16..32]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), expected_acc_low, "VMADN Acc[0..16]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x140), expected_result, "VMADN result when doing VMADN V6, V6, V1")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x150), expected_result, "VMADN result when doing VMADN V7, V0, V7")?;

    Ok(())
}

pub struct VMADNAll {}

impl Test for VMADNAll {
    fn name(&self) -> &str { "RSP VMADN" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::All,
            [0x8000, 0, 0x8003, 0, 0, 0, 0xffff, 0x8000],
            [0, 0xffff, 0xffff, 0xffff, 0, 0xffff, 0, 0],
            [0, 0xffff, 0xffff, 0x8002, 0x4000, 0x4002, 0xbffd, 0x4000],
            [0x8000, 0, 0x8003, 0, 0, 0, 0x8003, 0x8000],
        )
    }
}

pub struct VMADNH3 {}

impl Test for VMADNH3 {
    fn name(&self) -> &str { "RSP VMADN (e=H3)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::H3,
            [0x8000, 0, 0, 0, 0x8000, 0, 0, 0x8000],
            [0, 0xffff, 0xffff, 0xffff, 0, 0xffff, 0xffff, 0],
            [0, 0xffff, 0x8002, 0x8002, 0x4000, 0x4002, 0x4002, 0x4000],
            [0x8000, 0, 0, 0, 0x8000, 0, 0, 0x8000],
        )
    }
}

pub struct VMADNH1 {}

impl Test for VMADNH1 {
    fn name(&self) -> &str { "RSP VMADN (e=H1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::H1,
            [0x8000, 0, 0, 0, 0x8000, 0, 0, 0x8000],
            [0, 0xffff, 0xffff, 0xffff, 0, 0xffff, 0xffff, 0],
            [0, 0xffff, 0x8002, 0x8002, 0x4000, 0x4002, 0x4002, 0x4000],
            [0x8000, 0, 0, 0, 0x8000, 0, 0, 0x8000],
        )
    }
}

pub struct VMADN6 {}

impl Test for VMADN6 {
    fn name(&self) -> &str { "RSP VMADN (e=_6)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::_6,
            [0x8000, 0xfffd, 3, 3, 0, 0xffff, 0xffff, 0],
            [0, 0, 0, 0, 0xffff, 0, 0, 0xffff],
            [0, 1, 0x7ffe, 0x7ffe, 0xc001, 0xbffd, 0xbffd, 0xc001],
            [0x8000, 0xfffd, 3, 3, 0, 0x8003, 0x8003, 0],
        )
    }
}

pub struct VMADNAccumulatorOverflowed {}

impl Test for VMADNAccumulatorOverflowed {
    fn name(&self) -> &str { "RSP VMADN (accumulator itself overflowed)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Prepare input data
        SPMEM::write_vector16_into_dmem(0x00, &[0x8321, 0x7FEC, 0, 0, 0, 0, 0, 0]);
        SPMEM::write_vector16_into_dmem(0x10, &[0x8123, 0x8123, 0, 0, 0, 0, 0, 0]);

        // Assemble RSP program. First use VMULF to set accumulator to something known, then use VMADN
        let mut assembler = RSPAssembler::new(0);

        assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
        assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);

        assembler.write_vmulf(VR::V2, VR::V0, VR::V1, Element::All);

        // Loop begin. After exactlty 130000 iterations the second element in the accumulator will have overflown
        assembler.write_li(GPR::A0, 130000);
        assembler.write_vmadn(VR::V2, VR::V0, VR::V1, Element::All);
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

        // Keep going for another 3175 iterations so that the first element overflows
        assembler.write_li(GPR::A0, 3176);
        assembler.write_vmadn(VR::V2, VR::V0, VR::V1, Element::All);
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
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), [0, 0, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000], "Result after accumulator overflow in [1]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), [0x830d, 0x8000, 0, 0, 0, 0, 0, 0], "Acc[32..48] after accumulator overflow in [1]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), [0x5e14, 0x1cb6, 0, 0, 0, 0, 0, 0], "Acc[16..32] after accumulator overflow in [1]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), [0xd876, 0x85c8, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000], "Acc[0..16] after accumulator overflow in [1]")?;

        // After second overflow
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x140), [0xffff, 0, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000], "Result after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x150), [0x7fff, 0x8320, 0, 0, 0, 0, 0, 0], "Acc[32..48] after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x160), [0xe5cc, 0xaca8, 0, 0, 0, 0, 0, 0], "Acc[16..32] after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x170), [0xe1ae, 0x7968, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000], "Acc[0..16] after accumulator overflow in [0]")?;

        Ok(())
    }
}
