use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct LH {}

impl Test for LH {
    fn name(&self) -> &str { "RSP LH" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        // Pre-fill data
        SPMEM::write(0x000, 0xBADDECAF);
        SPMEM::write(0x004, 0x01234567);
        SPMEM::write(0xFFC, 0xBCAD7E8F);

        assembler.write_li(GPR::V0, 0x6);

        for gpr in GPR::S0..=GPR::S5 {
            assembler.write_lui(gpr, 0);
        }

        assembler.write_lh(GPR::S0, GPR::R0, 0x0);
        assembler.write_lh(GPR::S1, GPR::R0, 0x1);
        assembler.write_lh(GPR::S3, GPR::V0, 0x7FFD);
        assembler.write_lh(GPR::S2, GPR::V0, 0x0);
        assembler.write_lh(GPR::S4, GPR::R0, 0x1FFE);
        assembler.write_lh(GPR::S5, GPR::R0, 0x7FFF);

        assembler.write_sw(GPR::S0, GPR::R0, 0x10);
        assembler.write_sw(GPR::S1, GPR::R0, 0x14);
        assembler.write_sw(GPR::S2, GPR::R0, 0x18);
        assembler.write_sw(GPR::S3, GPR::R0, 0x1C);
        assembler.write_sw(GPR::S4, GPR::R0, 0x20);
        assembler.write_sw(GPR::S5, GPR::R0, 0x24);

        // Load into R0 and ensure it gets ignored
        assembler.write_lh(GPR::R0, GPR::R0, 0);
        assembler.write_sw(GPR::R0, GPR::V0, 0x22);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x10), 0xFFFFBADD, "LH FROM DMEM[0x00]")?;
        soft_assert_eq(SPMEM::read(0x14), 0xFFFFDDEC, "LH FROM DMEM[0x01]")?;
        soft_assert_eq(SPMEM::read(0x18), 0x00004567, "LH FROM DMEM[0x06]")?;
        soft_assert_eq(SPMEM::read(0x1C), 0xFFFFAF01, "LH FROM DMEM[0x03]")?;
        soft_assert_eq(SPMEM::read(0x20), 0x00007E8F, "LH FROM DMEM[0xFFE]")?;
        soft_assert_eq(SPMEM::read(0x24), 0xFFFF8FBA, "LH FROM DMEM[0xFFF]")?;
        soft_assert_eq(SPMEM::read(0x28), 0x00000000, "LH into R0")?;


        Ok(())
    }
}
