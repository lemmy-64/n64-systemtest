use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::cop0;
use crate::cop0::CauseException;
use crate::exception_handler::expect_exception;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct LWAddressNotSignExtended {}

impl Test for LWAddressNotSignExtended {
    fn name(&self) -> &str { "LW with address not sign extended (64 bit)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Enable 64 bit kernel addressing mode
        unsafe { crate::cop0::set_status(0x240000E0); }

        // Load from 0x00000000_80201234 causes TLBL, as upper bits are 0
        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::TLBL, 1, || {
            unsafe {
                asm!("
                    .set noat
                    LUI $2, 0x8020
                    ORI $2, $2, 0x1234
                    // zero out upper bits
                    DSLL32 $2, $2, 0
                    DSRL32 $2, $2, 0
                    LW $0, 0($2)
                ", out("$2") _)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000080, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x8C400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, 0x00000000_80201234, "BadVAddr")?;
        soft_assert_eq(exception_context.cause, 0x8, "Cause")?;
        soft_assert_eq(exception_context.status, 0x240000E2, "Status")?;
        soft_assert_eq(exception_context.context, 0x401000, "Context")?;
        soft_assert_eq(exception_context.xcontext, 0x401000, "XContext")?;

        Ok(())
    }
}

pub struct SWAddressNotSignExtended {}

impl Test for SWAddressNotSignExtended {
    fn name(&self) -> &str { "SW with address not sign extended (64 bit)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Enable 64 bit kernel addressing mode
        unsafe { crate::cop0::set_status(0x240000E0); }

        // Store to 0x00000000_80201234 causes TLBS, as upper bits are 0
        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::TLBS, 1, || {
            unsafe {
                asm!("
                    .set noat
                    LUI $2, 0x8020
                    ORI $2, $2, 0x1234
                    // zero out upper bits
                    DSLL32 $2, $2, 0
                    DSRL32 $2, $2, 0
                    SW $0, 0($2)
                ", out("$2") _)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000080, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0xAC400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, 0x00000000_80201234, "BadVAddr")?;
        soft_assert_eq(exception_context.cause, 0xC, "Cause")?;
        soft_assert_eq(exception_context.status, 0x240000E2, "Status")?;
        soft_assert_eq(exception_context.context, 0x401000, "Context")?;
        soft_assert_eq(exception_context.xcontext, 0x401000, "XContext")?;

        Ok(())
    }
}

pub struct LWTLBMissTest {}

impl Test for LWTLBMissTest {
    fn name(&self) -> &str { "LW tlb miss test (64 bit)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Enable 64 bit kernel addressing mode
        unsafe { crate::cop0::set_status(0x240000E0); }

        // Load from 0x00000000_00201234 causes TLBL, as upper bits are 0
        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::TLBL, 1, || {
            unsafe {
                asm!("
                    .set noat
                    LUI $2, 0x0020
                    ORI $2, $2, 0x1234
                    LW $0, 0($2)
                ", out("$2") _)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000080, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x8C400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, 0x00000000_00201234, "BadVAddr")?;
        soft_assert_eq(exception_context.cause, 0x8, "Cause")?;
        soft_assert_eq(exception_context.status, 0x240000E2, "Status")?;
        soft_assert_eq(exception_context.context, 0x1000, "Context")?;
        soft_assert_eq(exception_context.xcontext, 0x1000, "XContext")?;

        Ok(())
    }
}
