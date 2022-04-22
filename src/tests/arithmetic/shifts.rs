use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

fn test_sra<const SHIFT_AMOUNT: u32>(source_value: u64) -> u64 {
    let mut result: u64 = 0;
    unsafe {
        asm!("
            .set noat
            .set noreorder

            LD $2, 0($3)
            SRA $2, $2, {SHIFT_AMOUNT}
            SD $2, 0($4)

        ", SHIFT_AMOUNT = const SHIFT_AMOUNT, out("$2") _, in("$3") &source_value, in("$4") &mut result)
    }
    result
}

fn test_srl<const SHIFT_AMOUNT: u32>(source_value: u64) -> u64 {
    let mut result: u64 = 0;
    unsafe {
        asm!("
            .set noat
            .set noreorder

            LD $2, 0($3)
            SRL $2, $2, {SHIFT_AMOUNT}
            SD $2, 0($4)

        ", SHIFT_AMOUNT = const SHIFT_AMOUNT, out("$2") _, in("$3") &source_value, in("$4") &mut result)
    }
    result
}

fn test_sll<const SHIFT_AMOUNT: u32>(source_value: u64) -> u64 {
    let mut result: u64 = 0;
    unsafe {
        asm!("
            .set noat
            .set noreorder

            LD $2, 0($3)
            SLL $2, $2, {SHIFT_AMOUNT}
            SD $2, 0($4)

        ", SHIFT_AMOUNT = const SHIFT_AMOUNT, out("$2") _, in("$3") &source_value, in("$4") &mut result)
    }
    result
}

pub struct SRA {}

impl Test for SRA {
    fn name(&self) -> &str { "SRA" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((0x00000000_12345678u64, 4u32, 0x00000000_01234567u64)),
            Box::new((0x00000000_82345678u64, 0u32, 0xFFFFFFFF_82345678u64)),

            Box::new((0x01234567_89ABCDEFu64, 4u32, 0x00000000_789ABCDEu64)),
            Box::new((0x00000008_789ABCDEu64, 4u32, 0xFFFFFFFF_8789ABCDu64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u64, u32, u64)>() {
            Some((source_value, shift_amount, expected_value)) => {
                let result = match shift_amount {
                    0 => test_sra::<0>(*source_value),
                    4 => test_sra::<4>(*source_value),
                    _ => panic!()
                };
                soft_assert_eq(result, *expected_value, "Result")?;
                Ok(())
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct SRL {}

impl Test for SRL {
    fn name(&self) -> &str { "SRL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((0x00000000_12345678u64, 4u32, 0x00000000_01234567u64)),
            Box::new((0x00000000_82345678u64, 0u32, 0xFFFFFFFF_82345678u64)),

            Box::new((0x01234567_89ABCDEFu64, 4u32, 0x00000000_089ABCDEu64)),
            Box::new((0x00000008_789ABCDEu64, 4u32, 0x00000000_0789ABCDu64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u64, u32, u64)>() {
            Some((source_value, shift_amount, expected_value)) => {
                let result = match shift_amount {
                    0 => test_srl::<0>(*source_value),
                    4 => test_srl::<4>(*source_value),
                    _ => panic!()
                };
                soft_assert_eq(result, *expected_value, "Result")?;
                Ok(())
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct SLL {}

impl Test for SLL {
    fn name(&self) -> &str { "SLL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((0x00000000_12345678u64, 4u32, 0x00000000_23456780u64)),
            Box::new((0x00000000_82345678u64, 0u32, 0xFFFFFFFF_82345678u64)),
            Box::new((0x12345678_789ABCDEu64, 4u32, 0xFFFFFFFF_89ABCDE0u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u64, u32, u64)>() {
            Some((source_value, shift_amount, expected_value)) => {
                let result = match shift_amount {
                    0 => test_sll::<0>(*source_value),
                    4 => test_sll::<4>(*source_value),
                    _ => panic!()
                };
                soft_assert_eq(result, *expected_value, "Result")?;
                Ok(())
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

pub struct ShiftsIntoR0 {}

impl Test for ShiftsIntoR0 {
    fn name(&self) -> &str { "ShiftsIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut sll: u64 = 0xDECAF15BADCAFE;
        let mut srl: u64 = 0xDECAF15BADCAFE;
        let mut sra: u64 = 0xDECAF15BADCAFE;

        unsafe {
            asm!("
                LUI $2, 0x1234
                SLL $0, $2, 1
                SD $0, 0($3)
                SRL $0, $2, 1
                SD $0, 0($4)
                SRA $0, $2, 1
                SD $0, 0($5)
            ", out("$2") _, in("$3") &mut sll, in("$4") &mut srl, in("$5") &mut sra)
        }

        soft_assert_eq(sll, 0, "SLL into R0")?;
        soft_assert_eq(srl, 0, "SRL into R0")?;
        soft_assert_eq(sra, 0, "SRA into R0")?;

        Ok(())
    }
}
