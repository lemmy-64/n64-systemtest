use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use crate::cop0::CauseException;
use crate::exception_handler::expect_exception;

use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct Break {}

impl Test for Break {
    fn name(&self) -> &str { "Break" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Bp, 1, || {
            unsafe {
                asm!("
                    .set noat
                    BREAK 0x319
                ")
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(((unsafe { *(exception_context.exceptpc as *const u32) }) >> 16) & 0x3FF, 0x319, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x24, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct BreakDelay {}

impl Test for BreakDelay {
    fn name(&self) -> &str { "Break (delay slot)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Bp, 2, || {
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    BEQ $0, $0, 2f
                    BREAK 0x319
                    2:
                    NOP
                    NOP
                ")
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(((unsafe { *(exception_context.exceptpc as *const u32).add(1) }) >> 16) & 0x3FF, 0x319, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x80000024, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct Syscall {}

impl Test for Syscall {
    fn name(&self) -> &str { "Syscall" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Sys, 1, || {
            unsafe {
                asm!("
                    .set noat
                    SYSCALL 0xF123F
                ")
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(((unsafe { *(exception_context.exceptpc as *const u32) }) >> 6) & 0xFFFFF, 0xF123F, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x20, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct SyscallDelay {}

impl Test for SyscallDelay {
    fn name(&self) -> &str { "Syscall (delay slot)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::Sys, 2, || {
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    BNE $0, $0, 2f
                    SYSCALL 0xF123F
                    2:
                    NOP
                    NOP
                ")
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(((unsafe { *(exception_context.exceptpc as *const u32).add(1) }) >> 6) & 0xFFFFF, 0xF123F, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x80000020, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

/// Instruction 31 doesn't exist. If it is called (Linux calls it for example), we expect a Reserved-Instruction exception
pub struct Reserved31 {}

impl Test for Reserved31 {
    fn name(&self) -> &str { "Reserved (31)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::RI, 1, || {
            unsafe {
                asm!("
                    .set noat
                    .word 0x7C03E83B
                ")
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x7C03E83B, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x28, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}

pub struct Reserved31Delay {}

impl Test for Reserved31Delay {
    fn name(&self) -> &str { "Reserved (31) (delay slot)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::RI, 2, || {
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    BNE $0, $0, 2f
                    .word 0x7C03E83B
                    2:
                    NOP
                    NOP
                ")
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32).add(1) }, 0x7C03E83B, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x80000028, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;

        Ok(())
    }
}
