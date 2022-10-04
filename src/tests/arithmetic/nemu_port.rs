use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct LUIOpcodeTest1 {}

impl Test for LUIOpcodeTest1 {
    fn name(&self) -> &str { "LUIOpcodeTest1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r2: u64 = 0x1234567898765432;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LUI $2, 0x891
                SD $2, 0($18)
            ", out("$2") _, in("$18") &mut r2);
        }
        soft_assert_eq(r2, 0x8910000, "Register $2")?;
        Ok(())
    }
}

pub struct LUIOpcodeTest2 {}

impl Test for LUIOpcodeTest2 {
    fn name(&self) -> &str { "LUIOpcodeTest2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r2: u64 = 0x1234567898765432;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LUI $2, 0xf891
                SD $2, 0($18)
            ", out("$2") _, in("$18") &mut r2);
        }
        soft_assert_eq(r2, 0xfffffffff8910000, "Register $2")?;
        Ok(())
    }
}

pub struct LUIOpcodeTestIntoR0 {}

impl Test for LUIOpcodeTestIntoR0 {
    fn name(&self) -> &str { "LUIOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LUI $0, 0xf891
                SD $0, 0($16)
            ", in("$16") &mut r0);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        Ok(())
    }
}

pub struct DADDIOpcodeTest {}

