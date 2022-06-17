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

fn test<F: Fn(&mut RSPAssembler, GPR, GPR)>(value: u32, expected_value: u32, write_shift: F) -> Result<(), String> {
    let mut assembler = RSPAssembler::new(0);

    // Write source into one register and shift into another one
    assembler.write_li(GPR::A0, value);
    write_shift(&mut assembler, GPR::S1, GPR::A0);

    // Write into a register and shift it into itself
    assembler.write_li(GPR::S2, value);
    write_shift(&mut assembler, GPR::S2, GPR::S2);

    // Shift from R0
    write_shift(&mut assembler, GPR::S3, GPR::R0);

    // Write results
    assembler.write_sw(GPR::S1, GPR::R0, 0x0);
    assembler.write_sw(GPR::S2, GPR::R0, 0x4);
    assembler.write_sw(GPR::S3, GPR::R0, 0x8);

    // Shift into R0
    assembler.write_li(GPR::V0, 0);
    write_shift(&mut assembler, GPR::R0, GPR::S3);
    assembler.write_sw(GPR::R0, GPR::V0, 0xC);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq(SPMEM::read(0x0), expected_value, "Shift result")?;
    soft_assert_eq(SPMEM::read(0x4), expected_value, "Shift result")?;
    soft_assert_eq(SPMEM::read(0x8), 0, "Any shift using R0 as source reg must result in 0")?;
    soft_assert_eq(SPMEM::read(0xC), 0, "Shift into R0 must not change R0")?;

    Ok(())
}

pub struct SLL {}

