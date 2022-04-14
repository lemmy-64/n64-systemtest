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
        assembler.write_ori(GPR::A0, GPR::R0, 0b00000001);
        assembler.write_beq(GPR::S0, GPR::S1, 2);

        // Overflow to 0x000. This is the delay slot
        assembler.write_ori(GPR::A0, GPR::A0, 0b00000010);

        // 0x004: This is skipped
        assembler.write_ori(GPR::A0, GPR::A0, 0b00000100);

        // 0x008: Jump target
        assembler.write_ori(GPR::A0, GPR::A0, 0b00001000);

        // Do another BEQ, this time it will not be taken
        assembler.write_beq(GPR::S0, GPR::R0, 2);

        // Delay slot
        assembler.write_ori(GPR::A0, GPR::A0, 0b00010000);

        // This would have been skipped if the registers were equal
        assembler.write_ori(GPR::A0, GPR::A0, 0b00100000);

        // Jump target 2
        assembler.write_ori(GPR::A0, GPR::A0, 0b01000000);

        // 0x00C: Save result
        assembler.write_sw(GPR::A0, GPR::R0, 0x0);
        assembler.write_sw(GPR::RA, GPR::R0, 0x4);

        // 0x010: Save result
        assembler.write_break();

        RSP::run_and_wait(0xFEC);

        soft_assert_eq(SPMEM::read(0x0), 0b01111011, "Branches not taken as expected")?;
        soft_assert_eq(SPMEM::read(0x4), 0, "BEQ is not expected to change the RA register")?;

        Ok(())
    }
}
