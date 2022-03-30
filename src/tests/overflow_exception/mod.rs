use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use crate::cop0::CauseException;
use crate::exception_handler::expect_exception;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct AddOverflowPositive {}

impl Test for AddOverflowPositive {
    fn name(&self) -> &str { "ADD (overflow, positive)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32 = 0xBADDECAF;
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0x7FFFFFFF;
            let b: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    ADD $2, $4, $5
                ", in("$4") a, in("$5") b, inout("$2") result)
            }

            Ok(())
        })?;

        soft_assert_eq(result, 0xBADDECAF, "Result should be unchanged on overflow")?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x851020, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct AddOverflowNegative {}

impl Test for AddOverflowNegative {
    fn name(&self) -> &str { "ADD (overflow, negative)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32 = 0xBADDECAF;
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0xE8760110;
            let b: u32 = 0x83214321;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    ADD $2, $4, $5
                ", in("$4") a, in("$5") b, inout("$2") result)
            }

            Ok(())
        })?;

        soft_assert_eq(result, 0xBADDECAF, "Result should be unchanged on overflow")?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x851020, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct AddOverflowIntoR0 {}

impl Test for AddOverflowIntoR0 {
    fn name(&self) -> &str { "ADD (overflow, into R0)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0x7FFFFFFF;
            let b: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    ADD $0, $4, $5
                ", in("$4") a, in("$5") b)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x850020, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct AddOverflowDelaySlot1 {}

impl Test for AddOverflowDelaySlot1 {
    fn name(&self) -> &str { "ADD (overflow in delay, not taken)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32 = 0xBADDECAF;
        let exception_context = expect_exception(CauseException::Ov, 2, || {
            let a: u32 = 0x7FFFFFFF;
            let b: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    BNE $0, $0, 2f
                    ADD $2, $4, $6
                    2:
                    NOP
                    NOP
                ", in("$4") a, in("$6") b, inout("$2") result)
            }

            Ok(())
        })?;

        soft_assert_eq(result, 0xBADDECAF, "Result should be unchanged on overflow")?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32).add(1) }, 0x861020, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x80000030, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct AddOverflowDelaySlot2 {}

impl Test for AddOverflowDelaySlot2 {
    fn name(&self) -> &str { "ADD (overflow in delay, taken)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32 = 0xBADDECAF;
        let exception_context = expect_exception(CauseException::Ov, 2, || {
            let a: u32 = 0x7FFFFFFF;
            let b: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    BEQ $0, $0, 2f
                    ADD $2, $5, $6
                    2:
                    NOP
                    NOP
                ", in("$5") a, in("$6") b, inout("$2") result)
            }

            Ok(())
        })?;

        soft_assert_eq(result, 0xBADDECAF, "Result should be unchanged on overflow")?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32).add(1) }, 0xA61020, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x80000030, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct DoubleAddOverflow {}

impl Test for DoubleAddOverflow {
    fn name(&self) -> &str { "DADD (overflow)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32 = 0xBADDECAF;
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0x7FFFFFFF;
            let b: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    DSLL32 $4, $4, 0
                    DSLL32 $5, $5, 0
                    DADD $2, $4, $5
                ", in("$4") a, in("$5") b, inout("$2") result)
            }

            Ok(())
        })?;

        soft_assert_eq(result, 0xBADDECAF, "Result should be unchanged on overflow")?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x85102C, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct DoubleAddOverflowIntoR0 {}

impl Test for DoubleAddOverflowIntoR0 {
    fn name(&self) -> &str { "DADD (overflow, into R0)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0x7FFFFFFF;
            let b: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    DSLL32 $4, $4, 0
                    DSLL32 $5, $5, 0
                    DADD $0, $4, $5
                ", in("$4") a, in("$5") b)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x85002C, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct SubOverflow {}

impl Test for SubOverflow {
    fn name(&self) -> &str { "SUB (overflow)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32 = 0xBADDECAF;
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0x80000000;
            let b: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    SUB $2, $4, $5
                ", in("$4") a, in("$5") b, inout("$2") result)
            }

            Ok(())
        })?;

