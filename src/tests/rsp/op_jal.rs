use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct J {}

impl Test for J {
    fn name(&self) -> &str { "RSP J" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program at end of DMEM
        let mut assembler = RSPAssembler::new(0xFF4);

        // 0xFF4:
        assembler.write_ori(GPR::RA, GPR::R0, 0);

        // 0xFF8:
        assembler.write_ori(GPR::A0, GPR::R0, 1);

        // 0xFFC: Jump to 8 with a bunch of useless bits set
        assembler.write_j(0xFFFF008);

        // 0x000: Delay slot (after overflow)
        assembler.write_ori(GPR::A0, GPR::A0, 2);

        // 0x004: This is skipped
        assembler.write_ori(GPR::A0, GPR::A0, 4);

        // 0x008: Jump target
        assembler.write_ori(GPR::A0, GPR::A0, 8);

        // 0x00C: Save result
        assembler.write_sw(GPR::A0, GPR::R0, 0x0);
        assembler.write_sw(GPR::RA, GPR::R0, 0x4);

        // 0x010: Save result
        assembler.write_break();

        RSP::run_and_wait(0xFF4);

        soft_assert_eq(SPMEM::read(0x0), 11, "J is expected to handle delay slot in 0x000 and jump to 0x008, skipping 0x004")?;
        soft_assert_eq(SPMEM::read(0x4), 0, "J is not expected to change the RA register")?;

        Ok(())
    }
}
