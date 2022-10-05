use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

fn test_div(dividend: u64, divisor: u64) -> (u64, u64) {
    let mut quotient: u64 = 0;
    let mut remainder: u64 = 0;
    unsafe {
        asm!("
            .set noat
            .set noreorder

            LD $2, 0($4)
            LD $3, 0($5)
            DIV $0, $2, $3
            MFLO $2
            MFHI $3
            SD $2, 0($6)
            SD $3, 0($7)

        ", out("$2") _, out("$3") _, in("$4") &dividend, in("$5") &divisor, in("$6") &mut quotient, in("$7") &mut remainder)
    }
    (quotient, remainder)
}

fn test_divu(dividend: u64, divisor: u64) -> (u64, u64) {
    let mut quotient: u64 = 0;
    let mut remainder: u64 = 0;
    unsafe {
        asm!("
            .set noat
            .set noreorder

            LD $2, 0($4)
            LD $3, 0($5)
            DIVU $0, $2, $3
            MFLO $2
            MFHI $3
            SD $2, 0($6)
            SD $3, 0($7)

        ", out("$2") _, out("$3") _, in("$4") &dividend, in("$5") &divisor, in("$6") &mut quotient, in("$7") &mut remainder)
    }
    (quotient, remainder)
}

fn test_ddiv(dividend: u64, divisor: u64) -> (u64, u64) {
    let mut quotient: u64 = 0;
    let mut remainder: u64 = 0;
    unsafe {
        asm!("
            .set noat
            .set noreorder

            LD $2, 0($4)
            LD $3, 0($5)
            DDIV $0, $2, $3
            MFLO $2
            MFHI $3
            SD $2, 0($6)
            SD $3, 0($7)

        ", out("$2") _, out("$3") _, in("$4") &dividend, in("$5") &divisor, in("$6") &mut quotient, in("$7") &mut remainder)
    }
    (quotient, remainder)
}

fn test_ddivu(dividend: u64, divisor: u64) -> (u64, u64) {
    let mut quotient: u64 = 0;
    let mut remainder: u64 = 0;
    unsafe {
        asm!("
            .set noat
            .set noreorder

            LD $2, 0($4)
            LD $3, 0($5)
            DDIVU $0, $2, $3
            MFLO $2
            MFHI $3
            SD $2, 0($6)
            SD $3, 0($7)

        ", out("$2") _, out("$3") _, in("$4") &dividend, in("$5") &divisor, in("$6") &mut quotient, in("$7") &mut remainder)
    }
    (quotient, remainder)
}


pub struct DIV {}

impl Test for DIV {
    fn name(&self) -> &str { "DIV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((0x00000000_01234567u64, 0u64, 0xFFFFFFFF_FFFFFFFFu64, 0x00000000_01234567u64)),
            Box::new((0u64, 0u64, 0xFFFFFFFF_FFFFFFFFu64, 0u64)),
            Box::new((0xFFFFFFFF_F1234567u64, 0u64, 0x00000000_00000001u64, 0xFFFFFFFF_F1234567u64)),
            Box::new((0xFFFFFFFF_80000000u64, 0xFFFFFFFF_FFFFFFFFu64, 0xFFFFFFFF_80000000u64, 0u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u64, u64, u64, u64)>() {
            Some((dividend, divisor, expected_quotient, expected_remainder)) => {
                let (quotient, remainder) = test_div(*dividend, *divisor);
                soft_assert_eq(quotient, *expected_quotient, "Quotient")?;
                soft_assert_eq(remainder, *expected_remainder, "Remainder")?;
                Ok(())
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}


pub struct DIVU {}

impl Test for DIVU {
    fn name(&self) -> &str { "DIVU" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((0x00000000_01234567u64, 0u64, 0xFFFFFFFF_FFFFFFFFu64, 0x00000000_01234567u64)),
            Box::new((0u64, 0u64, 0xFFFFFFFF_FFFFFFFFu64, 0u64)),
            Box::new((0xFFFFFFFF_F1234567u64, 0u64, 0xFFFFFFFF_FFFFFFFFu64, 0xFFFFFFFF_F1234567u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u64, u64, u64, u64)>() {
            Some((dividend, divisor, expected_quotient, expected_remainder)) => {
                let (quotient, remainder) = test_divu(*dividend, *divisor);
                soft_assert_eq(quotient, *expected_quotient, "Quotient")?;
                soft_assert_eq(remainder, *expected_remainder, "Remainder")?;
                Ok(())
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct DDIV {}

impl Test for DDIV {
    fn name(&self) -> &str { "DDIV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((0x01234567_89ABCDEFu64, 0u64, 0xFFFFFFFF_FFFFFFFFu64, 0x01234567_89ABCDEFu64)),
            Box::new((0u64, 0u64, 0xFFFFFFFF_FFFFFFFFu64, 0u64)),
            Box::new((0xF1234567_89ABCDEFu64, 0u64, 0x00000000_00000001u64, 0xF1234567_89ABCDEFu64)),
            Box::new((0x80000000_00000000u64, 0xFFFFFFFF_FFFFFFFFu64, 0x80000000_00000000u64, 0u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u64, u64, u64, u64)>() {
            Some((dividend, divisor, expected_quotient, expected_remainder)) => {
                let (quotient, remainder) = test_ddiv(*dividend, *divisor);
                soft_assert_eq(quotient, *expected_quotient, "Quotient")?;
                soft_assert_eq(remainder, *expected_remainder, "Remainder")?;
                Ok(())
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct DDIVU {}

impl Test for DDIVU {
    fn name(&self) -> &str { "DDIVU" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((0x01234567_89ABCDEFu64, 0u64, 0xFFFFFFFF_FFFFFFFFu64, 0x01234567_89ABCDEFu64)),
            Box::new((0u64, 0u64, 0xFFFFFFFF_FFFFFFFFu64, 0u64)),
            Box::new((0xF1234567_89ABCDEFu64, 0u64, 0xFFFFFFFF_FFFFFFFFu64, 0xF1234567_89ABCDEFu64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u64, u64, u64, u64)>() {
            Some((dividend, divisor, expected_quotient, expected_remainder)) => {
                let (quotient, remainder) = test_ddivu(*dividend, *divisor);
                soft_assert_eq(quotient, *expected_quotient, "Quotient")?;
                soft_assert_eq(remainder, *expected_remainder, "Remainder")?;
                Ok(())
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

