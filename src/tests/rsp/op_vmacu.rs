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

    // Assemble RSP program. First use VMULF to set accumulator to something known, then use VMACU
    let mut assembler = RSPAssembler::new(0);

    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);
    assembler.write_lqv(VR::V6, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V7, E::_0, 0x010, GPR::R0);

    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, Element::All);
    assembler.write_vmacu(VR::V2, VR::V0, VR::V1, e);

    assembler.write_vsar(VR::V3, VSARAccumulator::High);
    assembler.write_vsar(VR::V4, VSARAccumulator::Mid);
    assembler.write_vsar(VR::V5, VSARAccumulator::Low);

    assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
    assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V5, E::_0, 0x130, GPR::R0);

    // again but this time destructive by overwriting a source reg
    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, Element::All);
    assembler.write_vmacu(VR::V6, VR::V6, VR::V1, e);
    assembler.write_vmulf(VR::V2, VR::V0, VR::V1, Element::All);
    assembler.write_vmacu(VR::V7, VR::V0, VR::V7, e);

    assembler.write_sqv(VR::V6, E::_0, 0x140, GPR::R0);
    assembler.write_sqv(VR::V7, E::_0, 0x150, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), expected_result, "Result")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), expected_acc_top, "Acc[32..48]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), expected_acc_mid, "Acc[16..32]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), expected_acc_low, "Acc[0..16]")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x140), expected_result, "Result when doing VMACU V6, V6, V1")?;
    soft_assert_eq(SPMEM::read_vector16_from_dmem(0x150), expected_result, "Result when doing VMACU V7, V0, V7")?;

    Ok(())
}

pub struct VMACUAll {}

impl Test for VMACUAll {
    fn name(&self) -> &str { "RSP VMACU" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::All,
            [0, 0, 0, 1, 0xffff, 0, 0xffff, 0xffff],
            [0, 0, 0, 0, 0, 0xffff, 0, 1],
            [0, 0, 0, 1, 0xfffe, 2, 0xfffc, 0],
            [0x8000, 0x8000, 0x8000, 0, 0x8000, 0x8000, 0x8004, 0x8000],
        )
    }
}

pub struct VMACUH0 {}

impl Test for VMACUH0 {
    fn name(&self) -> &str { "RSP VMACU (e=H0)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::H0,
            [0, 0, 0, 0, 0xffff, 0, 0, 0xffff],
            [0, 0, 0, 0, 0, 0xffff, 0, 0],
            [0, 0, 0, 0, 0xfffe, 3, 0, 0xffff],
            [0x8000, 0x8000, 0x8000, 0xc000, 0x8000, 0x7ffe, 0x8000, 0x8000],
        )
    }
}

pub struct VMACU0 {}

impl Test for VMACU0 {
    fn name(&self) -> &str { "RSP VMACU (e=_0)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test(
            Element::_0,
            [0, 0, 0, 0, 0x7fff, 0, 0x7ffe, 0xffff],
            [0, 0, 0, 0, 0, 0xffff, 0, 0],
            [0, 0, 0, 0, 0x7fff, 0x8001, 0x7ffe, 0x8000],
            [0x8000, 0x8000, 0x8000, 0xc000, 0x8000, 0x8000, 0x8002, 0x8000],
        )
    }
}

pub struct VMACUAccumulatorOverflowed {}

impl Test for VMACUAccumulatorOverflowed {
    fn name(&self) -> &str { "RSP VMACU (accumulator itself overflowed)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Prepare input data
        SPMEM::write_vector16_into_dmem(0x00, &[0x8000, 0x7FFF, 0, 0, 0, 0, 0, 0]);
        SPMEM::write_vector16_into_dmem(0x10, &[0x8000, 0x8000, 0, 0, 0, 0, 0, 0]);

        // Assemble RSP program. First use VMULF to set accumulator to something known, then use VMACU
        let mut assembler = RSPAssembler::new(0);

        assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
        assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);

        assembler.write_vmulf(VR::V2, VR::V0, VR::V1, Element::All);

        // Loop begin. After 65535 iterations the first element in the accumulator will flip from 0x7FFF to 0x8000
        assembler.write_li(GPR::A0, 65535);
        assembler.write_vmacu(VR::V2, VR::V0, VR::V1, Element::All);
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

        // Keep going for another 3 iterations so that the second element flips from 0x8000 to 0x7FFF
        assembler.write_li(GPR::A0, 3);
        assembler.write_vmacu(VR::V2, VR::V0, VR::V1, Element::All);
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

        // Keep going for another 65533 iterations so that the first element flips from 0x8000 to 0x0000
        assembler.write_li(GPR::A0, 65533);
        assembler.write_vmacu(VR::V2, VR::V0, VR::V1, Element::All);
        assembler.write_addiu(GPR::A0, GPR::A0, -1);
        assembler.write_bgtz(GPR::A0, -3);
        assembler.write_nop();  // delay

        assembler.write_vsar_any_index(VR::V3, VR::V0, VR::V0, E::_8);
        assembler.write_vsar_any_index(VR::V4, VR::V0, VR::V0, E::_9);
        assembler.write_vsar_any_index(VR::V5, VR::V0, VR::V0, E::_10);

        assembler.write_sqv(VR::V2, E::_0, 0x180, GPR::R0);
        assembler.write_sqv(VR::V3, E::_0, 0x190, GPR::R0);
        assembler.write_sqv(VR::V4, E::_0, 0x1A0, GPR::R0);
        assembler.write_sqv(VR::V5, E::_0, 0x1B0, GPR::R0);

        assembler.write_break();

        RSP::run_and_wait(0);

        // After first overflow
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), [0, 0, 0, 0, 0, 0, 0, 0], "Result after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), [0x8000, 0x8001, 0, 0, 0, 0, 0, 0], "Acc[32..48] after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), [0, 0, 0, 0, 0, 0, 0, 0], "Acc[16..32] after accumulator overflow in [0]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), [0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000], "Acc[0..16] after accumulator overflow in [0]")?;

        // After second overflow
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x140), [0, 0xFFFF, 0, 0, 0, 0, 0, 0], "Result after accumulator overflow in [1]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x150), [0x8001, 0x7FFF, 0, 0, 0, 0, 0, 0], "Acc[32..48] after accumulator overflow in [1]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x160), [0x8000, 0x8003, 0, 0, 0, 0, 0, 0], "Acc[16..32] after accumulator overflow in [1]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x170), [0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000], "Acc[0..16] after accumulator overflow in [1]")?;

        // After third overflow
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x180), [0, 0xFFFF, 0, 0, 0, 0, 0, 0], "Result after accumulator overflow in [0] (2nd time)")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x190), [0, 0x0002, 0, 0, 0, 0, 0, 0], "Acc[32..48] after accumulator overflow in [0] (2nd time)")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x1A0), [0, 0, 0, 0, 0, 0, 0, 0], "Acc[16..32] after accumulator overflow in [0] (2nd time)")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x1B0), [0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000], "Acc[0..16] after accumulator overflow in [0] (2nd time)")?;

        Ok(())
    }
}
