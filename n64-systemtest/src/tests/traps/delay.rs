use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use crate::cop0::CauseException;
use crate::exception_handler::expect_exception;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct TNEDelay1 {}

impl Test for TNEDelay1 {
    fn name(&self) -> &str { "TNE (taken in delay slot of not taken branch)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Tr, 2, || {
            unsafe {
                asm!("
                    .set noreorder
                    LI $2, 0x01234567
                    LI $3, 0x01234566
                    BNE $0, $0, 2f
                    TNE $2, $3
                    2:
                    NOP
                    NOP
                ", out("$2") _, out("$3") _)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32).add(1) }, 0x00430036, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x80000034, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct TNEDelay2 {}

impl Test for TNEDelay2 {
    fn name(&self) -> &str { "TNE (taken in delay slot of taken branch)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Tr, 2, || {
            unsafe {
                asm!("
                    .set noreorder
                    LI $2, 0x01234567
                    LI $3, 0x01234566
                    BEQ $0, $0, 2f
                    TNE $2, $3
                    2:
                    NOP
                    NOP
                ", out("$2") _, out("$3") _)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32).add(1) }, 0x00430036, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x80000034, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