        soft_assert_eq(result, 0xBADDECAF, "Result should be unchanged on overflow")?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x851022, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct SubOverflowIntoR0 {}

impl Test for SubOverflowIntoR0 {
    fn name(&self) -> &str { "SUB (overflow, into R0)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0x80000000;
            let b: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    SUB $0, $4, $5
                ", in("$4") a, in("$5") b)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x850022, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct DoubleSubOverflow {}

impl Test for DoubleSubOverflow {
    fn name(&self) -> &str { "DSUB (overflow)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32 = 0xBADDECAF;
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0x80000000;
            let b: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    DSLL32 $4, $4, 0
                    DSLL32 $5, $5, 0
                    DSUB $2, $4, $5
                ", in("$4") a, in("$5") b, inout("$2") result)
            }

            Ok(())
        })?;

        soft_assert_eq(result, 0xBADDECAF, "Result should be unchanged on overflow")?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x85102E, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct DoubleSubOverflowIntoR0 {}

impl Test for DoubleSubOverflowIntoR0 {
    fn name(&self) -> &str { "DSUB (overflow, into R0)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0x80000000;
            let b: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    DSLL32 $4, $4, 0
                    DSLL32 $5, $5, 0
                    DSUB $0, $4, $5
                ", in("$4") a, in("$5") b)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x85002E, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct AddImmediateOverflow {}

impl Test for AddImmediateOverflow {
    fn name(&self) -> &str { "ADDI (overflow)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32 = 0xBADDECAF;
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    ADDI $2, $4, 0x1
                ", in("$4") a, inout("$2") result)
            }

            Ok(())
        })?;

        soft_assert_eq(result, 0xBADDECAF, "Result should be unchanged on overflow")?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x20820001, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct AddImmediateOverflowIntoR0 {}

impl Test for AddImmediateOverflowIntoR0 {
    fn name(&self) -> &str { "ADDI (overflow, into R0)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a: u32 = 0x7FFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    ADDI $0, $4, 0x1
                ", in("$4") a)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x20800001, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct DoubleAddImmediateOverflow {}

impl Test for DoubleAddImmediateOverflow {
    fn name(&self) -> &str { "DADDI (overflow)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32 = 0xBADDECAF;
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a_higher: u32 = 0x7FFFFFFF;
            let a_lower: u32 = 0xFFFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    // Move higher up
                    DSLL32 $7, $4, 0
                    // Remove sign extension from lower, store in 6
                    DSLL32 $6, $5, 0
                    DSRL32 $6, $6, 0
                    // Combine both
                    OR $6, $7, $6
                    // This will now fault
                    DADDI $2, $6, 1
                ", in("$4") a_higher, in("$5") a_lower, out("$6") _, out("$7") _, inout("$2") result)
            }

            Ok(())
        })?;

        soft_assert_eq(result, 0xBADDECAF, "Result should be unchanged on overflow")?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x60C20001, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct DoubleAddImmediateOverflowIntoR0 {}

impl Test for DoubleAddImmediateOverflowIntoR0 {
    fn name(&self) -> &str { "DADDI (overflow, into R0)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Ov, 1, || {
            let a_higher: u32 = 0x7FFFFFFF;
            let a_lower: u32 = 0xFFFFFFFF;
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    // Move higher up
                    DSLL32 $7, $4, 0
                    // Remove sign extension from lower, store in 6
                    DSLL32 $6, $5, 0
                    DSRL32 $6, $6, 0
                    // Combine both
                    OR $6, $7, $6
                    // This will now fault
                    DADDI $0, $6, 1
                ", in("$4") a_higher, in("$5") a_lower, out("$6") _, out("$7") _)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x60C00001, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x30, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

