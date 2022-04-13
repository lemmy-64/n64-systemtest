use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// LWU: For whatever reason, this exists on the RSP. It seems to do the same thing as LW

pub struct LWU {}

impl Test for LWU {
    fn name(&self) -> &str { "RSP LWU" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        // Pre-fill data
        SPMEM::write(0x000, 0xBADDECAF);
        SPMEM::write(0x004, 0x01234567);
        SPMEM::write(0x008, 0x00000000);
        SPMEM::write(0xFFC, 0xBCAD7E8F);

        for gpr in GPR::S0..=GPR::S7 {
            assembler.write_lui(gpr, 0);
        }

        assembler.write_li(GPR::V0, 0x6);

        assembler.write_lwu(GPR::S0, GPR::R0, 0x0);
        assembler.write_lwu(GPR::S1, GPR::R0, 0x1);
        assembler.write_lwu(GPR::S3, GPR::V0, 0x7FFD);
        assembler.write_lwu(GPR::S2, GPR::V0, 0x0);
        assembler.write_lwu(GPR::S4, GPR::R0, 0xFFC);
        assembler.write_lwu(GPR::S5, GPR::R0, 0x1FFD);
        assembler.write_lwu(GPR::S6, GPR::R0, 0x1FFE);
        assembler.write_lwu(GPR::S7, GPR::R0, 0x7FFF);

        assembler.write_sw(GPR::S0, GPR::R0, 0x10);
        assembler.write_sw(GPR::S1, GPR::R0, 0x14);
        assembler.write_sw(GPR::S2, GPR::R0, 0x18);
        assembler.write_sw(GPR::S3, GPR::R0, 0x1C);
        assembler.write_sw(GPR::S4, GPR::R0, 0x20);
        assembler.write_sw(GPR::S5, GPR::R0, 0x24);
        assembler.write_sw(GPR::S6, GPR::R0, 0x28);
        assembler.write_sw(GPR::S7, GPR::R0, 0x2C);

        // Load into R0 and ensure it gets ignored
        assembler.write_lwu(GPR::R0, GPR::R0, 0);
        assembler.write_sw(GPR::R0, GPR::V0, 0x2A);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x10), 0xBADDECAF, "LWU FROM DMEM[0x000]")?;
        soft_assert_eq(SPMEM::read(0x14), 0xDDECAF01, "LWU FROM DMEM[0x001]")?;
        soft_assert_eq(SPMEM::read(0x18), 0x45670000, "LWU FROM DMEM[0x006]")?;
        soft_assert_eq(SPMEM::read(0x1C), 0xAF012345, "LWU FROM DMEM[0x003]")?;
        soft_assert_eq(SPMEM::read(0x20), 0xBCAD7E8F, "LWU FROM DMEM[0xFFC]")?;
        soft_assert_eq(SPMEM::read(0x24), 0xAD7E8FBA, "LWU FROM DMEM[0xFFD]")?;
        soft_assert_eq(SPMEM::read(0x28), 0x7E8FBADD, "LWU FROM DMEM[0xFFE]")?;
        soft_assert_eq(SPMEM::read(0x2C), 0x8FBADDEC, "LWU FROM DMEM[0xFFF]")?;
        soft_assert_eq(SPMEM::read(0x30), 0x00000000, "LWU into R0")?;

        Ok(())
    }
}
