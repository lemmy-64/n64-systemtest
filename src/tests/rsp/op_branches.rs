use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

fn test<F: FnOnce(&mut RSPAssembler)>(value1: u32, value2: u32, expected_branch: bool, expected_ra_change: bool, write_branch: F) -> Result<(), String> {
    const START_OFFSET: usize = 0xFD8;
    let mut assembler = RSPAssembler::new(START_OFFSET);

    assembler.write_li(GPR::A0, value1);
    assembler.write_li(GPR::A1, value2);
    assembler.write_li(GPR::RA, 0);
    assembler.write_li(GPR::S1, 0);
    assembler.write_li(GPR::S2, 0);
    assembler.write_li(GPR::S3, 0);

    // Write NOPs so that the branch instruction is the last instruction (write_li above needs 1 or 2 instructions, so we have to be flexible)
    assert!(assembler.writer().offset() > START_OFFSET);
    while assembler.writer().offset() != 0xFFC {
        assembler.write_nop()
    }

    write_branch(&mut assembler);

    assembler.write_addiu(GPR::S1, GPR::S1, 1);
    assembler.write_addiu(GPR::S2, GPR::S2, 1);
    assembler.write_addiu(GPR::S3, GPR::S3, 1);

    // Write results
    assembler.write_sw(GPR::RA, GPR::R0, 0x0);
    assembler.write_sw(GPR::S1, GPR::R0, 0x4);
    assembler.write_sw(GPR::S2, GPR::R0, 0x8);
    assembler.write_sw(GPR::S3, GPR::R0, 0xC);

    assembler.write_break();

    RSP::run_and_wait(START_OFFSET);

    if expected_ra_change {
        soft_assert_eq(SPMEM::read(0x0), 4, "RA register")?;
    } else {
        soft_assert_eq(SPMEM::read(0x0), 0, "RA register")?;
    }
    soft_assert_eq(SPMEM::read(0x4), 1, "Delay slot was not executed")?;
    if expected_branch {
        soft_assert_eq(SPMEM::read(0x8), 0, "Branch wasn't taken but it should have been")?;
    } else {
        soft_assert_eq(SPMEM::read(0x8), 1, "Branch was taken but it shouldn't have been")?;
    }
    soft_assert_eq(SPMEM::read(0xC), 1, "Delay slot was not executed")?;
    soft_assert_eq(RSP::pc(), 0x20, "RSP PC after execution")?;

    Ok(())
}

pub struct BEQ {}

impl Test for BEQ {
    fn name(&self) -> &str { "RSP BEQ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0u32, 0u32)),
            Box::new((true, 1u32, 1u32)),
            Box::new((false, 1u32, 0u32)),
            Box::new((true, 0x7FFFFFFFu32, 0x7FFFFFFFu32)),
            Box::new((false, 0xFFFFFFFFu32, 0x7FFFFFFEu32)),
            Box::new((true, 1u32)),
            Box::new(true),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some((expected_branch, value1, value2)) =(*value).downcast_ref::<(bool, u32, u32)>() {
            test(*value1, *value2, *expected_branch, false, |assembler| {
                assembler.write_beq(GPR::A0, GPR::A1, 2)
            })
        } else if let Some((expected_branch, value)) =(*value).downcast_ref::<(bool, u32)>() {
            // Single value means we compare against itself. This is a useful test for recompilers that might want to optimize this
            test(*value, 0, *expected_branch, false,  |assembler| {
                assembler.write_beq(GPR::A0, GPR::A0, 2)
            })
        } else if let Some(expected_branch) =(*value).downcast_ref::<bool>() {
            // No value means we compare R0 against R0. This is a useful test for recompilers that might want to optimize this
            test(0, 0, *expected_branch, false, |assembler| {
                assembler.write_beq(GPR::R0, GPR::R0, 2)
            })
        } else {
            Err("Value is not valid".to_string())
        }
    }
}

pub struct BNE {}

impl Test for BNE {
    fn name(&self) -> &str { "RSP BNE" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, 0u32, 0u32)),
            Box::new((false, 1u32, 1u32)),
            Box::new((true, 1u32, 0u32)),
            Box::new((false, 0x7FFFFFFFu32, 0x7FFFFFFFu32)),
            Box::new((true, 0xFFFFFFFFu32, 0x7FFFFFFEu32)),
            Box::new((false, 1u32)),
            Box::new(false),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some((expected_branch, value1, value2)) =(*value).downcast_ref::<(bool, u32, u32)>() {
            test(*value1, *value2, *expected_branch, false, |assembler| {
                assembler.write_bne(GPR::A0, GPR::A1, 2)
            })
        } else if let Some((expected_branch, value)) =(*value).downcast_ref::<(bool, u32)>() {
            // Single value means we compare against itself. This is a useful test for recompilers that might want to optimize this
            test(*value, 0, *expected_branch, false, |assembler| {
                assembler.write_bne(GPR::A0, GPR::A0, 2)
            })
        } else if let Some(expected_branch) =(*value).downcast_ref::<bool>() {
            // No value means we compare R0 against R0. This is a useful test for recompilers that might want to optimize this
            test(0, 0, *expected_branch, false, |assembler| {
                assembler.write_bne(GPR::R0, GPR::R0, 2)
            })
        } else {
            Err("Value is not valid".to_string())
        }
    }
}

pub struct BLEZ {}

