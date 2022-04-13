use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Various tests using SW. Lessons learned:
// - SW works to any memory location, including unaligned.
// - When writing to 0xFFF, 0xFFE or 0xFFD, there is wrap-around to 0x0

pub struct SWAligned {}

impl Test for SWAligned {
    fn name(&self) -> &str { "RSP SW (aligned)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        assembler.write_li(GPR::T0, 0xFFFFF008);
        assembler.write_li(GPR::T1, 0x00000010);
        assembler.write_li(GPR::T2, 0x00001016);

        // 0x00: R0+0
        assembler.write_li(GPR::S0, 0x12345678);
        assembler.write_sw(GPR::S0, GPR::R0, 0x0);

        // 0x04: R0+0x04
        assembler.write_li(GPR::S0, 0x9ABCDEFF);
        assembler.write_sw(GPR::S0, GPR::R0, 0x04);

        // 0x08: T0+0x0. Out of bounds, expecting address masking
        assembler.write_li(GPR::S0, 0xEEDDCCBB);
        assembler.write_sw(GPR::S0, GPR::T0, 0x00);

        // 0x0C: T0+0x04, out of bounds. Expecting address masking
        assembler.write_li(GPR::S0, 0xAA998877);
        assembler.write_sw(GPR::S0, GPR::T0, 0x1004);

        // 0x10: T1+0x0 with offset out of bounds. Offset is 0 when masked
        assembler.write_li(GPR::S0, 0x66554433);
        assembler.write_sw(GPR::S0, GPR::T1, 0x1000);

        // 0x14: T2-0x2
        assembler.write_li(GPR::S0, 0x22110011);
        assembler.write_sw(GPR::S0, GPR::T2, -2);

        // 0xFFC
        assembler.write_li(GPR::S0, 0x94678213);
        assembler.write_sw(GPR::S0, GPR::R0, 0x1FFC);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x00), 0x12345678, "SW to DMEM[0x00]")?;
        soft_assert_eq(SPMEM::read(0x04), 0x9ABCDEFF, "SW to DMEM[0x04]")?;
        soft_assert_eq(SPMEM::read(0x08), 0xEEDDCCBB, "SW to DMEM[0x08]")?;
        soft_assert_eq(SPMEM::read(0x0C), 0xAA998877, "SW to DMEM[0x0C]")?;
        soft_assert_eq(SPMEM::read(0x10), 0x66554433, "SW to DMEM[0x10]")?;
        soft_assert_eq(SPMEM::read(0x14), 0x22110011, "SW to DMEM[0x14]")?;
        soft_assert_eq(SPMEM::read(0xFFC), 0x94678213, "SW to DMEM[0xFFC]")?;


        Ok(())
    }
}

pub struct SWUnaligned {}

impl Test for SWUnaligned {
    fn name(&self) -> &str { "RSP SW (unaligned)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        assembler.write_li(GPR::T0, 0x00000008);
        assembler.write_li(GPR::S0, 0xBADDECAF);
        assembler.write_li(GPR::S1, 0x12345678);
        assembler.write_li(GPR::S2, 0x91827364);

        // Clear with 0xBADDECAF aligned
        for u in (0..=0x18).step_by(4) {
            assembler.write_sw(GPR::S0, GPR::R0, u);
        }
        assembler.write_sw(GPR::S0, GPR::R0, 0xFFC);

        // 0x05: Unaligned (of by 1)
        assembler.write_sw(GPR::S1, GPR::R0, 0x5);

        // 0x0E: Unaligned (of by 2)
        assembler.write_sw(GPR::S1, GPR::R0, 0xE);

        // 0x0F: Unaligned (of by 3)
        assembler.write_sw(GPR::S1, GPR::R0, 0x17);

        // 0xFFE: Write at the very end. This is supposed to write two bytes at the end, then wrap over.
        // Some bits at the top that should be masked out
        assembler.write_sw(GPR::S2, GPR::R0, 0x7FFE);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x04), 0xBA123456, "SW to DMEM[0x05], lower u32")?;
        soft_assert_eq(SPMEM::read(0x08), 0x78DDECAF, "SW to DMEM[0x08], upper u32")?;

        soft_assert_eq(SPMEM::read(0x0C), 0xBADD1234, "SW to DMEM[0x0E], lower u32")?;
        soft_assert_eq(SPMEM::read(0x10), 0x5678ECAF, "SW to DMEM[0x0E], upper u32")?;

        soft_assert_eq(SPMEM::read(0x14), 0xBADDEC12, "SW to DMEM[0x17], lower u32")?;
        soft_assert_eq(SPMEM::read(0x18), 0x345678AF, "SW to DMEM[0x17], upper u32")?;

        soft_assert_eq(SPMEM::read(0xFFC), 0xBADD9182, "SW to DMEM[0xFFE], lower u32")?;
        soft_assert_eq(SPMEM::read(0x00), 0x7364ECAF, "SW to DMEM[0xFFE], upper u32 (wrapped around to start of DMEM)")?;

        Ok(())
    }
}