impl Test for DADDIOpcodeTest {
    fn name(&self) -> &str { "DADDIOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDI $3, $2, 4660
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1234567898761344, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIOpcodeTestWithNegativeImmediate {}

impl Test for DADDIOpcodeTestWithNegativeImmediate {
    fn name(&self) -> &str { "DADDIOpcodeTestWithNegativeImmediate" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDI $3, $2, -3532
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x123456789875f344, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIOpcodeTestLargeNegative1 {}

impl Test for DADDIOpcodeTestLargeNegative1 {
    fn name(&self) -> &str { "DADDIOpcodeTestLargeNegative1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $3, 0($19)
                DADDI $4, $3, -10208
                SD $4, 0($20)
            ", out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x4321432143211b41, "Register $4")?;
        Ok(())
    }
}

pub struct DADDIOpcodeTestLargeNegative2 {}

impl Test for DADDIOpcodeTestLargeNegative2 {
    fn name(&self) -> &str { "DADDIOpcodeTestLargeNegative2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $3, 0($19)
                DADDI $3, $3, -10208
                SD $3, 0($19)
            ", out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x4321432143211b41, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIOpcodeTestWithR0 {}

impl Test for DADDIOpcodeTestWithR0 {
    fn name(&self) -> &str { "DADDIOpcodeTestWithR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDI $3, $0, 4660
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1234, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIOpcodeTestWithOffsetZero {}

impl Test for DADDIOpcodeTestWithOffsetZero {
    fn name(&self) -> &str { "DADDIOpcodeTestWithOffsetZero" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDI $3, $2, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1234567898760110, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIOpcodeTestWithOffsetZeroAndR0 {}

impl Test for DADDIOpcodeTestWithOffsetZeroAndR0 {
    fn name(&self) -> &str { "DADDIOpcodeTestWithOffsetZeroAndR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDI $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIOpcodeTestIntoR0 {}

impl Test for DADDIOpcodeTestIntoR0 {
    fn name(&self) -> &str { "DADDIOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDI $0, $2, 4660
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIOpcodeTestIntoR0WithOffset0 {}

impl Test for DADDIOpcodeTestIntoR0WithOffset0 {
    fn name(&self) -> &str { "DADDIOpcodeTestIntoR0WithOffset0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDI $0, $2, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIUOpcodeTest {}

impl Test for DADDIUOpcodeTest {
    fn name(&self) -> &str { "DADDIUOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDIU $3, $2, 4660
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1234567898761344, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIUOpcodeTestWithNegativeImmediate {}

impl Test for DADDIUOpcodeTestWithNegativeImmediate {
    fn name(&self) -> &str { "DADDIUOpcodeTestWithNegativeImmediate" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDIU $3, $2, -3532
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x123456789875f344, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIUOpcodeTestLargeNegative1 {}

impl Test for DADDIUOpcodeTestLargeNegative1 {
    fn name(&self) -> &str { "DADDIUOpcodeTestLargeNegative1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $3, 0($19)
                DADDIU $4, $3, -10208
                SD $4, 0($20)
            ", out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x4321432143211b41, "Register $4")?;
        Ok(())
    }
}

pub struct DADDIUOpcodeTestLargeNegative2 {}

impl Test for DADDIUOpcodeTestLargeNegative2 {
    fn name(&self) -> &str { "DADDIUOpcodeTestLargeNegative2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $3, 0($19)
                DADDIU $3, $3, -10208
                SD $3, 0($19)
            ", out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x4321432143211b41, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIUOpcodeTestWithR0 {}

impl Test for DADDIUOpcodeTestWithR0 {
    fn name(&self) -> &str { "DADDIUOpcodeTestWithR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDIU $3, $0, 4660
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1234, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIUOpcodeTestWithOffsetZero {}

impl Test for DADDIUOpcodeTestWithOffsetZero {
    fn name(&self) -> &str { "DADDIUOpcodeTestWithOffsetZero" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDIU $3, $2, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1234567898760110, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIUOpcodeTestWithOffsetZeroAndR0 {}

impl Test for DADDIUOpcodeTestWithOffsetZeroAndR0 {
    fn name(&self) -> &str { "DADDIUOpcodeTestWithOffsetZeroAndR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDIU $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIUOpcodeTestIntoR0 {}

impl Test for DADDIUOpcodeTestIntoR0 {
    fn name(&self) -> &str { "DADDIUOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDIU $0, $2, 4660
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIUOpcodeTestNoOverflowPositive {}

impl Test for DADDIUOpcodeTestNoOverflowPositive {
    fn name(&self) -> &str { "DADDIUOpcodeTestNoOverflowPositive" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r2: u64 = 0x7fffffffffffff00;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDIU $3, $2, 256
                SD $2, 0($18)
                SD $3, 0($19)
            ", out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r2, 0x7fffffffffffff00, "Register $2")?;
        soft_assert_eq(r3, 0x8000000000000000, "Register $3")?;
        Ok(())
    }
}

pub struct DADDIUOpcodeTestNoOverflowNegative {}

impl Test for DADDIUOpcodeTestNoOverflowNegative {
    fn name(&self) -> &str { "DADDIUOpcodeTestNoOverflowNegative" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r2: u64 = 0x8000000000000000;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                DADDIU $3, $2, -1
                SD $2, 0($18)
                SD $3, 0($19)
            ", out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r2, 0x8000000000000000, "Register $2")?;
        soft_assert_eq(r3, 0x7fffffffffffffff, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTest {}

impl Test for ADDIOpcodeTest {
    fn name(&self) -> &str { "ADDIOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $3, $2, 4660
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0xffffffff98761344, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestWithNegativeImmediate {}

impl Test for ADDIOpcodeTestWithNegativeImmediate {
    fn name(&self) -> &str { "ADDIOpcodeTestWithNegativeImmediate" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $3, $2, -3532
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0xffffffff9875f344, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestLower12Bit {}

impl Test for ADDIOpcodeTestLower12Bit {
    fn name(&self) -> &str { "ADDIOpcodeTestLower12Bit" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x64;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $3, $2, 4000
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1004, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestUpper4Bit {}

impl Test for ADDIOpcodeTestUpper4Bit {
    fn name(&self) -> &str { "ADDIOpcodeTestUpper4Bit" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x64;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $3, $2, 12288
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x3064, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestLower12BitNegative {}

impl Test for ADDIOpcodeTestLower12BitNegative {
    fn name(&self) -> &str { "ADDIOpcodeTestLower12BitNegative" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x64;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $3, $2, -50
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x32, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestUpper4BitNegative {}

impl Test for ADDIOpcodeTestUpper4BitNegative {
    fn name(&self) -> &str { "ADDIOpcodeTestUpper4BitNegative" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x64;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $3, $2, -12288
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0xffffffffffffd064, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestLargeNegative1 {}

impl Test for ADDIOpcodeTestLargeNegative1 {
    fn name(&self) -> &str { "ADDIOpcodeTestLargeNegative1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $3, 0($19)
                ADDI $4, $3, -10208
                SD $4, 0($20)
            ", out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x43211b41, "Register $4")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestLargeNegative2 {}

impl Test for ADDIOpcodeTestLargeNegative2 {
    fn name(&self) -> &str { "ADDIOpcodeTestLargeNegative2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $3, 0($19)
                ADDI $3, $3, -10208
                SD $3, 0($19)
            ", out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x43211b41, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestWithR0 {}

impl Test for ADDIOpcodeTestWithR0 {
    fn name(&self) -> &str { "ADDIOpcodeTestWithR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $3, $0, 4660
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1234, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestWithOffsetZero {}

impl Test for ADDIOpcodeTestWithOffsetZero {
    fn name(&self) -> &str { "ADDIOpcodeTestWithOffsetZero" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $3, $2, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xffffffff98760110, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestWithOffsetZeroAndR0 {}

impl Test for ADDIOpcodeTestWithOffsetZeroAndR0 {
    fn name(&self) -> &str { "ADDIOpcodeTestWithOffsetZeroAndR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestIntoR0 {}

impl Test for ADDIOpcodeTestIntoR0 {
    fn name(&self) -> &str { "ADDIOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $0, $2, 4660
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIOpcodeTestIntoR0WithOffset0 {}

impl Test for ADDIOpcodeTestIntoR0WithOffset0 {
    fn name(&self) -> &str { "ADDIOpcodeTestIntoR0WithOffset0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDI $0, $2, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTest {}

impl Test for ADDIUOpcodeTest {
    fn name(&self) -> &str { "ADDIUOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $3, $2, 4660
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0xffffffff98761344, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestWithNegativeImmediate {}

impl Test for ADDIUOpcodeTestWithNegativeImmediate {
    fn name(&self) -> &str { "ADDIUOpcodeTestWithNegativeImmediate" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $3, $2, -3532
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0xffffffff9875f344, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestLower12Bit {}

impl Test for ADDIUOpcodeTestLower12Bit {
    fn name(&self) -> &str { "ADDIUOpcodeTestLower12Bit" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x64;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $3, $2, 4000
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1004, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestUpper4Bit {}

impl Test for ADDIUOpcodeTestUpper4Bit {
    fn name(&self) -> &str { "ADDIUOpcodeTestUpper4Bit" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x64;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $3, $2, 12288
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x3064, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestLower12BitNegative {}

impl Test for ADDIUOpcodeTestLower12BitNegative {
    fn name(&self) -> &str { "ADDIUOpcodeTestLower12BitNegative" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x64;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $3, $2, -50
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x32, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestUpper4BitNegative {}

impl Test for ADDIUOpcodeTestUpper4BitNegative {
    fn name(&self) -> &str { "ADDIUOpcodeTestUpper4BitNegative" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x64;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $3, $2, -12288
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0xffffffffffffd064, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestLargeNegative1 {}

impl Test for ADDIUOpcodeTestLargeNegative1 {
    fn name(&self) -> &str { "ADDIUOpcodeTestLargeNegative1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $3, 0($19)
                ADDIU $4, $3, -10208
                SD $4, 0($20)
            ", out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x43211b41, "Register $4")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestLargeNegative2 {}

impl Test for ADDIUOpcodeTestLargeNegative2 {
    fn name(&self) -> &str { "ADDIUOpcodeTestLargeNegative2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $3, 0($19)
                ADDIU $3, $3, -10208
                SD $3, 0($19)
            ", out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x43211b41, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestWithR0 {}

impl Test for ADDIUOpcodeTestWithR0 {
    fn name(&self) -> &str { "ADDIUOpcodeTestWithR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $3, $0, 4660
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1234, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestWithOffsetZero {}

impl Test for ADDIUOpcodeTestWithOffsetZero {
    fn name(&self) -> &str { "ADDIUOpcodeTestWithOffsetZero" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $3, $2, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xffffffff98760110, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestWithOffsetZeroAndR0 {}

impl Test for ADDIUOpcodeTestWithOffsetZeroAndR0 {
    fn name(&self) -> &str { "ADDIUOpcodeTestWithOffsetZeroAndR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestWithOffsetZeroIntoItself {}

impl Test for ADDIUOpcodeTestWithOffsetZeroIntoItself {
    fn name(&self) -> &str { "ADDIUOpcodeTestWithOffsetZeroIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432183214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $3, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xffffffff83214321, "Register $3")?;
        Ok(())
    }
}

pub struct ADDIUOpcodeTestIntoR0 {}

impl Test for ADDIUOpcodeTestIntoR0 {
    fn name(&self) -> &str { "ADDIUOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ADDIU $0, $2, 4660
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTest1 {}

impl Test for SLTIOpcodeTest1 {
    fn name(&self) -> &str { "SLTIOpcodeTest1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x100000000000;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $2, 5
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTest2 {}

impl Test for SLTIOpcodeTest2 {
    fn name(&self) -> &str { "SLTIOpcodeTest2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x2;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $2, 5
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTest3 {}

impl Test for SLTIOpcodeTest3 {
    fn name(&self) -> &str { "SLTIOpcodeTest3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xfffffffffffffffc;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $2, -2
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTest4 {}

impl Test for SLTIOpcodeTest4 {
    fn name(&self) -> &str { "SLTIOpcodeTest4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xfffffffffffffffc;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $2, -5
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTest5 {}

impl Test for SLTIOpcodeTest5 {
    fn name(&self) -> &str { "SLTIOpcodeTest5" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xfffffffffffffffc;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $2, 511
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTest6 {}

impl Test for SLTIOpcodeTest6 {
    fn name(&self) -> &str { "SLTIOpcodeTest6" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xfffffffffffffffc;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $2, -261
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTestEqual {}

impl Test for SLTIOpcodeTestEqual {
    fn name(&self) -> &str { "SLTIOpcodeTestEqual" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xfffffffffffffffc;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $2, -4
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTestWithR0 {}

impl Test for SLTIOpcodeTestWithR0 {
    fn name(&self) -> &str { "SLTIOpcodeTestWithR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $0, 1
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTestWithR0_1 {}

impl Test for SLTIOpcodeTestWithR0_1 {
    fn name(&self) -> &str { "SLTIOpcodeTestWithR0_1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $0, -1
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTestIntoR0 {}

impl Test for SLTIOpcodeTestIntoR0 {
    fn name(&self) -> &str { "SLTIOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1233;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $0, $2, 4660
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1233, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTestPositiveAgainst0 {}

impl Test for SLTIOpcodeTestPositiveAgainst0 {
    fn name(&self) -> &str { "SLTIOpcodeTestPositiveAgainst0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1233;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $2, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1233, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTestNegativeAgainst0 {}

impl Test for SLTIOpcodeTestNegativeAgainst0 {
    fn name(&self) -> &str { "SLTIOpcodeTestNegativeAgainst0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0xfffffffffffff233;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $2, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0xfffffffffffff233, "Register $2")?;
        soft_assert_eq(r3, 0x1, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIOpcodeTestR0Against0 {}

impl Test for SLTIOpcodeTestR0Against0 {
    fn name(&self) -> &str { "SLTIOpcodeTestR0Against0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1233;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTI $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1233, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTest1 {}

impl Test for SLTIUOpcodeTest1 {
    fn name(&self) -> &str { "SLTIUOpcodeTest1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x100000000000;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $3, $2, 5
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTest2 {}

impl Test for SLTIUOpcodeTest2 {
    fn name(&self) -> &str { "SLTIUOpcodeTest2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x2;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $3, $2, 5
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTest3 {}

impl Test for SLTIUOpcodeTest3 {
    fn name(&self) -> &str { "SLTIUOpcodeTest3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xfffffffffffffffc;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $3, $2, -2
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTest4 {}

impl Test for SLTIUOpcodeTest4 {
    fn name(&self) -> &str { "SLTIUOpcodeTest4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xfffffffffffffffc;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $3, $2, -5
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTestEqual {}

impl Test for SLTIUOpcodeTestEqual {
    fn name(&self) -> &str { "SLTIUOpcodeTestEqual" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xfffffffffffffffc;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $3, $2, -4
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTestWithR0 {}

impl Test for SLTIUOpcodeTestWithR0 {
    fn name(&self) -> &str { "SLTIUOpcodeTestWithR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $3, $0, 1
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTestWithR0_1 {}

impl Test for SLTIUOpcodeTestWithR0_1 {
    fn name(&self) -> &str { "SLTIUOpcodeTestWithR0_1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $3, $0, -1
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTestIntoR0 {}

impl Test for SLTIUOpcodeTestIntoR0 {
    fn name(&self) -> &str { "SLTIUOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1233;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $0, $2, 4660
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1233, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTestPositiveAgainst0 {}

impl Test for SLTIUOpcodeTestPositiveAgainst0 {
    fn name(&self) -> &str { "SLTIUOpcodeTestPositiveAgainst0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1233;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $3, $2, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1233, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTestNegativeAgainst0 {}

impl Test for SLTIUOpcodeTestNegativeAgainst0 {
    fn name(&self) -> &str { "SLTIUOpcodeTestNegativeAgainst0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0xffffffffffff1234;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $3, $2, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0xffffffffffff1234, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct SLTIUOpcodeTestR0Against0 {}

impl Test for SLTIUOpcodeTestR0Against0 {
    fn name(&self) -> &str { "SLTIUOpcodeTestR0Against0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1233;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                SLTIU $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1233, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct OROpcodeTest {}

impl Test for OROpcodeTest {
    fn name(&self) -> &str { "OROpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                OR $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x53355779db774331, "Register $4")?;
        Ok(())
    }
}

pub struct OROpcodeTestRTIsR0 {}

impl Test for OROpcodeTestRTIsR0 {
    fn name(&self) -> &str { "OROpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                OR $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct OROpcodeTestRSIsR0 {}

impl Test for OROpcodeTestRSIsR0 {
    fn name(&self) -> &str { "OROpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                OR $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct OROpcodeTestBothAreR0 {}

impl Test for OROpcodeTestBothAreR0 {
    fn name(&self) -> &str { "OROpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                OR $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct OROpcodeTestIntoR0 {}

impl Test for OROpcodeTestIntoR0 {
    fn name(&self) -> &str { "OROpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                OR $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct OROpcodeTestWithItself {}

impl Test for OROpcodeTestWithItself {
    fn name(&self) -> &str { "OROpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                OR $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct OROpcodeTestInputOutput1 {}

impl Test for OROpcodeTestInputOutput1 {
    fn name(&self) -> &str { "OROpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                OR $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xebedefabcfefebed, "Register $4")?;
        Ok(())
    }
}

pub struct OROpcodeTestInputOutput2 {}

impl Test for OROpcodeTestInputOutput2 {
    fn name(&self) -> &str { "OROpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                OR $4, $4, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xebedefabcfefebed, "Register $4")?;
        Ok(())
    }
}

pub struct OROpcodeTestInputOutput3 {}

impl Test for OROpcodeTestInputOutput3 {
    fn name(&self) -> &str { "OROpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                OR $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct ANDOpcodeTest {}

impl Test for ANDOpcodeTest {
    fn name(&self) -> &str { "ANDOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321012345432101;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                AND $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321012345432101, "Register $3")?;
        soft_assert_eq(r4, 0x220002000420100, "Register $4")?;
        Ok(())
    }
}

pub struct ANDOpcodeTestRTIsR0 {}

impl Test for ANDOpcodeTestRTIsR0 {
    fn name(&self) -> &str { "ANDOpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                AND $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct ANDOpcodeTestRSIsR0 {}

impl Test for ANDOpcodeTestRSIsR0 {
    fn name(&self) -> &str { "ANDOpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                AND $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct ANDOpcodeTestBothAreR0 {}

impl Test for ANDOpcodeTestBothAreR0 {
    fn name(&self) -> &str { "ANDOpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                AND $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct ANDOpcodeTestIntoR0 {}

impl Test for ANDOpcodeTestIntoR0 {
    fn name(&self) -> &str { "ANDOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                AND $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct ANDOpcodeTestWithItself {}

impl Test for ANDOpcodeTestWithItself {
    fn name(&self) -> &str { "ANDOpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                AND $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct ANDOpcodeTestInputOutput1 {}

impl Test for ANDOpcodeTestInputOutput1 {
    fn name(&self) -> &str { "ANDOpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                AND $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x301432141210301, "Register $4")?;
        Ok(())
    }
}

pub struct ANDOpcodeTestInputOutput2 {}

impl Test for ANDOpcodeTestInputOutput2 {
    fn name(&self) -> &str { "ANDOpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                AND $4, $4, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x301432141210301, "Register $4")?;
        Ok(())
    }
}

pub struct ANDOpcodeTestInputOutput3 {}

impl Test for ANDOpcodeTestInputOutput3 {
    fn name(&self) -> &str { "ANDOpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                AND $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct XOROpcodeTest {}

impl Test for XOROpcodeTest {
    fn name(&self) -> &str { "XOROpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                XOR $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x51151559db574231, "Register $4")?;
        Ok(())
    }
}

pub struct XOROpcodeTestRTIsR0 {}

impl Test for XOROpcodeTestRTIsR0 {
    fn name(&self) -> &str { "XOROpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                XOR $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct XOROpcodeTestRSIsR0 {}

impl Test for XOROpcodeTestRSIsR0 {
    fn name(&self) -> &str { "XOROpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                XOR $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct XOROpcodeTestBothAreR0 {}

impl Test for XOROpcodeTestBothAreR0 {
    fn name(&self) -> &str { "XOROpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                XOR $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct XOROpcodeTestIntoR0 {}

impl Test for XOROpcodeTestIntoR0 {
    fn name(&self) -> &str { "XOROpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                XOR $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct XOROpcodeTestWithItself {}

impl Test for XOROpcodeTestWithItself {
    fn name(&self) -> &str { "XOROpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                XOR $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct XOROpcodeTestInputOutput1 {}

impl Test for XOROpcodeTestInputOutput1 {
    fn name(&self) -> &str { "XOROpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                XOR $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xe8ecac8a8ecee8ec, "Register $4")?;
        Ok(())
    }
}

pub struct XOROpcodeTestInputOutput2 {}

impl Test for XOROpcodeTestInputOutput2 {
    fn name(&self) -> &str { "XOROpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                XOR $4, $4, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xe8ecac8a8ecee8ec, "Register $4")?;
        Ok(())
    }
}

pub struct XOROpcodeTestInputOutput3 {}

impl Test for XOROpcodeTestInputOutput3 {
    fn name(&self) -> &str { "XOROpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                XOR $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct NOROpcodeTest {}

impl Test for NOROpcodeTest {
    fn name(&self) -> &str { "NOROpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                NOR $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xaccaa8862488bcce, "Register $4")?;
        Ok(())
    }
}

pub struct NOROpcodeTestRTIsR0 {}

impl Test for NOROpcodeTestRTIsR0 {
    fn name(&self) -> &str { "NOROpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                NOR $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xbcdebcdebcdebcde, "Register $4")?;
        Ok(())
    }
}

pub struct NOROpcodeTestRSIsR0 {}

impl Test for NOROpcodeTestRSIsR0 {
    fn name(&self) -> &str { "NOROpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                NOR $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xbcdebcdebcdebcde, "Register $4")?;
        Ok(())
    }
}

pub struct NOROpcodeTestBothAreR0 {}

impl Test for NOROpcodeTestBothAreR0 {
    fn name(&self) -> &str { "NOROpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                NOR $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffffffffff, "Register $4")?;
        Ok(())
    }
}

pub struct NOROpcodeTestIntoR0 {}

impl Test for NOROpcodeTestIntoR0 {
    fn name(&self) -> &str { "NOROpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                NOR $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct NOROpcodeTestWithItself {}

impl Test for NOROpcodeTestWithItself {
    fn name(&self) -> &str { "NOROpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                NOR $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xbcdebcdebcdebcde, "Register $4")?;
        Ok(())
    }
}

pub struct NOROpcodeTestInputOutput1 {}

impl Test for NOROpcodeTestInputOutput1 {
    fn name(&self) -> &str { "NOROpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                NOR $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x1412105430101412, "Register $4")?;
        Ok(())
    }
}

pub struct NOROpcodeTestInputOutput2 {}

impl Test for NOROpcodeTestInputOutput2 {
    fn name(&self) -> &str { "NOROpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                NOR $4, $4, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x1412105430101412, "Register $4")?;
        Ok(())
    }
}

pub struct NOROpcodeTestInputOutput3 {}

impl Test for NOROpcodeTestInputOutput3 {
    fn name(&self) -> &str { "NOROpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                NOR $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x5432105432105432, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTest {}

impl Test for SLTOpcodeTest {
    fn name(&self) -> &str { "SLTOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x2710;
        let r3: u64 = 0x2711;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x1, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTest2 {}

impl Test for SLTOpcodeTest2 {
    fn name(&self) -> &str { "SLTOpcodeTest2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x2710;
        let r3: u64 = 0x2710;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTest3 {}

impl Test for SLTOpcodeTest3 {
    fn name(&self) -> &str { "SLTOpcodeTest3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x2710;
        let r3: u64 = 0x270f;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTest4 {}

impl Test for SLTOpcodeTest4 {
    fn name(&self) -> &str { "SLTOpcodeTest4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xffffff01ffffffff;
        let r3: u64 = 0xffffff02ffffffff;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x1, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTest5 {}

impl Test for SLTOpcodeTest5 {
    fn name(&self) -> &str { "SLTOpcodeTest5" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xffffff01ffffffff;
        let r3: u64 = 0xffffff01ffffffff;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTest6 {}

impl Test for SLTOpcodeTest6 {
    fn name(&self) -> &str { "SLTOpcodeTest6" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xffffff01ffffffff;
        let r3: u64 = 0xffffff00ffffffff;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTest7 {}

impl Test for SLTOpcodeTest7 {
    fn name(&self) -> &str { "SLTOpcodeTest7" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x2710;
        let r3: u64 = 0xffffffffffffff00;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTest8 {}

impl Test for SLTOpcodeTest8 {
    fn name(&self) -> &str { "SLTOpcodeTest8" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x10000270f;
        let r3: u64 = 0xffffffffffffff00;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTestWithSelf {}

impl Test for SLTOpcodeTestWithSelf {
    fn name(&self) -> &str { "SLTOpcodeTestWithSelf" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xdeadbeef;
        let r3: u64 = 0xffffffffffffff00;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $2
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTestWithR0Pos {}

impl Test for SLTOpcodeTestWithR0Pos {
    fn name(&self) -> &str { "SLTOpcodeTestWithR0Pos" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1;
        let r3: u64 = 0xffffffffffffff00;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $0
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTestWithR0Neg {}

impl Test for SLTOpcodeTestWithR0Neg {
    fn name(&self) -> &str { "SLTOpcodeTestWithR0Neg" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xffffffffffffffff;
        let r3: u64 = 0xffffffffffffff00;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $4, $2, $0
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x1, "Register $4")?;
        Ok(())
    }
}

pub struct SLTOpcodeTestIntoR0 {}

impl Test for SLTOpcodeTestIntoR0 {
    fn name(&self) -> &str { "SLTOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let r2: u64 = 0x1;
        let r3: u64 = 0x2;
        let r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLT $0, $2, $3
                SD $0, 0($16)
            ", in("$16") &mut r0, out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTest {}

impl Test for SLTUOpcodeTest {
    fn name(&self) -> &str { "SLTUOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x2710;
        let r3: u64 = 0x2711;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x1, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTest2 {}

impl Test for SLTUOpcodeTest2 {
    fn name(&self) -> &str { "SLTUOpcodeTest2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x2710;
        let r3: u64 = 0x2710;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTest3 {}

impl Test for SLTUOpcodeTest3 {
    fn name(&self) -> &str { "SLTUOpcodeTest3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x2710;
        let r3: u64 = 0x270f;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTest4 {}

impl Test for SLTUOpcodeTest4 {
    fn name(&self) -> &str { "SLTUOpcodeTest4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xffffff01ffffffff;
        let r3: u64 = 0xffffff02ffffffff;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x1, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTest5 {}

impl Test for SLTUOpcodeTest5 {
    fn name(&self) -> &str { "SLTUOpcodeTest5" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xffffff01ffffffff;
        let r3: u64 = 0xffffff01ffffffff;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTest6 {}

impl Test for SLTUOpcodeTest6 {
    fn name(&self) -> &str { "SLTUOpcodeTest6" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xffffff01ffffffff;
        let r3: u64 = 0xffffff00ffffffff;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTest7 {}

impl Test for SLTUOpcodeTest7 {
    fn name(&self) -> &str { "SLTUOpcodeTest7" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x2710;
        let r3: u64 = 0xffffffffffffff00;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x1, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTest8 {}

impl Test for SLTUOpcodeTest8 {
    fn name(&self) -> &str { "SLTUOpcodeTest8" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x10000270f;
        let r3: u64 = 0xffffffffffffff00;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $3
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x1, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTestWithSelf {}

impl Test for SLTUOpcodeTestWithSelf {
    fn name(&self) -> &str { "SLTUOpcodeTestWithSelf" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xdeadbeef;
        let r3: u64 = 0xffffffffffffff00;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $2
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTestWithR0Pos {}

impl Test for SLTUOpcodeTestWithR0Pos {
    fn name(&self) -> &str { "SLTUOpcodeTestWithR0Pos" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1;
        let r3: u64 = 0xffffffffffffff00;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $0
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTestWithR0Neg {}

impl Test for SLTUOpcodeTestWithR0Neg {
    fn name(&self) -> &str { "SLTUOpcodeTestWithR0Neg" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0xffffffffffffffff;
        let r3: u64 = 0xffffffffffffff00;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $4, $2, $0
                SD $4, 0($20)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SLTUOpcodeTestIntoR0 {}

impl Test for SLTUOpcodeTestIntoR0 {
    fn name(&self) -> &str { "SLTUOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let r2: u64 = 0x1;
        let r3: u64 = 0x2;
        let r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLTU $0, $2, $3
                SD $0, 0($16)
            ", in("$16") &mut r0, out("$2") _, in("$18") &r2, out("$3") _, in("$19") &r3, out("$4") _, in("$20") &r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        Ok(())
    }
}

pub struct ORIOpcodeTest1 {}

impl Test for ORIOpcodeTest1 {
    fn name(&self) -> &str { "ORIOpcodeTest1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ORI $3, $2, 0xf2ff
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x123456789876f3ff, "Register $3")?;
        Ok(())
    }
}

pub struct ORIOpcodeTest2 {}

impl Test for ORIOpcodeTest2 {
    fn name(&self) -> &str { "ORIOpcodeTest2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ORI $3, $2, 0x1234
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1234567898761334, "Register $3")?;
        Ok(())
    }
}

pub struct ORIOpcodeTest3 {}

impl Test for ORIOpcodeTest3 {
    fn name(&self) -> &str { "ORIOpcodeTest3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ORI $3, $2, 0xf0
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x12345678987601f0, "Register $3")?;
        Ok(())
    }
}

pub struct ORIOpcodeTest4 {}

impl Test for ORIOpcodeTest4 {
    fn name(&self) -> &str { "ORIOpcodeTest4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ORI $3, $2, 0xc0
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x12345678987601d0, "Register $3")?;
        Ok(())
    }
}

pub struct ORIOpcodeTestWithR0 {}

impl Test for ORIOpcodeTestWithR0 {
    fn name(&self) -> &str { "ORIOpcodeTestWithR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ORI $3, $0, 0x1234
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1234, "Register $3")?;
        Ok(())
    }
}

pub struct ORIOpcodeTestWithOffsetZero {}

impl Test for ORIOpcodeTestWithOffsetZero {
    fn name(&self) -> &str { "ORIOpcodeTestWithOffsetZero" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ORI $3, $2, 0x0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1234567898760110, "Register $3")?;
        Ok(())
    }
}

pub struct ORIOpcodeTestWithOffsetZeroAndR0 {}

impl Test for ORIOpcodeTestWithOffsetZeroAndR0 {
    fn name(&self) -> &str { "ORIOpcodeTestWithOffsetZeroAndR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ORI $3, $0, 0x0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct ORIOpcodeTestIntoR0 {}

impl Test for ORIOpcodeTestIntoR0 {
    fn name(&self) -> &str { "ORIOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ORI $0, $2, 0x1234
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct ANDIOpcodeTest1 {}

impl Test for ANDIOpcodeTest1 {
    fn name(&self) -> &str { "ANDIOpcodeTest1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ANDI $3, $2, 0x1234
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x10, "Register $3")?;
        Ok(())
    }
}

pub struct ANDIOpcodeTest2 {}

impl Test for ANDIOpcodeTest2 {
    fn name(&self) -> &str { "ANDIOpcodeTest2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ANDI $3, $2, 0xffff
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x110, "Register $3")?;
        Ok(())
    }
}

pub struct ANDIOpcodeTest3 {}

impl Test for ANDIOpcodeTest3 {
    fn name(&self) -> &str { "ANDIOpcodeTest3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x123456789876ff80;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ANDI $3, $2, 0x1fc0
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1f80, "Register $3")?;
        Ok(())
    }
}

pub struct ANDIOpcodeTest4 {}

impl Test for ANDIOpcodeTest4 {
    fn name(&self) -> &str { "ANDIOpcodeTest4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x123456789876ffe3;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ANDI $3, $2, 0x72
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x62, "Register $3")?;
        Ok(())
    }
}

pub struct ANDIOpcodeTest5 {}

impl Test for ANDIOpcodeTest5 {
    fn name(&self) -> &str { "ANDIOpcodeTest5" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x123456789876ffe2;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ANDI $3, $2, 0x83
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x82, "Register $3")?;
        Ok(())
    }
}

pub struct ANDIOpcodeTestWithR0 {}

impl Test for ANDIOpcodeTestWithR0 {
    fn name(&self) -> &str { "ANDIOpcodeTestWithR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ANDI $3, $0, 0x1234
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct ANDIOpcodeTestWithOffsetZero {}

impl Test for ANDIOpcodeTestWithOffsetZero {
    fn name(&self) -> &str { "ANDIOpcodeTestWithOffsetZero" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ANDI $3, $2, 0x0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct ANDIOpcodeTestWithOffsetZeroAndR0 {}

impl Test for ANDIOpcodeTestWithOffsetZeroAndR0 {
    fn name(&self) -> &str { "ANDIOpcodeTestWithOffsetZeroAndR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ANDI $3, $0, 0x0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct ANDIOpcodeTestIntoR0 {}

impl Test for ANDIOpcodeTestIntoR0 {
    fn name(&self) -> &str { "ANDIOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                ANDI $0, $2, 0x1234
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct XORIOpcodeTest1 {}

impl Test for XORIOpcodeTest1 {
    fn name(&self) -> &str { "XORIOpcodeTest1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                XORI $3, $2, 0x1234
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1234567898761324, "Register $3")?;
        Ok(())
    }
}

pub struct XORIOpcodeTest2 {}

impl Test for XORIOpcodeTest2 {
    fn name(&self) -> &str { "XORIOpcodeTest2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                XORI $3, $2, 0xffff
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x123456789876feef, "Register $3")?;
        Ok(())
    }
}

pub struct XORIOpcodeTest3 {}

impl Test for XORIOpcodeTest3 {
    fn name(&self) -> &str { "XORIOpcodeTest3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                XORI $3, $2, 0x1f0
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x12345678987600e0, "Register $3")?;
        Ok(())
    }
}

pub struct XORIOpcodeTest4 {}

impl Test for XORIOpcodeTest4 {
    fn name(&self) -> &str { "XORIOpcodeTest4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                XORI $3, $2, 0x71
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1234567898760161, "Register $3")?;
        Ok(())
    }
}

pub struct XORIOpcodeTest5 {}

impl Test for XORIOpcodeTest5 {
    fn name(&self) -> &str { "XORIOpcodeTest5" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                XORI $3, $2, 0x80
                SD $3, 0($19)
            ", out("$2") _, in("$18") &r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r3, 0x1234567898760190, "Register $3")?;
        Ok(())
    }
}

pub struct XORIOpcodeTestWithR0 {}

impl Test for XORIOpcodeTestWithR0 {
    fn name(&self) -> &str { "XORIOpcodeTestWithR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                XORI $3, $0, 0x1234
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1234, "Register $3")?;
        Ok(())
    }
}

pub struct XORIOpcodeTestWithOffsetZero {}

impl Test for XORIOpcodeTestWithOffsetZero {
    fn name(&self) -> &str { "XORIOpcodeTestWithOffsetZero" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                XORI $3, $2, 0x0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x1234567898760110, "Register $3")?;
        Ok(())
    }
}

pub struct XORIOpcodeTestWithOffsetZeroAndR0 {}

impl Test for XORIOpcodeTestWithOffsetZeroAndR0 {
    fn name(&self) -> &str { "XORIOpcodeTestWithOffsetZeroAndR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                XORI $3, $0, 0x0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        Ok(())
    }
}

pub struct XORIOpcodeTestIntoR0 {}

impl Test for XORIOpcodeTestIntoR0 {
    fn name(&self) -> &str { "XORIOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                XORI $0, $2, 0x1234
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        Ok(())
    }
}

pub struct ADDOpcodeTest {}

impl Test for ADDOpcodeTest {
    fn name(&self) -> &str { "ADDOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADD $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffdb974431, "Register $4")?;
        Ok(())
    }
}

pub struct ADDOpcodeTestInputOutput1 {}

impl Test for ADDOpcodeTestInputOutput1 {
    fn name(&self) -> &str { "ADDOpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefab1defabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADD $4, $4, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffb665acdd, "Register $4")?;
        Ok(())
    }
}

pub struct ADDOpcodeTestInputOutput2 {}

impl Test for ADDOpcodeTestInputOutput2 {
    fn name(&self) -> &str { "ADDOpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADD $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x1110eeee, "Register $4")?;
        Ok(())
    }
}

pub struct ADDOpcodeTestInputOutput3 {}

impl Test for ADDOpcodeTestInputOutput3 {
    fn name(&self) -> &str { "ADDOpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0x1;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADD $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x2, "Register $4")?;
        Ok(())
    }
}

pub struct ADDOpcodeTestRTIsR0 {}

impl Test for ADDOpcodeTestRTIsR0 {
    fn name(&self) -> &str { "ADDOpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADD $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x43214321, "Register $4")?;
        Ok(())
    }
}

pub struct ADDOpcodeTestRSIsR0 {}

impl Test for ADDOpcodeTestRSIsR0 {
    fn name(&self) -> &str { "ADDOpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADD $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x43214321, "Register $4")?;
        Ok(())
    }
}

pub struct ADDOpcodeTestBothAreR0 {}

impl Test for ADDOpcodeTestBothAreR0 {
    fn name(&self) -> &str { "ADDOpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADD $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct ADDOpcodeTestIntoR0 {}

impl Test for ADDOpcodeTestIntoR0 {
    fn name(&self) -> &str { "ADDOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADD $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct ADDOpcodeTestWithItself {}

impl Test for ADDOpcodeTestWithItself {
    fn name(&self) -> &str { "ADDOpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432133214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADD $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432133214321, "Register $3")?;
        soft_assert_eq(r4, 0x66428642, "Register $4")?;
        Ok(())
    }
}

pub struct ADDUOpcodeTest {}

impl Test for ADDUOpcodeTest {
    fn name(&self) -> &str { "ADDUOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADDU $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffdb974431, "Register $4")?;
        Ok(())
    }
}

pub struct ADDUOpcodeTestInputOutput1 {}

impl Test for ADDUOpcodeTestInputOutput1 {
    fn name(&self) -> &str { "ADDUOpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADDU $4, $4, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x6665acdd, "Register $4")?;
        Ok(())
    }
}

pub struct ADDUOpcodeTestInputOutput2 {}

impl Test for ADDUOpcodeTestInputOutput2 {
    fn name(&self) -> &str { "ADDUOpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADDU $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x1110eeee, "Register $4")?;
        Ok(())
    }
}

pub struct ADDUOpcodeTestInputOutput3 {}

impl Test for ADDUOpcodeTestInputOutput3 {
    fn name(&self) -> &str { "ADDUOpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADDU $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffff9bdf579a, "Register $4")?;
        Ok(())
    }
}

pub struct ADDUOpcodeTestRTIsR0 {}

impl Test for ADDUOpcodeTestRTIsR0 {
    fn name(&self) -> &str { "ADDUOpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADDU $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x43214321, "Register $4")?;
        Ok(())
    }
}

pub struct ADDUOpcodeTestRSIsR0 {}

impl Test for ADDUOpcodeTestRSIsR0 {
    fn name(&self) -> &str { "ADDUOpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADDU $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x43214321, "Register $4")?;
        Ok(())
    }
}

pub struct ADDUOpcodeTestBothAreR0 {}

impl Test for ADDUOpcodeTestBothAreR0 {
    fn name(&self) -> &str { "ADDUOpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADDU $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct ADDUOpcodeTestIntoR0 {}

impl Test for ADDUOpcodeTestIntoR0 {
    fn name(&self) -> &str { "ADDUOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADDU $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct ADDUOpcodeTestWithItself {}

impl Test for ADDUOpcodeTestWithItself {
    fn name(&self) -> &str { "ADDUOpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                ADDU $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffff86428642, "Register $4")?;
        Ok(())
    }
}

pub struct DADDOpcodeTest {}

impl Test for DADDOpcodeTest {
    fn name(&self) -> &str { "DADDOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADD $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x55559999db974431, "Register $4")?;
        Ok(())
    }
}

pub struct DADDOpcodeTestInputOutput1 {}

impl Test for DADDOpcodeTestInputOutput1 {
    fn name(&self) -> &str { "DADDOpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefab1defabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADD $4, $4, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xbe024623b665acdd, "Register $4")?;
        Ok(())
    }
}

pub struct DADDOpcodeTestInputOutput2 {}

impl Test for DADDOpcodeTestInputOutput2 {
    fn name(&self) -> &str { "DADDOpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADD $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xeeef32cd1110eeee, "Register $4")?;
        Ok(())
    }
}

pub struct DADDOpcodeTestInputOutput3 {}

impl Test for DADDOpcodeTestInputOutput3 {
    fn name(&self) -> &str { "DADDOpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0x1;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADD $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x2, "Register $4")?;
        Ok(())
    }
}

pub struct DADDOpcodeTestRTIsR0 {}

impl Test for DADDOpcodeTestRTIsR0 {
    fn name(&self) -> &str { "DADDOpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADD $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct DADDOpcodeTestRSIsR0 {}

impl Test for DADDOpcodeTestRSIsR0 {
    fn name(&self) -> &str { "DADDOpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADD $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct DADDOpcodeTestBothAreR0 {}

impl Test for DADDOpcodeTestBothAreR0 {
    fn name(&self) -> &str { "DADDOpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADD $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DADDOpcodeTestIntoR0 {}

impl Test for DADDOpcodeTestIntoR0 {
    fn name(&self) -> &str { "DADDOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADD $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DADDOpcodeTestWithItself {}

impl Test for DADDOpcodeTestWithItself {
    fn name(&self) -> &str { "DADDOpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x3321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADD $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x3321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x6642864286428642, "Register $4")?;
        Ok(())
    }
}

pub struct DADDUOpcodeTest {}

impl Test for DADDUOpcodeTest {
    fn name(&self) -> &str { "DADDUOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADDU $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x55559999db974431, "Register $4")?;
        Ok(())
    }
}

pub struct DADDUOpcodeTestInputOutput1 {}

impl Test for DADDUOpcodeTestInputOutput1 {
    fn name(&self) -> &str { "DADDUOpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADDU $4, $4, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xbe0246246665acdd, "Register $4")?;
        Ok(())
    }
}

pub struct DADDUOpcodeTestInputOutput2 {}

impl Test for DADDUOpcodeTestInputOutput2 {
    fn name(&self) -> &str { "DADDUOpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADDU $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xeeef32cd1110eeee, "Register $4")?;
        Ok(())
    }
}

pub struct DADDUOpcodeTestInputOutput3 {}

impl Test for DADDUOpcodeTestInputOutput3 {
    fn name(&self) -> &str { "DADDUOpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0x2bcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADDU $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x579bdf579bdf579a, "Register $4")?;
        Ok(())
    }
}

pub struct DADDUOpcodeTestRTIsR0 {}

impl Test for DADDUOpcodeTestRTIsR0 {
    fn name(&self) -> &str { "DADDUOpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADDU $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct DADDUOpcodeTestRSIsR0 {}

impl Test for DADDUOpcodeTestRSIsR0 {
    fn name(&self) -> &str { "DADDUOpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADDU $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct DADDUOpcodeTestBothAreR0 {}

impl Test for DADDUOpcodeTestBothAreR0 {
    fn name(&self) -> &str { "DADDUOpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADDU $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DADDUOpcodeTestIntoR0 {}

impl Test for DADDUOpcodeTestIntoR0 {
    fn name(&self) -> &str { "DADDUOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADDU $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DADDUOpcodeTestWithItself {}

impl Test for DADDUOpcodeTestWithItself {
    fn name(&self) -> &str { "DADDUOpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DADDU $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x8642864286428642, "Register $4")?;
        Ok(())
    }
}

pub struct SUBOpcodeTest {}

impl Test for SUBOpcodeTest {
    fn name(&self) -> &str { "SUBOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUB $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x2aab4211, "Register $4")?;
        Ok(())
    }
}

pub struct SUBOpcodeTestInputOutput1 {}

impl Test for SUBOpcodeTestInputOutput1 {
    fn name(&self) -> &str { "SUBOpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x3321432143214321;
        let mut r4: u64 = 0x3bcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUB $4, $4, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x3321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffb579aabd, "Register $4")?;
        Ok(())
    }
}

pub struct SUBOpcodeTestInputOutput2 {}

impl Test for SUBOpcodeTestInputOutput2 {
    fn name(&self) -> &str { "SUBOpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0x8000000010000000;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUB $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x33214321, "Register $4")?;
        Ok(())
    }
}

pub struct SUBOpcodeTestInputOutput3 {}

impl Test for SUBOpcodeTestInputOutput3 {
    fn name(&self) -> &str { "SUBOpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUB $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SUBOpcodeTestRTIsR0 {}

impl Test for SUBOpcodeTestRTIsR0 {
    fn name(&self) -> &str { "SUBOpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUB $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x43214321, "Register $4")?;
        Ok(())
    }
}

pub struct SUBOpcodeTestRSIsR0 {}

impl Test for SUBOpcodeTestRSIsR0 {
    fn name(&self) -> &str { "SUBOpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUB $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffbcdebcdf, "Register $4")?;
        Ok(())
    }
}

pub struct SUBOpcodeTestBothAreR0 {}

impl Test for SUBOpcodeTestBothAreR0 {
    fn name(&self) -> &str { "SUBOpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUB $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SUBOpcodeTestIntoR0 {}

impl Test for SUBOpcodeTestIntoR0 {
    fn name(&self) -> &str { "SUBOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567878760110;
        let mut r3: u64 = 0x321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUB $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567878760110, "Register $2")?;
        soft_assert_eq(r3, 0x321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SUBOpcodeTestWithItself {}

impl Test for SUBOpcodeTestWithItself {
    fn name(&self) -> &str { "SUBOpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUB $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SUBUOpcodeTest {}

impl Test for SUBUOpcodeTest {
    fn name(&self) -> &str { "SUBUOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUBU $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffaaab4211, "Register $4")?;
        Ok(())
    }
}

pub struct SUBUOpcodeTestInputOutput1 {}

impl Test for SUBUOpcodeTestInputOutput1 {
    fn name(&self) -> &str { "SUBUOpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x3321432143214321;
        let mut r4: u64 = 0x3bcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUBU $4, $4, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x3321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffb579aabd, "Register $4")?;
        Ok(())
    }
}

pub struct SUBUOpcodeTestInputOutput2 {}

impl Test for SUBUOpcodeTestInputOutput2 {
    fn name(&self) -> &str { "SUBUOpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0x8000000010000000;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUBU $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x33214321, "Register $4")?;
        Ok(())
    }
}

pub struct SUBUOpcodeTestInputOutput3 {}

impl Test for SUBUOpcodeTestInputOutput3 {
    fn name(&self) -> &str { "SUBUOpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUBU $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SUBUOpcodeTestRTIsR0 {}

impl Test for SUBUOpcodeTestRTIsR0 {
    fn name(&self) -> &str { "SUBUOpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUBU $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x43214321, "Register $4")?;
        Ok(())
    }
}

pub struct SUBUOpcodeTestRSIsR0 {}

impl Test for SUBUOpcodeTestRSIsR0 {
    fn name(&self) -> &str { "SUBUOpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUBU $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffbcdebcdf, "Register $4")?;
        Ok(())
    }
}

pub struct SUBUOpcodeTestBothAreR0 {}

impl Test for SUBUOpcodeTestBothAreR0 {
    fn name(&self) -> &str { "SUBUOpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUBU $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SUBUOpcodeTestIntoR0 {}

impl Test for SUBUOpcodeTestIntoR0 {
    fn name(&self) -> &str { "SUBUOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUBU $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SUBUOpcodeTestWithItself {}

impl Test for SUBUOpcodeTestWithItself {
    fn name(&self) -> &str { "SUBUOpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SUBU $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBOpcodeTest {}

impl Test for DSUBOpcodeTest {
    fn name(&self) -> &str { "DSUBOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUB $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x30ececa8aaab4211, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBOpcodeTestInputOutput1 {}

impl Test for DSUBOpcodeTestInputOutput1 {
    fn name(&self) -> &str { "DSUBOpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x3321432143214321;
        let mut r4: u64 = 0x3bcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUB $4, $4, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x3321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x29999933b579aabd, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBOpcodeTestInputOutput2 {}

impl Test for DSUBOpcodeTestInputOutput2 {
    fn name(&self) -> &str { "DSUBOpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0x2000000010000000;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUB $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x2321432133214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBOpcodeTestInputOutput3 {}

impl Test for DSUBOpcodeTestInputOutput3 {
    fn name(&self) -> &str { "DSUBOpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUB $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBOpcodeTestRTIsR0 {}

impl Test for DSUBOpcodeTestRTIsR0 {
    fn name(&self) -> &str { "DSUBOpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUB $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBOpcodeTestRSIsR0 {}

impl Test for DSUBOpcodeTestRSIsR0 {
    fn name(&self) -> &str { "DSUBOpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUB $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xbcdebcdebcdebcdf, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBOpcodeTestBothAreR0 {}

impl Test for DSUBOpcodeTestBothAreR0 {
    fn name(&self) -> &str { "DSUBOpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUB $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBOpcodeTestIntoR0 {}

impl Test for DSUBOpcodeTestIntoR0 {
    fn name(&self) -> &str { "DSUBOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUB $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBOpcodeTestWithItself {}

impl Test for DSUBOpcodeTestWithItself {
    fn name(&self) -> &str { "DSUBOpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUB $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBUOpcodeTest {}

impl Test for DSUBUOpcodeTest {
    fn name(&self) -> &str { "DSUBUOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUBU $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x30ececa8aaab4211, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBUOpcodeTestInputOutput1 {}

impl Test for DSUBUOpcodeTestInputOutput1 {
    fn name(&self) -> &str { "DSUBUOpcodeTestInputOutput1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x3321432143214321;
        let mut r4: u64 = 0x3bcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUBU $4, $4, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x3321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x29999933b579aabd, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBUOpcodeTestInputOutput2 {}

impl Test for DSUBUOpcodeTestInputOutput2 {
    fn name(&self) -> &str { "DSUBUOpcodeTestInputOutput2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0x3000000010000000;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUBU $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x1321432133214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBUOpcodeTestInputOutput3 {}

impl Test for DSUBUOpcodeTestInputOutput3 {
    fn name(&self) -> &str { "DSUBUOpcodeTestInputOutput3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567818760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUBU $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567818760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBUOpcodeTestRTIsR0 {}

impl Test for DSUBUOpcodeTestRTIsR0 {
    fn name(&self) -> &str { "DSUBUOpcodeTestRTIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUBU $4, $3, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBUOpcodeTestRSIsR0 {}

impl Test for DSUBUOpcodeTestRSIsR0 {
    fn name(&self) -> &str { "DSUBUOpcodeTestRSIsR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUBU $4, $0, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xbcdebcdebcdebcdf, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBUOpcodeTestBothAreR0 {}

impl Test for DSUBUOpcodeTestBothAreR0 {
    fn name(&self) -> &str { "DSUBUOpcodeTestBothAreR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUBU $4, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBUOpcodeTestIntoR0 {}

impl Test for DSUBUOpcodeTestIntoR0 {
    fn name(&self) -> &str { "DSUBUOpcodeTestIntoR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUBU $0, $2, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBUOpcodeTestWithItself {}

impl Test for DSUBUOpcodeTestWithItself {
    fn name(&self) -> &str { "DSUBUOpcodeTestWithItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUBU $4, $3, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DSUBUOpcodeTestNoOverflowNegativeMinusPositive {}

impl Test for DSUBUOpcodeTestNoOverflowNegativeMinusPositive {
    fn name(&self) -> &str { "DSUBUOpcodeTestNoOverflowNegativeMinusPositive" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r2: u64 = 0xbffffff000000000;
        let mut r3: u64 = 0x6f00000000123456;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSUBU $4, $3, $2
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r2, 0xbffffff000000000, "Register $2")?;
        soft_assert_eq(r3, 0x6f00000000123456, "Register $3")?;
        soft_assert_eq(r4, 0xaf00001000123456, "Register $4")?;
        Ok(())
    }
}

pub struct SLLOpcodeTest {}

impl Test for SLLOpcodeTest {
    fn name(&self) -> &str { "SLLOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLL $4, $3, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x64286420, "Register $4")?;
        Ok(())
    }
}

pub struct SLLOpcodeTestShift0 {}

impl Test for SLLOpcodeTestShift0 {
    fn name(&self) -> &str { "SLLOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLL $4, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xfffffffff3214321, "Register $4")?;
        Ok(())
    }
}

pub struct SLLOpcodeTestSignExtension {}

impl Test for SLLOpcodeTestSignExtension {
    fn name(&self) -> &str { "SLLOpcodeTestSignExtension" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLL $4, $3, 6
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffc850c840, "Register $4")?;
        Ok(())
    }
}

pub struct SLLOpcodeTestIntoItself {}

impl Test for SLLOpcodeTestIntoItself {
    fn name(&self) -> &str { "SLLOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLL $3, $3, 6
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xffffffffc850c840, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SLLOpcodeTestIntoItself0 {}

impl Test for SLLOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "SLLOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLL $3, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xffffffffc3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SLLOpcodeTestIntoR0Shift0 {}

impl Test for SLLOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "SLLOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLL $0, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SLLOpcodeTestFromR0 {}

impl Test for SLLOpcodeTestFromR0 {
    fn name(&self) -> &str { "SLLOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLL $3, $0, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SLLOpcodeTestFromR0Shift0 {}

impl Test for SLLOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "SLLOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLL $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTest {}

impl Test for SLLVOpcodeTest {
    fn name(&self) -> &str { "SLLVOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x64286420, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestShift0 {}

impl Test for SLLVOpcodeTestShift0 {
    fn name(&self) -> &str { "SLLVOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xfffffffff3214321, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestShiftTargetAndSourceSame1 {}

impl Test for SLLVOpcodeTestShiftTargetAndSourceSame1 {
    fn name(&self) -> &str { "SLLVOpcodeTestShiftTargetAndSourceSame1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x3;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x10, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestShiftTargetAndSourceSame2 {}

impl Test for SLLVOpcodeTestShiftTargetAndSourceSame2 {
    fn name(&self) -> &str { "SLLVOpcodeTestShiftTargetAndSourceSame2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x3;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $4, $4, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0xc, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestShiftTargetAndSourceSame3 {}

impl Test for SLLVOpcodeTestShiftTargetAndSourceSame3 {
    fn name(&self) -> &str { "SLLVOpcodeTestShiftTargetAndSourceSame3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x3;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x18, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestShiftTooLarge {}

impl Test for SLLVOpcodeTestShiftTooLarge {
    fn name(&self) -> &str { "SLLVOpcodeTestShiftTooLarge" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x24;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x24, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0x32143210, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestSignExtension {}

impl Test for SLLVOpcodeTestSignExtension {
    fn name(&self) -> &str { "SLLVOpcodeTestSignExtension" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x6;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x6, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffc850c840, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestIntoItself {}

impl Test for SLLVOpcodeTestIntoItself {
    fn name(&self) -> &str { "SLLVOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x6;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x6, "Register $2")?;
        soft_assert_eq(r3, 0xffffffffc850c840, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestIntoItself0 {}

impl Test for SLLVOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "SLLVOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0xffffffffc3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestIntoR0Shift0 {}

impl Test for SLLVOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "SLLVOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $0, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestFromR0 {}

impl Test for SLLVOpcodeTestFromR0 {
    fn name(&self) -> &str { "SLLVOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $3, $0, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SLLVOpcodeTestFromR0Shift0 {}

impl Test for SLLVOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "SLLVOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SLLV $3, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLL32OpcodeTest {}

impl Test for DSLL32OpcodeTest {
    fn name(&self) -> &str { "DSLL32OpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x8321432183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL32 $4, $3, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x8321432183214321, "Register $3")?;
        soft_assert_eq(r4, 0x6428642000000000, "Register $4")?;
        Ok(())
    }
}

pub struct DSLL32OpcodeTestShift0 {}

impl Test for DSLL32OpcodeTestShift0 {
    fn name(&self) -> &str { "DSLL32OpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x83214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL32 $4, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x83214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xf321432100000000, "Register $4")?;
        Ok(())
    }
}

pub struct DSLL32OpcodeTestIntoItself {}

impl Test for DSLL32OpcodeTestIntoItself {
    fn name(&self) -> &str { "DSLL32OpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x8321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL32 $3, $3, 6
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xc850c84000000000, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLL32OpcodeTestIntoItself0 {}

impl Test for DSLL32OpcodeTestIntoItself0 {
    fn name(&self) -> &str { "DSLL32OpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL32 $3, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xc321432100000000, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLL32OpcodeTestIntoR0Shift0 {}

impl Test for DSLL32OpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "DSLL32OpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL32 $0, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLL32OpcodeTestFromR0 {}

impl Test for DSLL32OpcodeTestFromR0 {
    fn name(&self) -> &str { "DSLL32OpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL32 $3, $0, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLL32OpcodeTestFromR0Shift0 {}

impl Test for DSLL32OpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "DSLL32OpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL32 $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLOpcodeTest {}

impl Test for DSLLOpcodeTest {
    fn name(&self) -> &str { "DSLLOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL $4, $3, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x6428642864286420, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLOpcodeTestShift0 {}

impl Test for DSLLOpcodeTestShift0 {
    fn name(&self) -> &str { "DSLLOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL $4, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLOpcodeTestIntoItself {}

impl Test for DSLLOpcodeTestIntoItself {
    fn name(&self) -> &str { "DSLLOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL $3, $3, 6
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xc850c850c850c840, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLOpcodeTestIntoItself0 {}

impl Test for DSLLOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "DSLLOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL $3, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLOpcodeTestIntoR0Shift0 {}

impl Test for DSLLOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "DSLLOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL $0, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLOpcodeTestFromR0 {}

impl Test for DSLLOpcodeTestFromR0 {
    fn name(&self) -> &str { "DSLLOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL $3, $0, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLOpcodeTestFromR0Shift0 {}

impl Test for DSLLOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "DSLLOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLL $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTest {}

impl Test for DSLLVOpcodeTest {
    fn name(&self) -> &str { "DSLLVOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x6428642864286420, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestShift0 {}

impl Test for DSLLVOpcodeTestShift0 {
    fn name(&self) -> &str { "DSLLVOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0x43214321f3214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestShift1 {}

impl Test for DSLLVOpcodeTestShift1 {
    fn name(&self) -> &str { "DSLLVOpcodeTestShift1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x24;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x24, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0x3214321000000000, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestShiftTargetAndSourceSame1 {}

impl Test for DSLLVOpcodeTestShiftTargetAndSourceSame1 {
    fn name(&self) -> &str { "DSLLVOpcodeTestShiftTargetAndSourceSame1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x3;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x10, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestShiftTargetAndSourceSame2 {}

impl Test for DSLLVOpcodeTestShiftTargetAndSourceSame2 {
    fn name(&self) -> &str { "DSLLVOpcodeTestShiftTargetAndSourceSame2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x3;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $4, $4, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0xc, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestShiftTargetAndSourceSame3 {}

impl Test for DSLLVOpcodeTestShiftTargetAndSourceSame3 {
    fn name(&self) -> &str { "DSLLVOpcodeTestShiftTargetAndSourceSame3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x3;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x18, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestShiftTooLarge {}

impl Test for DSLLVOpcodeTestShiftTooLarge {
    fn name(&self) -> &str { "DSLLVOpcodeTestShiftTooLarge" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x44;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x44, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0x3214321f32143210, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestIntoItself {}

impl Test for DSLLVOpcodeTestIntoItself {
    fn name(&self) -> &str { "DSLLVOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x6;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x6, "Register $2")?;
        soft_assert_eq(r3, 0xc850c850c850c840, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestIntoItself0 {}

impl Test for DSLLVOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "DSLLVOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestIntoR0Shift0 {}

impl Test for DSLLVOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "DSLLVOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $0, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestFromR0 {}

impl Test for DSLLVOpcodeTestFromR0 {
    fn name(&self) -> &str { "DSLLVOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $3, $0, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSLLVOpcodeTestFromR0Shift0 {}

impl Test for DSLLVOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "DSLLVOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSLLV $3, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRLOpcodeTest {}

impl Test for SRLOpcodeTest {
    fn name(&self) -> &str { "SRLOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRL $4, $3, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432183214321, "Register $3")?;
        soft_assert_eq(r4, 0x4190a19, "Register $4")?;
        Ok(())
    }
}

pub struct SRLOpcodeTestShift0 {}

impl Test for SRLOpcodeTestShift0 {
    fn name(&self) -> &str { "SRLOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRL $4, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xfffffffff3214321, "Register $4")?;
        Ok(())
    }
}

pub struct SRLOpcodeTestIntoItself {}

impl Test for SRLOpcodeTestIntoItself {
    fn name(&self) -> &str { "SRLOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRL $3, $3, 6
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x10c850c, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRLOpcodeTestIntoItself0 {}

impl Test for SRLOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "SRLOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRL $3, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xffffffffc3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRLOpcodeTestIntoR0Shift0 {}

impl Test for SRLOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "SRLOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRL $0, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRLOpcodeTestFromR0 {}

impl Test for SRLOpcodeTestFromR0 {
    fn name(&self) -> &str { "SRLOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRL $3, $0, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRLOpcodeTestFromR0Shift0 {}

impl Test for SRLOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "SRLOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRL $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTest {}

impl Test for SRLVOpcodeTest {
    fn name(&self) -> &str { "SRLVOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x4321432183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x4321432183214321, "Register $3")?;
        soft_assert_eq(r4, 0x4190a19, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestShift0 {}

impl Test for SRLVOpcodeTestShift0 {
    fn name(&self) -> &str { "SRLVOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xfffffffff3214321, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestShiftTargetAndSourceSame1 {}

impl Test for SRLVOpcodeTestShiftTargetAndSourceSame1 {
    fn name(&self) -> &str { "SRLVOpcodeTestShiftTargetAndSourceSame1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x10;
        let mut r4: u64 = 0x3;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x10, "Register $3")?;
        soft_assert_eq(r4, 0x2, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestShiftTargetAndSourceSame2 {}

impl Test for SRLVOpcodeTestShiftTargetAndSourceSame2 {
    fn name(&self) -> &str { "SRLVOpcodeTestShiftTargetAndSourceSame2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0xc;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $4, $4, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x3, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestShiftTargetAndSourceSame3 {}

impl Test for SRLVOpcodeTestShiftTargetAndSourceSame3 {
    fn name(&self) -> &str { "SRLVOpcodeTestShiftTargetAndSourceSame3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x1;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestShiftTargetAndSourceSame4 {}

impl Test for SRLVOpcodeTestShiftTargetAndSourceSame4 {
    fn name(&self) -> &str { "SRLVOpcodeTestShiftTargetAndSourceSame4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0xffffffff;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x1, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestShiftTooLarge {}

impl Test for SRLVOpcodeTestShiftTooLarge {
    fn name(&self) -> &str { "SRLVOpcodeTestShiftTooLarge" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x24;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x24, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xf321432, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestIntoItself {}

impl Test for SRLVOpcodeTestIntoItself {
    fn name(&self) -> &str { "SRLVOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x6;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x6, "Register $2")?;
        soft_assert_eq(r3, 0x10c850c, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestIntoItself0 {}

impl Test for SRLVOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "SRLVOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0xffffffffc3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestIntoR0Shift0 {}

impl Test for SRLVOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "SRLVOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $0, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestFromR0 {}

impl Test for SRLVOpcodeTestFromR0 {
    fn name(&self) -> &str { "SRLVOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $3, $0, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRLVOpcodeTestFromR0Shift0 {}

impl Test for SRLVOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "SRLVOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRLV $3, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRL32OpcodeTest {}

impl Test for DSRL32OpcodeTest {
    fn name(&self) -> &str { "DSRL32OpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x8321432183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL32 $4, $3, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x8321432183214321, "Register $3")?;
        soft_assert_eq(r4, 0x4190a19, "Register $4")?;
        Ok(())
    }
}

pub struct DSRL32OpcodeTestShift0 {}

impl Test for DSRL32OpcodeTestShift0 {
    fn name(&self) -> &str { "DSRL32OpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x83214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL32 $4, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x83214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0x83214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSRL32OpcodeTestIntoItself {}

impl Test for DSRL32OpcodeTestIntoItself {
    fn name(&self) -> &str { "DSRL32OpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x8321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL32 $3, $3, 6
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x20c850c, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRL32OpcodeTestIntoItself0 {}

impl Test for DSRL32OpcodeTestIntoItself0 {
    fn name(&self) -> &str { "DSRL32OpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL32 $3, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRL32OpcodeTestIntoR0Shift0 {}

impl Test for DSRL32OpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "DSRL32OpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL32 $0, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRL32OpcodeTestFromR0 {}

impl Test for DSRL32OpcodeTestFromR0 {
    fn name(&self) -> &str { "DSRL32OpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL32 $3, $0, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRL32OpcodeTestFromR0Shift0 {}

impl Test for DSRL32OpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "DSRL32OpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL32 $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLOpcodeTest {}

impl Test for DSRLOpcodeTest {
    fn name(&self) -> &str { "DSRLOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x8321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL $4, $3, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x8321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4190a190a190a19, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLOpcodeTestShift0 {}

impl Test for DSRLOpcodeTestShift0 {
    fn name(&self) -> &str { "DSRLOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL $4, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLOpcodeTestIntoItself {}

impl Test for DSRLOpcodeTestIntoItself {
    fn name(&self) -> &str { "DSRLOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL $3, $3, 6
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x10c850c850c850c, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLOpcodeTestIntoItself0 {}

impl Test for DSRLOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "DSRLOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL $3, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLOpcodeTestIntoR0Shift0 {}

impl Test for DSRLOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "DSRLOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL $0, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLOpcodeTestFromR0 {}

impl Test for DSRLOpcodeTestFromR0 {
    fn name(&self) -> &str { "DSRLOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL $3, $0, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLOpcodeTestFromR0Shift0 {}

impl Test for DSRLOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "DSRLOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRL $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTest {}

impl Test for DSRLVOpcodeTest {
    fn name(&self) -> &str { "DSRLVOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x8321432183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x8321432183214321, "Register $3")?;
        soft_assert_eq(r4, 0x4190a190c190a19, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestShift0 {}

impl Test for DSRLVOpcodeTestShift0 {
    fn name(&self) -> &str { "DSRLVOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x83214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x83214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0x83214321f3214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestShift1 {}

impl Test for DSRLVOpcodeTestShift1 {
    fn name(&self) -> &str { "DSRLVOpcodeTestShift1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x24;
        let mut r3: u64 = 0x83214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x24, "Register $2")?;
        soft_assert_eq(r3, 0x83214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0x8321432, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestShiftTargetAndSourceSame1 {}

impl Test for DSRLVOpcodeTestShiftTargetAndSourceSame1 {
    fn name(&self) -> &str { "DSRLVOpcodeTestShiftTargetAndSourceSame1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x10;
        let mut r4: u64 = 0x3;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x10, "Register $3")?;
        soft_assert_eq(r4, 0x2, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestShiftTargetAndSourceSame2 {}

impl Test for DSRLVOpcodeTestShiftTargetAndSourceSame2 {
    fn name(&self) -> &str { "DSRLVOpcodeTestShiftTargetAndSourceSame2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0xc;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $4, $4, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x3, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestShiftTargetAndSourceSame3 {}

impl Test for DSRLVOpcodeTestShiftTargetAndSourceSame3 {
    fn name(&self) -> &str { "DSRLVOpcodeTestShiftTargetAndSourceSame3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x1;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestShiftTargetAndSourceSame4 {}

impl Test for DSRLVOpcodeTestShiftTargetAndSourceSame4 {
    fn name(&self) -> &str { "DSRLVOpcodeTestShiftTargetAndSourceSame4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0xffffffffffffffff;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x1, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestShiftTooLarge {}

impl Test for DSRLVOpcodeTestShiftTooLarge {
    fn name(&self) -> &str { "DSRLVOpcodeTestShiftTooLarge" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x44;
        let mut r3: u64 = 0x83214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x44, "Register $2")?;
        soft_assert_eq(r3, 0x83214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0x83214321f321432, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestIntoItself {}

impl Test for DSRLVOpcodeTestIntoItself {
    fn name(&self) -> &str { "DSRLVOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x6;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x6, "Register $2")?;
        soft_assert_eq(r3, 0x10c850c850c850c, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestIntoItself0 {}

impl Test for DSRLVOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "DSRLVOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestIntoR0Shift0 {}

impl Test for DSRLVOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "DSRLVOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $0, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestFromR0 {}

impl Test for DSRLVOpcodeTestFromR0 {
    fn name(&self) -> &str { "DSRLVOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $3, $0, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRLVOpcodeTestFromR0Shift0 {}

impl Test for DSRLVOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "DSRLVOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRLV $3, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRAOpcodeTest {}

impl Test for SRAOpcodeTest {
    fn name(&self) -> &str { "SRAOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRA $4, $3, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432183214321, "Register $3")?;
        soft_assert_eq(r4, 0xc190a19, "Register $4")?;
        Ok(())
    }
}

pub struct SRAOpcodeTest2 {}

impl Test for SRAOpcodeTest2 {
    fn name(&self) -> &str { "SRAOpcodeTest2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321932183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRA $4, $3, 16
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321932183214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffff93218321, "Register $4")?;
        Ok(())
    }
}

pub struct SRAOpcodeTestShift0 {}

impl Test for SRAOpcodeTestShift0 {
    fn name(&self) -> &str { "SRAOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRA $4, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xfffffffff3214321, "Register $4")?;
        Ok(())
    }
}

pub struct SRAOpcodeTestIntoItself {}

impl Test for SRAOpcodeTestIntoItself {
    fn name(&self) -> &str { "SRAOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRA $3, $3, 6
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xffffffff850c850c, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRAOpcodeTestIntoItself0 {}

impl Test for SRAOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "SRAOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRA $3, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xffffffffc3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRAOpcodeTestIntoR0Shift0 {}

impl Test for SRAOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "SRAOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRA $0, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRAOpcodeTestFromR0 {}

impl Test for SRAOpcodeTestFromR0 {
    fn name(&self) -> &str { "SRAOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRA $3, $0, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRAOpcodeTestFromR0Shift0 {}

impl Test for SRAOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "SRAOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRA $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTest {}

impl Test for SRAVOpcodeTest {
    fn name(&self) -> &str { "SRAVOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x4321432183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x4321432183214321, "Register $3")?;
        soft_assert_eq(r4, 0xc190a19, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTest2 {}

impl Test for SRAVOpcodeTest2 {
    fn name(&self) -> &str { "SRAVOpcodeTest2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x10;
        let mut r3: u64 = 0x4321932183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x10, "Register $2")?;
        soft_assert_eq(r3, 0x4321932183214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffff93218321, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestShift0 {}

impl Test for SRAVOpcodeTestShift0 {
    fn name(&self) -> &str { "SRAVOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xfffffffff3214321, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestShiftTargetAndSourceSame1 {}

impl Test for SRAVOpcodeTestShiftTargetAndSourceSame1 {
    fn name(&self) -> &str { "SRAVOpcodeTestShiftTargetAndSourceSame1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x10;
        let mut r4: u64 = 0x3;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x10, "Register $3")?;
        soft_assert_eq(r4, 0x2, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestShiftTargetAndSourceSame2 {}

impl Test for SRAVOpcodeTestShiftTargetAndSourceSame2 {
    fn name(&self) -> &str { "SRAVOpcodeTestShiftTargetAndSourceSame2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0xc;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $4, $4, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x3, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestShiftTargetAndSourceSame3 {}

impl Test for SRAVOpcodeTestShiftTargetAndSourceSame3 {
    fn name(&self) -> &str { "SRAVOpcodeTestShiftTargetAndSourceSame3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x1;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestShiftTargetAndSourceSame4 {}

impl Test for SRAVOpcodeTestShiftTargetAndSourceSame4 {
    fn name(&self) -> &str { "SRAVOpcodeTestShiftTargetAndSourceSame4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0xfffff123812345ff;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffffffe247, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestShiftTooLarge {}

impl Test for SRAVOpcodeTestShiftTooLarge {
    fn name(&self) -> &str { "SRAVOpcodeTestShiftTooLarge" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x24;
        let mut r3: u64 = 0x43214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x24, "Register $2")?;
        soft_assert_eq(r3, 0x43214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0x1f321432, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestIntoItself {}

impl Test for SRAVOpcodeTestIntoItself {
    fn name(&self) -> &str { "SRAVOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x6;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x6, "Register $2")?;
        soft_assert_eq(r3, 0xffffffff850c850c, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestIntoItself0 {}

impl Test for SRAVOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "SRAVOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0xffffffffc3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestIntoR0Shift0 {}

impl Test for SRAVOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "SRAVOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $0, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestFromR0 {}

impl Test for SRAVOpcodeTestFromR0 {
    fn name(&self) -> &str { "SRAVOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $3, $0, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct SRAVOpcodeTestFromR0Shift0 {}

impl Test for SRAVOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "SRAVOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                SRAV $3, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRA32OpcodeTest {}

impl Test for DSRA32OpcodeTest {
    fn name(&self) -> &str { "DSRA32OpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x8321432183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA32 $4, $3, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x8321432183214321, "Register $3")?;
        soft_assert_eq(r4, 0xfffffffffc190a19, "Register $4")?;
        Ok(())
    }
}

pub struct DSRA32OpcodeTestShift0 {}

impl Test for DSRA32OpcodeTestShift0 {
    fn name(&self) -> &str { "DSRA32OpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x83214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA32 $4, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x83214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xffffffff83214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSRA32OpcodeTestIntoItself {}

impl Test for DSRA32OpcodeTestIntoItself {
    fn name(&self) -> &str { "DSRA32OpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x8321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA32 $3, $3, 6
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0xfffffffffe0c850c, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRA32OpcodeTestIntoItself0 {}

impl Test for DSRA32OpcodeTestIntoItself0 {
    fn name(&self) -> &str { "DSRA32OpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA32 $3, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRA32OpcodeTestIntoR0Shift0 {}

impl Test for DSRA32OpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "DSRA32OpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA32 $0, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRA32OpcodeTestFromR0 {}

impl Test for DSRA32OpcodeTestFromR0 {
    fn name(&self) -> &str { "DSRA32OpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA32 $3, $0, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRA32OpcodeTestFromR0Shift0 {}

impl Test for DSRA32OpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "DSRA32OpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA32 $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAOpcodeTest {}

impl Test for DSRAOpcodeTest {
    fn name(&self) -> &str { "DSRAOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x8321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA $4, $3, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x8321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0xfc190a190a190a19, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAOpcodeTestShift0 {}

impl Test for DSRAOpcodeTestShift0 {
    fn name(&self) -> &str { "DSRAOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA $4, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x4321432143214321, "Register $3")?;
        soft_assert_eq(r4, 0x4321432143214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAOpcodeTestIntoItself {}

impl Test for DSRAOpcodeTestIntoItself {
    fn name(&self) -> &str { "DSRAOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA $3, $3, 6
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x10c850c850c850c, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAOpcodeTestIntoItself0 {}

impl Test for DSRAOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "DSRAOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA $3, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAOpcodeTestIntoR0Shift0 {}

impl Test for DSRAOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "DSRAOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA $0, $3, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAOpcodeTestFromR0 {}

impl Test for DSRAOpcodeTestFromR0 {
    fn name(&self) -> &str { "DSRAOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA $3, $0, 5
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAOpcodeTestFromR0Shift0 {}

impl Test for DSRAOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "DSRAOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRA $3, $0, 0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTest {}

impl Test for DSRAVOpcodeTest {
    fn name(&self) -> &str { "DSRAVOpcodeTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x8321432183214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x8321432183214321, "Register $3")?;
        soft_assert_eq(r4, 0xfc190a190c190a19, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestShift0 {}

impl Test for DSRAVOpcodeTestShift0 {
    fn name(&self) -> &str { "DSRAVOpcodeTestShift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x83214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x83214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0x83214321f3214321, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestShift1 {}

impl Test for DSRAVOpcodeTestShift1 {
    fn name(&self) -> &str { "DSRAVOpcodeTestShift1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x24;
        let mut r3: u64 = 0x83214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x24, "Register $2")?;
        soft_assert_eq(r3, 0x83214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xfffffffff8321432, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestShiftTargetAndSourceSame1 {}

impl Test for DSRAVOpcodeTestShiftTargetAndSourceSame1 {
    fn name(&self) -> &str { "DSRAVOpcodeTestShiftTargetAndSourceSame1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x10;
        let mut r4: u64 = 0x3;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $4, $3, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x10, "Register $3")?;
        soft_assert_eq(r4, 0x2, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestShiftTargetAndSourceSame2 {}

impl Test for DSRAVOpcodeTestShiftTargetAndSourceSame2 {
    fn name(&self) -> &str { "DSRAVOpcodeTestShiftTargetAndSourceSame2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0xc;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $4, $4, $3
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x3, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestShiftTargetAndSourceSame3 {}

impl Test for DSRAVOpcodeTestShiftTargetAndSourceSame3 {
    fn name(&self) -> &str { "DSRAVOpcodeTestShiftTargetAndSourceSame3" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x1;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0x0, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestShiftTargetAndSourceSame4 {}

impl Test for DSRAVOpcodeTestShiftTargetAndSourceSame4 {
    fn name(&self) -> &str { "DSRAVOpcodeTestShiftTargetAndSourceSame4" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x2;
        let mut r4: u64 = 0x82345678001122ff;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $4, $4, $4
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x2, "Register $3")?;
        soft_assert_eq(r4, 0xffffffffffffffff, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestShiftTooLarge {}

impl Test for DSRAVOpcodeTestShiftTooLarge {
    fn name(&self) -> &str { "DSRAVOpcodeTestShiftTooLarge" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x44;
        let mut r3: u64 = 0x83214321f3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $4, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x44, "Register $2")?;
        soft_assert_eq(r3, 0x83214321f3214321, "Register $3")?;
        soft_assert_eq(r4, 0xf83214321f321432, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestIntoItself {}

impl Test for DSRAVOpcodeTestIntoItself {
    fn name(&self) -> &str { "DSRAVOpcodeTestIntoItself" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x6;
        let mut r3: u64 = 0x4321432143214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x6, "Register $2")?;
        soft_assert_eq(r3, 0x10c850c850c850c, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestIntoItself0 {}

impl Test for DSRAVOpcodeTestIntoItself0 {
    fn name(&self) -> &str { "DSRAVOpcodeTestIntoItself0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $3, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestIntoR0Shift0 {}

impl Test for DSRAVOpcodeTestIntoR0Shift0 {
    fn name(&self) -> &str { "DSRAVOpcodeTestIntoR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x0;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $0, $3, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x0, "Register $2")?;
        soft_assert_eq(r3, 0x43214321c3214321, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestFromR0 {}

impl Test for DSRAVOpcodeTestFromR0 {
    fn name(&self) -> &str { "DSRAVOpcodeTestFromR0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x5;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $3, $0, $2
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x5, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}

pub struct DSRAVOpcodeTestFromR0Shift0 {}

impl Test for DSRAVOpcodeTestFromR0Shift0 {
    fn name(&self) -> &str { "DSRAVOpcodeTestFromR0Shift0" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { vec! {} }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut r0: u64 = 0;
        let mut r2: u64 = 0x1234567898760110;
        let mut r3: u64 = 0x43214321c3214321;
        let mut r4: u64 = 0xabcdefabcdefabcd;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $2, 0($18)
                LD $3, 0($19)
                LD $4, 0($20)
                DSRAV $3, $0, $0
                SD $0, 0($16)
                SD $2, 0($18)
                SD $3, 0($19)
                SD $4, 0($20)
            ", in("$16") &mut r0, out("$2") _, in("$18") &mut r2, out("$3") _, in("$19") &mut r3, out("$4") _, in("$20") &mut r4);
        }
        soft_assert_eq(r0, 0x0, "Register $0")?;
        soft_assert_eq(r2, 0x1234567898760110, "Register $2")?;
        soft_assert_eq(r3, 0x0, "Register $3")?;
        soft_assert_eq(r4, 0xabcdefabcdefabcd, "Register $4")?;
        Ok(())
    }
}
