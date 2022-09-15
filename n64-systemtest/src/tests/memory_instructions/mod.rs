use alloc::boxed::Box;
use alloc::string::{String};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use crate::cop0;
use cop0::clear_llbit;

use crate::tests::{Level, Test, soft_asserts};
use soft_asserts::soft_assert_eq;

pub struct LL {}

impl Test for LL {
    fn name(&self) -> &str { "LL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        clear_llbit();

        let base: u32 = 0xDEADCAFE;
        let mut result: u32 = 0;
        let mut llbit: u32 = 0;

        unsafe {
            asm!(r"
                .set noat
                .set noreorder

                LL $2, 0($3)
                SC $2, 0($4)
                SW $2, 0($5)
            ", out("$2") _, in("$3") &base, in("$4") &mut result, in("$5") &mut llbit);
        }

        let expected_lladdr = (&base as *const u32 as u64 & 0x1fffffff) >> 4;

        soft_assert_eq(cop0::lladdr(), expected_lladdr, "LLAddr")?;
        soft_assert_eq(result, base, "Result")?;
        soft_assert_eq(llbit, 1, "LLBit")?;
        Ok(())
    }
}

pub struct LLIntoR0 {}

impl Test for LLIntoR0 {
    fn name(&self) -> &str { "LLIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        clear_llbit();

        let base: u32 = 0xDEADCAFE;
        let mut result: u32 = 0;
        let mut llbit: u32 = 0;

        unsafe {
            asm!(r"
                .set noat
                .set noreorder

                LL $0, 0($3)
                SC $2, 0($4)
                SW $2, 0($5)
            ", out("$2") _, in("$3") &base, in("$4") &mut result, in("$5") &mut llbit);
        }

        let expected_lladdr = (&base as *const u32 as u64 & 0x1fffffff) >> 4;

        soft_assert_eq(cop0::lladdr(), expected_lladdr, "LLAddr")?;
        soft_assert_eq(llbit, 1, "LLBit")?;
        Ok(())
    }
}

pub struct LLD {}

impl Test for LLD {
    fn name(&self) -> &str { "LLD" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        clear_llbit();

        let base: u64 = 0xDEADCAFEBABEBEEF;
        let mut result: u64 = 0;
        let mut llbit: u32 = 0;

        unsafe {
            asm!(r"
                .set noat
                .set noreorder

                LLD $2, 0($3)
                SCD $2, 0($4)
                SW $2, 0($5)
            ", out("$2") _, in("$3") &base, in("$4") &mut result, in("$5") &mut llbit);
        }

        let expected_lladdr = (&base as *const u64 as u64 & 0x1fffffff) >> 4;

        soft_assert_eq(cop0::lladdr(), expected_lladdr, "LLAddr")?;
        soft_assert_eq(result, base, "Result")?;
        soft_assert_eq(llbit, 1, "LLBit")?;
        Ok(())
    }
}

pub struct LLDIntoR0 {}

impl Test for LLDIntoR0 {
    fn name(&self) -> &str { "LLDIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        clear_llbit();

        let base: u64 = 0xDEADCAFEBABEBEEF;
        let mut result: u64 = 0;
        let mut llbit: u32 = 0;

        unsafe {
            asm!(r"
                .set noat
                .set noreorder

                LLD $0, 0($3)
                SCD $2, 0($4)
                SW $2, 0($5)
            ", out("$2") _, in("$3") &base, in("$4") &mut result, in("$5") &mut llbit);
        }

        let expected_lladdr = (&base as *const u64 as u64 & 0x1fffffff) >> 4;

        soft_assert_eq(cop0::lladdr(), expected_lladdr, "LLAddr")?;
        soft_assert_eq(llbit, 1, "LLBit")?;
        Ok(())
    }
}

pub struct SC {}

impl Test for SC {
    fn name(&self) -> &str { "SC" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        clear_llbit();

        let base: u32 = 0xDEADCAFE;
        let mut result: u32 = 0;
        let mut llbit: u32 = 0;

        unsafe {
            asm!(r"
                .set noat
                .set noreorder

                LL $2, 0($3)
                SC $2, 0($4)
                SW $2, 0($5)
            ", out("$2") _, in("$3") &base, in("$4") &mut result, in("$5") &mut llbit);
        }

        soft_assert_eq(result, base, "Result")?;
        soft_assert_eq(llbit, 1, "LLBit")?;
        Ok(())
    }
}

pub struct SCWithoutLL {}

impl Test for SCWithoutLL {
    fn name(&self) -> &str { "SCWithoutLL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        clear_llbit();

        let base: u32 = 0x78934571;
        let mut result: u32 = 0x89734251;
        let mut llbit: u32 = 1;

        unsafe {
            asm!(r"
                .set noat
                .set noreorder

                LW $2, 0($3)
                SC $2, 0($4)
                SW $2, 0($5)
            ", out("$2") _, in("$3") &base, in("$4") &mut result, in("$5") &mut llbit);
        }

        soft_assert_eq(result, 0x89734251, "Result")?;
        soft_assert_eq(llbit, 0, "LLBit")?;
        Ok(())
    }
}

pub struct SCD {}

impl Test for SCD {
    fn name(&self) -> &str { "SCD" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        clear_llbit();

        let base: u64 = 0xDEADCAFEBABEBEEF;
        let mut result: u64 = 0;
        let mut llbit: u32 = 0;

        unsafe {
            asm!(r"
                .set noat
                .set noreorder

                LLD $2, 0($3)
                SCD $2, 0($4)
                SW $2, 0($5)
            ", out("$2") _, in("$3") &base, in("$4") &mut result, in("$5") &mut llbit);
        }

        soft_assert_eq(result, base, "Result")?;
        soft_assert_eq(llbit, 1, "LLBit")?;
        Ok(())
    }
}

pub struct SCDWithoutLL {}

impl Test for SCDWithoutLL {
    fn name(&self) -> &str { "SCDWithoutLL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        clear_llbit();

        let base: u64 = 0x8758182919289192;
        let mut result: u64 = 0x6789321045671234;
        let mut llbit: u32 = 1;

        unsafe {
            asm!(r"
                .set noat
                .set noreorder

                LD $2, 0($3)
                SCD $2, 0($4)
                SW $2, 0($5)
            ", out("$2") _, in("$3") &base, in("$4") &mut result, in("$5") &mut llbit);
        }

        soft_assert_eq(result, 0x6789321045671234, "Result")?;
        soft_assert_eq(llbit, 0, "LLBit")?;
        Ok(())
    }
}