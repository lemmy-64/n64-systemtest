use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct BEQ {}

impl Test for BEQ {
    fn name(&self) -> &str { "RSP BEQ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program at end of DMEM
        let mut assembler = RSPAssembler::new(0xFEC);

        assembler.write_ori(GPR::S0, GPR::R0, 0xFFFE);
        assembler.write_ori(GPR::S1, GPR::R0, 0xFFFE);
        assembler.write_ori(GPR::RA, GPR::R0, 0);
        assembler.write_ori(GPR::A0, GPR::R0, 0b000_000_000_1);

        // BEQ that is taken
        assembler.write_beq(GPR::S0, GPR::S1, 2);
        assembler.write_ori(GPR::A0, GPR::A0, 0b000_000_001_0); // 0x000: delay slot (located at 0x000 due to DMEM overflow)
        assembler.write_ori(GPR::A0, GPR::A0, 0b000_000_010_0); // 0x004: skipped
        assembler.write_ori(GPR::A0, GPR::A0, 0b000_000_100_0); // 0x008: branch target

        // BEQ that is not taken
        assembler.write_beq(GPR::S0, GPR::R0, 2);
        assembler.write_ori(GPR::A0, GPR::A0, 0b000_001_000_0);  // delay slot
        assembler.write_ori(GPR::A0, GPR::A0, 0b000_010_000_0);  // executed as branch isn't taken
        assembler.write_ori(GPR::A0, GPR::A0, 0b000_100_000_0);  // branch target

        // BEQ where a register is compared to itself (something that a recompiler might fast-path)
        assembler.write_beq(GPR::S0, GPR::S0, 2);
        assembler.write_ori(GPR::A0, GPR::A0, 0b001_000_000_0); // delay slot
        assembler.write_ori(GPR::A0, GPR::A0, 0b010_000_000_0); // skipped
        assembler.write_ori(GPR::A0, GPR::A0, 0b100_000_000_0); // branch target

        // Write results
        assembler.write_sw(GPR::A0, GPR::R0, 0x0);
        assembler.write_sw(GPR::RA, GPR::R0, 0x4);

        // 0x010: Save result
        assembler.write_break();

        RSP::run_and_wait(0xFEC);

        soft_assert_eq(SPMEM::read(0x0), 0b101_111_101_1, "Branches not taken as expected")?;
        soft_assert_eq(SPMEM::read(0x4), 0, "BEQ is not expected to change the RA register")?;

        Ok(())
    }
}
