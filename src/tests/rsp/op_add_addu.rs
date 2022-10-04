use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

fn test<F: Fn(&mut RSPAssembler,GPR,GPR,GPR)>(write_add: F) -> Result<(), String> {
    let mut assembler = RSPAssembler::new(0);

    assembler.write_li(GPR::T0, 0x12345678);
    assembler.write_li(GPR::T1, 0xFFFFEDCB);
    assembler.write_li(GPR::T2, 0x00001234);

    write_add(&mut assembler, GPR::S0, GPR::T0, GPR::T1);
    write_add(&mut assembler, GPR::S1, GPR::T0, GPR::T2);

    assembler.write_li(GPR::S2, 5);
    write_add(&mut assembler, GPR::S2, GPR::S2, GPR::T1);

    assembler.write_li(GPR::S3, 8);
    write_add(&mut assembler, GPR::S3, GPR::T2, GPR::S3);

    assembler.write_li(GPR::S4, 20);
    write_add(&mut assembler, GPR::S4, GPR::S4, GPR::S4);

    assembler.write_li(GPR::S5, 20);
    write_add(&mut assembler, GPR::S5, GPR::S5, GPR::R0);

    assembler.write_li(GPR::S6, 40);
    write_add(&mut assembler, GPR::S6, GPR::R0, GPR::S6);

    write_add(&mut assembler, GPR::S7, GPR::R0, GPR::R0);

    // Write results
    assembler.write_sw(GPR::S0, GPR::R0, 0x0);
    assembler.write_sw(GPR::S1, GPR::R0, 0x4);
    assembler.write_sw(GPR::S2, GPR::R0, 0x8);
    assembler.write_sw(GPR::S3, GPR::R0, 0xC);
    assembler.write_sw(GPR::S4, GPR::R0, 0x10);
    assembler.write_sw(GPR::S5, GPR::R0, 0x14);
    assembler.write_sw(GPR::S6, GPR::R0, 0x18);
    assembler.write_sw(GPR::S7, GPR::R0, 0x1C);

    // Write to R0
    assembler.write_li(GPR::AT, 0);
    write_add(&mut assembler, GPR::R0, GPR::T0, GPR::T1);
    assembler.write_sw(GPR::R0, GPR::AT, 0x20);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq(SPMEM::read(0x00), 0x12344443, "Addition result 0x00")?;
    soft_assert_eq(SPMEM::read(0x04), 0x123468ac, "Addition result 0x04")?;
    soft_assert_eq(SPMEM::read(0x08), 0xffffedd0, "Addition result 0x08")?;
    soft_assert_eq(SPMEM::read(0x0C), 0x0000123c, "Addition result 0x0C")?;
    soft_assert_eq(SPMEM::read(0x10), 0x00000028, "Addition result 0x10")?;
    soft_assert_eq(SPMEM::read(0x14), 0x00000014, "Addition result 0x14")?;
    soft_assert_eq(SPMEM::read(0x18), 0x00000028, "Addition result 0x18")?;
    soft_assert_eq(SPMEM::read(0x1C), 0, "Addition result 0x1C")?;
    soft_assert_eq(SPMEM::read(0x20), 0, "Addition into R0 must be ignored")?;

    Ok(())
}

pub struct ADD {}

impl Test for ADD {
    fn name(&self) -> &str { "RSP ADD" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(|assembler, rd, rs, rt| {
            assembler.write_add(rd, rt, rs)
        })
    }
}

pub struct ADDU {}

impl Test for ADDU {
    fn name(&self) -> &str { "RSP ADDU" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(|assembler, rd, rs, rt| {
            assembler.write_addu(rd, rt, rs)
        })
    }
}