impl Test for BLEZ {
    fn name(&self) -> &str { "RSP BLEZ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0u32)),
            Box::new((false, 1u32)),
            Box::new((false, 0x7FFFFFFFu32)),
            Box::new((true, 0xFFFFFFFFu32)),
            Box::new((true, 0xFFFF0000u32)),
            Box::new(true), // Compares R0
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some((expected_branch, value)) =(*value).downcast_ref::<(bool, u32)>() {
            test(*value, 0, *expected_branch, false, |assembler| {
                assembler.write_blez(GPR::A0, 2)
            })
        } else if let Some(expected_branch) =(*value).downcast_ref::<bool>() {
            // No value means we compare R0 against R0. This is a useful test for recompilers that might want to optimize this
            test(0, 0, *expected_branch, false, |assembler| {
                assembler.write_blez(GPR::R0, 2)
            })
        } else {
            Err("Value is not valid".to_string())
        }
    }
}

pub struct BGTZ {}

impl Test for BGTZ {
    fn name(&self) -> &str { "RSP BGTZ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, 0u32)),
            Box::new((true, 1u32)),
            Box::new((true, 0x7FFFFFFFu32)),
            Box::new((false, 0xFFFFFFFFu32)),
            Box::new((false, 0xFFFF0000u32)),
            Box::new(false), // Compares R0
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some((expected_branch, value)) =(*value).downcast_ref::<(bool, u32)>() {
            test(*value, 0, *expected_branch, false, |assembler| {
                assembler.write_bgtz(GPR::A0, 2)
            })
        } else if let Some(expected_branch) =(*value).downcast_ref::<bool>() {
            test(0, 0, *expected_branch, false, |assembler| {
                assembler.write_bgtz(GPR::R0, 2)
            })
        } else {
            Err("Value is not valid".to_string())
        }
    }
}

pub struct BLTZ {}

impl Test for BLTZ {
    fn name(&self) -> &str { "RSP BLTZ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, 0u32)),
            Box::new((false, 1u32)),
            Box::new((false, 0x7FFFFFFFu32)),
            Box::new((true, 0xFFFFFFFFu32)),
            Box::new((true, 0xFFFF0000u32)),
            Box::new(false), // Compares R0
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some((expected_branch, value)) =(*value).downcast_ref::<(bool, u32)>() {
            test(*value, 0, *expected_branch, false, |assembler| {
                assembler.write_bltz(GPR::A0, 2)
            })
        } else if let Some(expected_branch) =(*value).downcast_ref::<bool>() {
            test(0, 0, *expected_branch, false, |assembler| {
                assembler.write_bltz(GPR::R0, 2)
            })
        } else {
            Err("Value is not valid".to_string())
        }
    }
}

pub struct BGEZ {}

impl Test for BGEZ {
    fn name(&self) -> &str { "RSP BGEZ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0u32)),
            Box::new((true, 1u32)),
            Box::new((true, 0x7FFFFFFFu32)),
            Box::new((false, 0xFFFFFFFFu32)),
            Box::new((false, 0xFFFF0000u32)),
            Box::new(true), // Compares R0
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some((expected_branch, value)) =(*value).downcast_ref::<(bool, u32)>() {
            test(*value, 0, *expected_branch, false, |assembler| {
                assembler.write_bgez(GPR::A0, 2)
            })
        } else if let Some(expected_branch) =(*value).downcast_ref::<bool>() {
            test(0, 0, *expected_branch, false, |assembler| {
                assembler.write_bgez(GPR::R0, 2)
            })
        } else {
            Err("Value is not valid".to_string())
        }
    }
}

pub struct BLTZAL {}

impl Test for BLTZAL {
    fn name(&self) -> &str { "RSP BLTZAL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, 0u32)),
            Box::new((false, 1u32)),
            Box::new((false, 0x7FFFFFFFu32)),
            Box::new((true, 0xFFFFFFFFu32)),
            Box::new((true, 0xFFFF0000u32)),
            Box::new(false), // Compares R0
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some((expected_branch, value)) =(*value).downcast_ref::<(bool, u32)>() {
            test(*value, 0, *expected_branch, true, |assembler| {
                assembler.write_bltzal(GPR::A0, 2)
            })
        } else if let Some(expected_branch) =(*value).downcast_ref::<bool>() {
            test(0, 0, *expected_branch, true, |assembler| {
                assembler.write_bltzal(GPR::R0, 2)
            })
        } else {
            Err("Value is not valid".to_string())
        }
    }
}

pub struct BGEZAL {}

impl Test for BGEZAL {
    fn name(&self) -> &str { "RSP BGEZAL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0u32)),
            Box::new((true, 1u32)),
            Box::new((true, 0x7FFFFFFFu32)),
            Box::new((false, 0xFFFFFFFFu32)),
            Box::new((false, 0xFFFF0000u32)),
            Box::new(true), // Compares R0
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        if let Some((expected_branch, value)) =(*value).downcast_ref::<(bool, u32)>() {
            test(*value, 0, *expected_branch, true, |assembler| {
                assembler.write_bgezal(GPR::A0, 2)
            })
        } else if let Some(expected_branch) =(*value).downcast_ref::<bool>() {
            test(0, 0, *expected_branch, true, |assembler| {
                assembler.write_bgezal(GPR::R0, 2)
            })
        } else {
            Err("Value is not valid".to_string())
        }
    }
}
