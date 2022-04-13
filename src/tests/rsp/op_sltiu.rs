use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// SLTIU sign extends the immediate to 32 bit but then does an unsigned comparison
// Essentially, the instruction correctly compares positive with positive and negative with negative,
// but treats all positive numbers as smaller than negative numbers

pub struct SLTIU {}

impl Test for SLTIU {
    fn name(&self) -> &str { "RSP SLTIU" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        assembler.write_li(GPR::V0, 0x00000000);
        assembler.write_li(GPR::V1, 0x00000010);
        assembler.write_li(GPR::A0, 0xFFFFFFF0);

        assembler.write_sltiu(GPR::S0, GPR::V0, 0x0000u16 as i16);
        assembler.write_sltiu(GPR::S1, GPR::V0, 0x0001u16 as i16);
        assembler.write_sltiu(GPR::S2, GPR::V0, 0xFFFFu16 as i16);

        assembler.write_sltiu(GPR::S3, GPR::V1, 0x0010u16 as i16);
        assembler.write_sltiu(GPR::S4, GPR::V1, 0x0011u16 as i16);
        assembler.write_sltiu(GPR::S5, GPR::V1, 0x0009u16 as i16);
        assembler.write_sltiu(GPR::S6, GPR::V1, 0xFFFEu16 as i16);

        assembler.write_sltiu(GPR::S7, GPR::A0, 0xFFF0u16 as i16);
        assembler.write_sltiu(GPR::T8, GPR::A0, 0xFFF1u16 as i16);
        assembler.write_sltiu(GPR::T9, GPR::A0, 0xFFEFu16 as i16);
        assembler.write_sltiu(GPR::K0, GPR::A0, 0x0000u16 as i16);
        assembler.write_sltiu(GPR::K1, GPR::A0, 0x0010u16 as i16);

        for (i, gpr) in (GPR::S0..=GPR::K1).enumerate() {
            assembler.write_sw(gpr, GPR::R0, (i * 4) as i16);
        }

        // into R0
        assembler.write_li(GPR::A0, 0);
        assembler.write_sltiu(GPR::R0, GPR::V0, 1);
        assembler.write_sw(GPR::R0, GPR::A0, 0x100);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x00), 0, "0x00000000 < 0x0000")?;
        soft_assert_eq(SPMEM::read(0x04), 1, "0x00000000 < 0x0001")?;
        soft_assert_eq(SPMEM::read(0x08), 1, "0x00000000 < 0xFFFF")?;

        soft_assert_eq(SPMEM::read(0x0C), 0, "0x00000010 < 0x0010")?;
        soft_assert_eq(SPMEM::read(0x10), 1, "0x00000010 < 0x0011")?;
        soft_assert_eq(SPMEM::read(0x14), 0, "0x00000010 < 0x0009")?;
        soft_assert_eq(SPMEM::read(0x18), 1, "0x00000010 < 0xFFFE")?;

        soft_assert_eq(SPMEM::read(0x1C), 0, "0xFFFFFFF0 < 0xFFF0")?;
        soft_assert_eq(SPMEM::read(0x20), 1, "0xFFFFFFF0 < 0xFFF1")?;
        soft_assert_eq(SPMEM::read(0x24), 0, "0xFFFFFFF0 < 0xFFEF")?;
        soft_assert_eq(SPMEM::read(0x28), 0, "0xFFFFFFF0 < 0x0000")?;
        soft_assert_eq(SPMEM::read(0x2C), 0, "0xFFFFFFF0 < 0x0010")?;

        soft_assert_eq(SPMEM::read(0x100), 0, "R0 should never change")?;

        Ok(())
    }
}