impl Test for SLL {
    fn name(&self) -> &str { "RSP SLL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // source value, shift amount, expected value
            Box::new((0u32, 5u32, 0u32)),
            Box::new((0x01u32, 5u32, 0x20u32)),
            Box::new((0xFFFFFFFFu32, 8u32, 0xFFFFFF00u32)),
            Box::new((0xFFFF88FFu32, 0u32, 0xFFFF88FFu32)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u32, u32, u32)>() {
            Some((source_value, shift_amount, expected_value)) => {
                test(*source_value, *expected_value, |assembler, target_reg, source_reg| {
                    assembler.write_sll(target_reg, source_reg, *shift_amount)
                })
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct SLLV {}

impl Test for SLLV {
    fn name(&self) -> &str { "RSP SLLV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // source value, shift amount, expected value
            Box::new((0u32, 5u32, 0u32)),
            Box::new((0x01u32, 5u32, 0x20u32)),
            Box::new((0xFFFFFFFFu32, 8u32, 0xFFFFFF00u32)),
            Box::new((0xFFFF88FFu32, 0u32, 0xFFFF88FFu32)),
            Box::new((0xFFFFFFFFu32, 250u32, 0xFC000000u32)),
            Box::new((0x00010010u32, 8897643u32, 0x08008000u32)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u32, u32, u32)>() {
            Some((source_value, shift_amount, expected_value)) => {
                test(*source_value, *expected_value, |assembler, target_reg, source_reg| {
                    assembler.write_li(GPR::AT, *shift_amount);
                    assembler.write_sllv(target_reg, source_reg, GPR::AT)
                })
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct SLLVWithShiftAmountOverwrite {}

impl Test for SLLVWithShiftAmountOverwrite {
    fn name(&self) -> &str { "RSP SLLV (where target=shift amount)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // When recompiling to x86/x64, this shift might a special case: X := Y << X
        // That's because Y would be copied into X before the shift, but then the shift amount is gone
        test(5, 0b100000, |assembler, target_reg, source_reg| {
            assembler.write_li(GPR::AT, 1);
            if source_reg == GPR::R0 {
                // The source==R0 is a special case in the test above. It doesn't matter for this test
                assembler.write_li(target_reg, 0)
            } else {
                assembler.write_sllv(target_reg, GPR::AT, source_reg)
            }
        })
    }
}

pub struct SRL {}

impl Test for SRL {
    fn name(&self) -> &str { "RSP SRL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // source value, shift amount, expected value
            Box::new((0u32, 5u32, 0u32)),
            Box::new((0x20u32, 4u32, 0x2u32)),
            Box::new((0xFFFFFFFFu32, 8u32, 0x00FFFFFFu32)),
            Box::new((0x80000000u32, 31u32, 0x00000001u32)),
            Box::new((0xFFFF88FFu32, 0u32, 0xFFFF88FFu32)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u32, u32, u32)>() {
            Some((source_value, shift_amount, expected_value)) => {
                test(*source_value, *expected_value, |assembler, target_reg, source_reg| {
                    assembler.write_srl(target_reg, source_reg, *shift_amount)
                })
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct SRLV {}

impl Test for SRLV {
    fn name(&self) -> &str { "RSP SRLV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // source value, shift amount, expected value
            Box::new((0u32, 5u32, 0u32)),
            Box::new((0x20u32, 4u32, 0x2u32)),
            Box::new((0xFFFFFFFFu32, 8u32, 0x00FFFFFFu32)),
            Box::new((0x80000000u32, 31u32, 0x00000001u32)),
            Box::new((0xFFFF88FFu32, 0u32, 0xFFFF88FFu32)),
            Box::new((0xFFFF88FFu32, 846u32, 0x0003FFFEu32)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u32, u32, u32)>() {
            Some((source_value, shift_amount, expected_value)) => {
                test(*source_value, *expected_value, |assembler, target_reg, source_reg| {
                    assembler.write_li(GPR::AT, *shift_amount);
                    assembler.write_srlv(target_reg, source_reg, GPR::AT)
                })
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct SRLVWithShiftAmountOverwrite {}

impl Test for SRLVWithShiftAmountOverwrite {
    fn name(&self) -> &str { "RSP SRLV (where target=shift amount)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // When recompiling to x86/x64, this shift might a special case: X := Y << X
        // That's because Y would be copied into X before the shift, but then the shift amount is gone
        test(5, 0x7FFF891, |assembler, target_reg, source_reg| {
            assembler.write_li(GPR::AT, 0xFFFF1234);
            if source_reg == GPR::R0 {
                // The source==R0 is a special case in the test above. It doesn't matter for this test
                assembler.write_li(target_reg, 0)
            } else {
                assembler.write_srlv(target_reg, GPR::AT, source_reg)
            }
        })
    }
}

pub struct SRA {}

impl Test for SRA {
    fn name(&self) -> &str { "RSP SRA" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // source value, shift amount, expected value
            Box::new((0u32, 5u32, 0u32)),
            Box::new((0x20u32, 4u32, 0x2u32)),
            Box::new((0xFFFFFFFFu32, 8u32, 0xFFFFFFFFu32)),
            Box::new((0x80000000u32, 31u32, 0xFFFFFFFFu32)),
            Box::new((0xFFFF88FFu32, 0u32, 0xFFFF88FFu32)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u32, u32, u32)>() {
            Some((source_value, shift_amount, expected_value)) => {
                test(*source_value, *expected_value, |assembler, target_reg, source_reg| {
                    assembler.write_sra(target_reg, source_reg, *shift_amount)
                })
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct SRAV {}

impl Test for SRAV {
    fn name(&self) -> &str { "RSP SRAV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // source value, shift amount, expected value
            Box::new((0u32, 5u32, 0u32)),
            Box::new((0x20u32, 4u32, 0x2u32)),
            Box::new((0xFFFFFFFFu32, 8u32, 0xFFFFFFFFu32)),
            Box::new((0x80000000u32, 31u32, 0xFFFFFFFFu32)),
            Box::new((0xFFFF88FFu32, 0u32, 0xFFFF88FFu32)),
            Box::new((0xFFFF88FFu32, 846u32, 0xFFFFFFFEu32)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u32, u32, u32)>() {
            Some((source_value, shift_amount, expected_value)) => {
                test(*source_value, *expected_value, |assembler, target_reg, source_reg| {
                    assembler.write_li(GPR::AT, *shift_amount);
                    assembler.write_srav(target_reg, source_reg, GPR::AT)
                })
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct SRAVWithShiftAmountOverwrite {}

impl Test for SRAVWithShiftAmountOverwrite {
    fn name(&self) -> &str { "RSP SRAV (where target=shift amount)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // When recompiling to x86/x64, this shift might a special case: X := Y << X
        // That's because Y would be copied into X before the shift, but then the shift amount is gone
        test(5, 0xFFFFF891, |assembler, target_reg, source_reg| {
            assembler.write_li(GPR::AT, 0xFFFF1234);
            if source_reg == GPR::R0 {
                // The source==R0 is a special case in the test above. It doesn't matter for this test
                assembler.write_li(target_reg, 0)
            } else {
                assembler.write_srav(target_reg, GPR::AT, source_reg)
            }
        })
    }
}

