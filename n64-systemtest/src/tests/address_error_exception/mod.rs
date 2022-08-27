use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use crate::cop0;
use crate::cop0::CauseException;
use crate::exception_handler::expect_exception;
use crate::memory_map::MemoryMap;

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
        // Use a pointer after a HEAP. This way we can guarantee constant pointers (allowing us to verify context, badvaddr against constants below)
        let p = (MemoryMap::HEAP_END_VIRTUAL_CACHED + 12345) as *mut u32;

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
        soft_assert_eq(exception_context.badvaddr, p as isize as u64, "BadVAddr during AdEL exception")?;
        soft_assert_eq(exception_context.cause, 0x10, "Cause during AdEL exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during AdEL exception")?;
        soft_assert_eq(exception_context.context, 0x401410, "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, 0x1_FFC01410, "XContext during AdEL exception")?;

        Ok(())
    }
}

pub struct UnalignedLW2 {}

impl Test for UnalignedLW2 {
    fn name(&self) -> &str { "Unaligned LW exception (with high bits of context set)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Use a pointer after a HEAP. This way we can guarantee constant pointers (allowing us to verify context, badvaddr against constants below)
        let p = (MemoryMap::HEAP_END_VIRTUAL_CACHED + 22345) as *mut u32;

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
        soft_assert_eq(exception_context.cause, 0x10, "Cause during AdEL exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during AdEL exception")?;
        soft_assert_eq(exception_context.context, 0xFFFFFFFF_FFC01420, "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, 0xFFFFFFFF_FFC01420, "XContext during AdEL exception")?;

        Ok(())
    }
}

pub struct UnalignedLWDelay {}

impl Test for UnalignedLWDelay {
    fn name(&self) -> &str { "Unaligned LW exception (delay slot)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Use a pointer after a HEAP. This way we can guarantee constant pointers (allowing us to verify context, badvaddr against constants below)
        let p = (MemoryMap::HEAP_END_VIRTUAL_CACHED + 12345) as *mut u32;

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
        soft_assert_eq(exception_context.cause, 0x80000010, "Cause during AdEL exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during AdEL exception")?;
        soft_assert_eq(exception_context.context, 0x401410, "Context during AdEL exception")?;
        soft_assert_eq(exception_context.xcontext, 0x1_FFC01410, "XContext during AdEL exception")?;

        Ok(())
    }
}

pub struct UnalignedSW {}

impl Test for UnalignedSW {
    fn name(&self) -> &str { "Unaligned SW exception" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p = (MemoryMap::HEAP_END_VIRTUAL_CACHED + 32345) as *mut u32;

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
        soft_assert_eq(exception_context.cause, 0x14, "Cause during AdES exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during AdES exception")?;
        soft_assert_eq(exception_context.context, 0x401430, "Context during AdES exception")?;
        soft_assert_eq(exception_context.xcontext, 0x1ffc01430, "XContext during AdES exception")?;

        Ok(())
    }
}

pub struct LWAddressNotSignExtended {}

impl Test for LWAddressNotSignExtended {
    fn name(&self) -> &str { "LW with address not sign extended" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p = (MemoryMap::HEAP_END_VIRTUAL_CACHED + 0x1234) as *mut u32;
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
        soft_assert_eq(exception_context.cause, 0x10, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;
        soft_assert_eq(exception_context.context, 0x401400, "Context")?;
        soft_assert_eq(exception_context.xcontext, 0x401400, "XContext")?;

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
        let p = (MemoryMap::HEAP_END_VIRTUAL_CACHED + 0x1234) as *mut u32;
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
        soft_assert_eq(exception_context.cause, 0x14, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;
        soft_assert_eq(exception_context.context, 0x401400, "Context")?;
        soft_assert_eq(exception_context.xcontext, 0x401400, "XContext")?;

        Ok(())
    }
}
