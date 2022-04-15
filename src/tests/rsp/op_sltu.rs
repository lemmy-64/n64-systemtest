use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct SLTU {}

impl Test for SLTU {
    fn name(&self) -> &str { "RSP SLT" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut assembler = RSPAssembler::new(0);

        assembler.write_li(GPR::T0, 0x00000004);
        assembler.write_li(GPR::T1, 0x00000004);
        assembler.write_li(GPR::T2, 0x00000005);
        assembler.write_li(GPR::T3, 0x00000000);
        assembler.write_li(GPR::T4, 0xFFFFFFF1);
        assembler.write_li(GPR::T5, 0xFFFFFFF2);

        assembler.write_sltu(GPR::S0, GPR::T0, GPR::T1);
        assembler.write_sltu(GPR::S1, GPR::T0, GPR::T2);
        assembler.write_sltu(GPR::S2, GPR::T0, GPR::T3);
        assembler.write_sltu(GPR::S3, GPR::T4, GPR::T5);
        assembler.write_sltu(GPR::S4, GPR::T0, GPR::T4);
        assembler.write_li(GPR::S5, 5);
        assembler.write_sltu(GPR::S5, GPR::T0, GPR::S5);
        assembler.write_li(GPR::S6, 3);
        assembler.write_sltu(GPR::S6, GPR::S6, GPR::T0);
        assembler.write_li(GPR::S7, 3);
        assembler.write_sltu(GPR::S7, GPR::S7, GPR::S7);
        assembler.write_sltu(GPR::T8, GPR::T0, GPR::T0);
        assembler.write_sltu(GPR::T9, GPR::T0, GPR::R0);
        assembler.write_sltu(GPR::K0, GPR::R0, GPR::T0);

        // Write results
        assembler.write_sw(GPR::S0, GPR::R0, 0x0);
        assembler.write_sw(GPR::S1, GPR::R0, 0x4);
        assembler.write_sw(GPR::S2, GPR::R0, 0x8);
        assembler.write_sw(GPR::S3, GPR::R0, 0xC);
        assembler.write_sw(GPR::S4, GPR::R0, 0x10);
        assembler.write_sw(GPR::S5, GPR::R0, 0x14);
        assembler.write_sw(GPR::S6, GPR::R0, 0x18);
        assembler.write_sw(GPR::S7, GPR::R0, 0x1C);
        assembler.write_sw(GPR::T8, GPR::R0, 0x20);
        assembler.write_sw(GPR::T9, GPR::R0, 0x24);
        assembler.write_sw(GPR::K0, GPR::R0, 0x28);

        // Write to R0
        assembler.write_li(GPR::AT, 0);
        assembler.write_sltu(GPR::R0, GPR::T0, GPR::T2);
        assembler.write_sw(GPR::R0, GPR::AT, 0x30);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x00), 0, "SLTU result 0x00")?;
        soft_assert_eq(SPMEM::read(0x04), 1, "SLTU result 0x04")?;
        soft_assert_eq(SPMEM::read(0x08), 0, "SLTU result 0x08")?;
        soft_assert_eq(SPMEM::read(0x0C), 1, "SLTU result 0x0C")?;
        soft_assert_eq(SPMEM::read(0x10), 1, "SLTU result 0x10")?;
        soft_assert_eq(SPMEM::read(0x14), 1, "SLTU result 0x14")?;
        soft_assert_eq(SPMEM::read(0x18), 1, "SLTU result 0x18")?;
        soft_assert_eq(SPMEM::read(0x1C), 0, "SLTU result 0x1C")?;
        soft_assert_eq(SPMEM::read(0x20), 0, "SLTU result 0x20")?;
        soft_assert_eq(SPMEM::read(0x24), 0, "SLTU result 0x24")?;
        soft_assert_eq(SPMEM::read(0x28), 1, "SLTU result 0x28")?;

        soft_assert_eq(SPMEM::read(0x30), 0, "SLTU into R0 must be ignored")?;

        Ok(())
    }
}
