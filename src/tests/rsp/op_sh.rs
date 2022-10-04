use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Various tests using SH. Lessons learned:
// - SH works to any memory location, including unaligned.
// - When writing to 0xFFF, there is wrap-around to 0x0

pub struct SHAligned {}

impl Test for SHAligned {
    fn name(&self) -> &str { "RSP SH (aligned)" }

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

        assembler.write_li(GPR::T0, 0xFFFFF008);
        assembler.write_li(GPR::T1, 0x00000010);
        assembler.write_li(GPR::T2, 0x00001017);

        // 0x00: R0+0
        assembler.write_li(GPR::S0, 0x12345678);
        assembler.write_sh(GPR::S0, GPR::R0, 0x0);

        // 0x06: R0+0x06
        assembler.write_li(GPR::S0, 0x9ABCDEFF);
        assembler.write_sh(GPR::S0, GPR::R0, 0x06);

        // 0x08: T0+0x0. Out of bounds, expecting address masking
        assembler.write_li(GPR::S0, 0xEEDDCCBB);
        assembler.write_sh(GPR::S0, GPR::T0, 0x00);

        // 0x0E: T0+0x10, out of bounds. Expecting address masking
        assembler.write_li(GPR::S0, 0xAA998877);
        assembler.write_sh(GPR::S0, GPR::T0, 0x1006);

        // 0x10: T1+0x0 with offset out of bounds. Offset is 0 when masked
        assembler.write_li(GPR::S0, 0x66554433);
        assembler.write_sh(GPR::S0, GPR::T1, 0x1000);

        // 0x16: T2-0x1
        assembler.write_li(GPR::S0, 0x22110011);
        assembler.write_sh(GPR::S0, GPR::T2, -1);

        // 0xFFE
        assembler.write_li(GPR::S0, 0x43349821);
        assembler.write_sh(GPR::S0, GPR::R0, 0xFFE);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x00), 0x5678ECAF, "SH to DMEM[0x00]")?;
        soft_assert_eq(SPMEM::read(0x04), 0xBADDDEFF, "SH to DMEM[0x06]")?;
        soft_assert_eq(SPMEM::read(0x08), 0xCCBBECAF, "SH to DMEM[0x08]")?;
        soft_assert_eq(SPMEM::read(0x0C), 0xBADD8877, "SH to DMEM[0x0E]")?;
        soft_assert_eq(SPMEM::read(0x10), 0x4433ECAF, "SH to DMEM[0x10]")?;
        soft_assert_eq(SPMEM::read(0x14), 0xBADD0011, "SH to DMEM[0x16]")?;
        soft_assert_eq(SPMEM::read(0xFFC), 0xBADD9821, "SH to DMEM[0xFFE]")?;


        Ok(())
    }
}

pub struct SHUnaligned {}

impl Test for SHUnaligned {
    fn name(&self) -> &str { "RSP SH (unaligned)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        assembler.write_li(GPR::T0, 0x00000008);
        assembler.write_li(GPR::S0, 0xBADDECAF);
        assembler.write_li(GPR::S1, 0x12345678);
        assembler.write_li(GPR::S2, 0x91827364);

        assembler.write_sw(GPR::S0, GPR::R0, 0x000);
        assembler.write_sw(GPR::S0, GPR::R0, 0x004);
        assembler.write_sw(GPR::S0, GPR::R0, 0x008);
        assembler.write_sw(GPR::S0, GPR::R0, 0x00C);
        assembler.write_sw(GPR::S0, GPR::R0, 0xFFC);

        // 0x05: Unaligned (of by 1)
        assembler.write_sh(GPR::S1, GPR::R0, 0x5);

        // 0x0A: Unaligned (of by 1)
        assembler.write_sh(GPR::S1, GPR::T0, 0x3);

        // 0xFFF: Write at the very end. This is supposed to write two bytes at the end, then wrap over.
        // Some bits at the top that should be masked out
        assembler.write_sh(GPR::S2, GPR::R0, 0x7FFF);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x04), 0xBA5678AF, "SH to DMEM[0x05]")?;

        soft_assert_eq(SPMEM::read(0x08), 0xBADDEC56, "SH to DMEM[0x0A], lower u32")?;
        soft_assert_eq(SPMEM::read(0x0C), 0x78DDECAF, "SH to DMEM[0x0A], upper u32")?;

        soft_assert_eq(SPMEM::read(0xFFC), 0xBADDEC73, "SH to DMEM[0xFFF], lower u32")?;
        soft_assert_eq(SPMEM::read(0x00), 0x64DDECAF, "SH to DMEM[0xFFF], upper u32 (wrapped around to start of DMEM)")?;

        Ok(())
    }
}
