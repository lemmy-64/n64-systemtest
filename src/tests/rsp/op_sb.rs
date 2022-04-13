use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Various tests using SB

pub struct SB {}

impl Test for SB {
    fn name(&self) -> &str { "RSP SB" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        // Pre-fill data
        assembler.write_li(GPR::S0, 0xBADDECAF);
        assembler.write_sw(GPR::S0, GPR::R0, 0x000);
        assembler.write_sw(GPR::S0, GPR::R0, 0x004);
        assembler.write_sw(GPR::S0, GPR::R0, 0x008);
        assembler.write_sw(GPR::S0, GPR::R0, 0x00C);
        assembler.write_sw(GPR::S0, GPR::R0, 0x010);
        assembler.write_sw(GPR::S0, GPR::R0, 0x014);
        assembler.write_sw(GPR::S0, GPR::R0, 0xFFC);

        assembler.write_li(GPR::T0, 0xFFFFF00A);
        assembler.write_li(GPR::T1, 0x00000010);
        assembler.write_li(GPR::T2, 0x00001012);

        // 0x00: R0+0
        assembler.write_li(GPR::S0, 0x12345678);
        assembler.write_sb(GPR::S0, GPR::R0, 0x0);

        // 0x05: R0+0x05
        assembler.write_li(GPR::S0, 0x9ABCDEFF);
        assembler.write_sb(GPR::S0, GPR::R0, 0x05);

        // 0x0A: T0+0x0. Out of bounds, expecting address masking
        assembler.write_li(GPR::S0, 0xEEDDCCBB);
        assembler.write_sb(GPR::S0, GPR::T0, 0x00);

        // 0x0F: T0+0x05, out of bounds. Expecting address masking
        assembler.write_li(GPR::S0, 0xAA998877);
        assembler.write_sb(GPR::S0, GPR::T0, 0x1005);

        // 0x11: T2-0x1
        assembler.write_li(GPR::S0, 0x22110011);
        assembler.write_sb(GPR::S0, GPR::T2, -1);

        // 0xFFF
        assembler.write_li(GPR::S0, 0x43349821);
        assembler.write_sb(GPR::S0, GPR::R0, 0xFFF);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x00), 0x78DDECAF, "SB to DMEM[0x00]")?;
        soft_assert_eq(SPMEM::read(0x04), 0xBAFFECAF, "SB to DMEM[0x05]")?;
        soft_assert_eq(SPMEM::read(0x08), 0xBADDBBAF, "SB to DMEM[0x0A]")?;
        soft_assert_eq(SPMEM::read(0x0C), 0xBADDEC77, "SB to DMEM[0x0F]")?;
        soft_assert_eq(SPMEM::read(0x10), 0xBA11ECAF, "SB to DMEM[0x11]")?;
        soft_assert_eq(SPMEM::read(0xFFC), 0xBADDEC21, "SB to DMEM[0xFFF]")?;


        Ok(())
    }
}
