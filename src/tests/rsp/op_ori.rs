use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct ORI {}

impl Test for ORI {
    fn name(&self) -> &str { "RSP ORI" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        assembler.write_li(GPR::V0, 0xFFFFF00F);
        assembler.write_li(GPR::V1, 0x11111111);
        assembler.write_ori(GPR::S0, GPR::V0, 0x1234);
        assembler.write_ori(GPR::S1, GPR::V0, 0x0072);
        assembler.write_ori(GPR::S2, GPR::V0, 0x0000);
        assembler.write_ori(GPR::S3, GPR::V0, 0xFFFF);

        assembler.write_ori(GPR::S4, GPR::V1, 0x1234);
        assembler.write_ori(GPR::S5, GPR::V1, 0x0072);
        assembler.write_ori(GPR::S6, GPR::V1, 0x0000);
        assembler.write_ori(GPR::S7, GPR::V1, 0xFFFF);

        for (i, gpr) in (GPR::S0..=GPR::S7).enumerate() {
            assembler.write_sw(gpr, GPR::R0, (i * 4) as i16);
        }

        // into R0
        assembler.write_li(GPR::A0, 0);
        assembler.write_ori(GPR::R0, GPR::V1, 0xFFFF);
        assembler.write_sw(GPR::R0, GPR::A0, 0x100);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x00), 0xFFFFF23F, "0xFFFFFFFF | 0x1234")?;
        soft_assert_eq(SPMEM::read(0x04), 0xFFFFF07F, "0xFFFFFFFF | 0x0072")?;
        soft_assert_eq(SPMEM::read(0x08), 0xFFFFF00F, "0xFFFFFFFF | 0x0000")?;
        soft_assert_eq(SPMEM::read(0x0C), 0xFFFFFFFF, "0xFFFFFFFF | 0xFFFF")?;

        soft_assert_eq(SPMEM::read(0x10), 0x11111335, "0x11111111 | 0x1234")?;
        soft_assert_eq(SPMEM::read(0x14), 0x11111173, "0x11111111 | 0x0072")?;
        soft_assert_eq(SPMEM::read(0x18), 0x11111111, "0x11111111 | 0x0000")?;
        soft_assert_eq(SPMEM::read(0x1C), 0x1111FFFF, "0x11111111 | 0xFFFF")?;

        soft_assert_eq(SPMEM::read(0x100), 0, "R0 should never change")?;

        Ok(())
    }
}
