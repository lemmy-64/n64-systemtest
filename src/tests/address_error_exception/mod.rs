use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use arbitrary_int::{u31, u41};
use crate::cop0;
use crate::cop0::{CauseException, Context, XContext};
use crate::exception_handler::expect_exception;

use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// TODO: Less copy&paste below to more easily test more cases
// TODO: Test 16 bit
// TODO: Test 64 bit
// TODO: Test unsigned
// TODO: Test delay slot (in that case make sure to skip 2 instructions in the exception handler)
// TODO: Test nested exceptions (address_error_exception access while exception is already happening)
// TODO: Test 64 bit address mode. Some 64 bit addresses are valid, others aren't

pub struct UnalignedLW {}

impl Test for UnalignedLW {
    fn name(&self) -> &str { "Unaligned LW exception" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let a = 0x12345678u32;
        // Make unaligned pointer
        let p = &a as *const u32 as isize + 2;

        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::AdEL, 1, || {
            unsafe {
                asm!("
                    .set noat
                    LW $0, 0({gpr})
                ", gpr = in(reg) p)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x8C600000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, p as u64, "BadVAddr during AdEL exception")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x10, "Cause during AdEL exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during AdEL exception")?;
        soft_assert_eq(exception_context.context, Context::from_virtual_address(p as u64), "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, XContext::from_virtual_address(p as u64), "XContext during AdEL exception")?;

        Ok(())
    }
}

pub struct UnalignedLW2 {}

impl Test for UnalignedLW2 {
    fn name(&self) -> &str { "Unaligned LW exception (with high bits of context set)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let a = 0x12345678u32;
        // Make unaligned pointer
        let p = &a as *const u32 as isize + 3;

        unsafe { cop0::set_context_64(0xFFFFFFFF_FFFFFFFF); }
        unsafe { cop0::set_xcontext_64(0xFFFFFFFF_FFFFFFFF); }
        let exception_context = expect_exception(CauseException::AdEL, 1, || {
            unsafe {
                asm!("
                    .set noat
                    LW $0, 0($2)
                ", in("$2") p)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x8C400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, p as isize as u64, "BadVAddr during AdEL exception")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x10, "Cause during AdEL exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during AdEL exception")?;
        soft_assert_eq(exception_context.context, Context::from_virtual_address(p as u64).with_pte_base(u41::MAX), "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, XContext::from_virtual_address(p as u64).with_pte_base(u31::MAX), "XContext during AdEL exception")?;

        Ok(())
    }
}

pub struct UnalignedLWDelay {}

impl Test for UnalignedLWDelay {
    fn name(&self) -> &str { "Unaligned LW exception (delay slot)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let a = 0x12345678u32;
        // Make unaligned pointer
        let p = &a as *const u32 as isize + 1;

        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::AdEL, 2, || {
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    BNE $1, $1, 2f
                    LW $0, 0($2)
                    2:
                    NOP
                    NOP
                ", in("$2") p)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32).add(1) }, 0x8C400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, p as isize as u64, "BadVAddr during AdEL exception")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x80000010, "Cause during AdEL exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during AdEL exception")?;
        soft_assert_eq(exception_context.context, Context::from_virtual_address(p as u64), "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, XContext::from_virtual_address(p as u64), "XContext during AdEL exception")?;

        Ok(())
    }
}

pub struct UnalignedSW {}

impl Test for UnalignedSW {
    fn name(&self) -> &str { "Unaligned SW exception" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let a = 0x12345678u32;
        // Make unaligned pointer
        let p = &a as *const u32 as isize + 2;

        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::AdES, 1, || {
            unsafe {
                asm!("
                    .set noat
                    SW $0, 0($2)
                ", in("$2") p)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0xAC400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, p as isize as u64, "BadVAddr during AdES exception")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x14, "Cause during AdES exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during AdES exception")?;
        soft_assert_eq(exception_context.context, Context::from_virtual_address(p as u64), "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, XContext::from_virtual_address(p as u64), "XContext during AdEL exception")?;

        Ok(())
    }
}

pub struct UnalignedJump {}

impl Test for UnalignedJump {
    fn name(&self) -> &str { "Unaligned jump exception" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut addr = 0u32;
        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::AdEL, 0, || {
            unsafe {
                asm!("
                    .set noat
                    LA $2, 1f
                    ADDIU $2, 2
                    JR $2
                1:
                    ADDIU $0, $0, 0x1234
                    NOP
                ", out("$2") addr);
            }

            Ok(())
        })?;

        let addr64 = addr as i32 as u64;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc, addr64, "ExceptPC")?;
        soft_assert_eq(exception_context.badvaddr, addr64, "BadVAddr during AdEL exception")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x10, "Cause during AdEL exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during AdEL exception")?;
        soft_assert_eq(exception_context.context, Context::from_virtual_address(addr64), "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, XContext::from_virtual_address(addr64), "XContext during AdEL exception")?;

        Ok(())
    }
}

pub struct UnalignedJumpWithDelaySlot {}

impl Test for UnalignedJumpWithDelaySlot {
    fn name(&self) -> &str { "Unaligned jump exception with delay slot" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut addr = 0u32;
        let mut val = 0u32;
        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::AdEL, 0, || {
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    ORI $3, $0, 0x01
                    LA $2, 1f
                    ADDIU $2, 1
                    JR $2
                    ORI $3, $3, 0x02
                1:
                    ORI $3, $3, 0x04
                ", out("$2") addr,
                   out("$3") val);
            }

            Ok(())
        })?;

        let addr64 = addr as i32 as u64;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc, addr64, "ExceptPC")?;
        soft_assert_eq(exception_context.badvaddr, addr64, "BadVAddr during AdEL exception")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x10, "Cause during AdEL exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during AdEL exception")?;
        soft_assert_eq(exception_context.context, Context::from_virtual_address(addr64), "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, XContext::from_virtual_address(addr64), "XContext during AdEL exception")?;
        soft_assert_eq(val, 7, "delay slot execution during exception")?;

        Ok(())
    }
}


pub struct LWAddressNotSignExtended {}

impl Test for LWAddressNotSignExtended {
    fn name(&self) -> &str { "LW with address not sign extended" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let a = 0x12345678u32;
        let p = &a as *const u32 as u32;
        // Load from 0x00000000_80xxxxxx causes AdEL, as upper bits are 0
        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::AdEL, 1, || {
            unsafe {
                asm!("
                    .set noat
                    // zero out upper bits
                    DSLL32 $2, $2, 0
                    DSRL32 $2, $2, 0
                    LW $0, 0($2)
                ", inout("$2") p => _)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x8C400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, p as u64 & 0xFFFFFFFF, "BadVAddr")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x10, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;
        soft_assert_eq(exception_context.context, Context::from_virtual_address(p as u64), "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, XContext::from_virtual_address(p as u64), "XContext during AdEL exception")?;

        Ok(())
    }
}

pub struct SWAddressNotSignExtended {}

impl Test for SWAddressNotSignExtended {
    fn name(&self) -> &str { "SW with address not sign extended" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Store into 0x00000000_80xxxxxx causes AdES, as upper bits are 0
        let a = 0x12345678u32;
        let p = &a as *const u32 as u32;
        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::AdES, 1, || {
            unsafe {
                asm!("
                    .set noat
                    // zero out upper bits
                    DSLL32 $2, $2, 0
                    DSRL32 $2, $2, 0
                    SW $0, 0($2)
                ", inout("$2") p => _)
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0xAC400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, p as u64 & 0xFFFFFFFF, "BadVAddr")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x14, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;
        soft_assert_eq(exception_context.context, Context::from_virtual_address(p as u64), "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, XContext::from_virtual_address(p as u64), "XContext during AdEL exception")?;

        Ok(())
    }
}
