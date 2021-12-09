use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::{cop0, MemoryMap};
use crate::cop0::{CauseException, make_entry_hi, make_entry_lo};
use crate::exception_handler::expect_exception;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub fn test_miss_exception<F>(pagemask: u32, offset: u32, valid: bool, dirty: bool, maybe_exception_context: Option<(CauseException, bool)>, f: F) -> Result<(), String>
    where F: FnOnce(u32) {
    unsafe { cop0::clear_tlb(); }
    unsafe { cop0::set_context_64(0); }
    unsafe { cop0::set_xcontext_64(0); }

    let virtual_page_base = 0x012345678 & !0b1111111111111 & !pagemask;

    // Setup 16k mapping from 0x0DEA0000 to MemoryMap::HEAP_END
    unsafe {
        cop0::write_tlb(
            10,
            pagemask,
            make_entry_lo(true, valid, dirty, 0, (MemoryMap::HEAP_END >> 12) as u32),
            make_entry_lo(true, false, false, 0, 0),
            make_entry_hi(2, virtual_page_base >> 13));

        // Change EntryHi to confirm it gets set for the exception handler
        cop0::set_entry_hi(make_entry_hi(1, 0));
    }

    match maybe_exception_context {
        None => {
            f(virtual_page_base + offset);
        }
        Some((code, check_entry_hi)) => {
            let exception_context = expect_exception(code, 1, || {
                f(virtual_page_base + offset);
                Ok(())
            })?;

            let expected_context = (((virtual_page_base + offset) >> 13) << 4) as u64;
            soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector for TLB exception")?;
            // Testing for the exact ExceptPC is difficult as we can't easily find out the PC of the LW above. Test ballpark and sign-extension at least
            soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC during TLB exception")?;
            soft_assert_eq(exception_context.badvaddr, (virtual_page_base + offset) as u64, "BadVAddr during TLB exception")?;
            soft_assert_eq(exception_context.cause, (code as u32) << 2, "Cause during TLB exception")?;
            soft_assert_eq(exception_context.status, 0x24000002, "Status during TLB exception")?;
            soft_assert_eq(exception_context.context, expected_context, "Context during TLB exception")?;
            soft_assert_eq(exception_context.xcontext, expected_context, "XContext during TLB exception")?;
            if check_entry_hi {
                // The docs say that asid on exception is the asid of the TLB entry, but test don't confirm that. It seems to stay unchanged
                soft_assert_eq(exception_context.entry_hi, make_entry_hi(1, virtual_page_base >> 13), "EntryHi during TLB exception")?;
            }
        }
    }

    Ok(())
}

pub struct ReadMiss4k {}

impl Test for ReadMiss4k {
    fn name(&self) -> &str { "TLB: Read after 4k page, expect TLBL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0; // 4k
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; };
        test_miss_exception(pagemask, 4092, true, true, None, f)?;
        test_miss_exception(pagemask, 4096, true, true, Some((CauseException::TLBL, false)), f)?;
        Ok(())
    }
}

pub struct ReadMiss16k {}

impl Test for ReadMiss16k {
    fn name(&self) -> &str { "TLB: Read after 16k page, expect TLBL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0b11 << 13; // 16k
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; };
        test_miss_exception(pagemask, 16380, true, true, None, f)?;
        test_miss_exception(pagemask, 16384, true, true, Some((CauseException::TLBL, false)), f)?;
        Ok(())
    }
}

pub struct ReadMiss64k {}

impl Test for ReadMiss64k {
    fn name(&self) -> &str { "TLB: Read after 64k page, expect TLBL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0b1111 << 13; // 64k
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; };
        test_miss_exception(pagemask, 65532, true, true, None, f)?;
        test_miss_exception(pagemask, 65536, true, true, Some((CauseException::TLBL, false)), f)?;
        Ok(())
    }
}

pub struct ReadMiss256k {}

impl Test for ReadMiss256k {
    fn name(&self) -> &str { "TLB: Read after 256k page, expect TLBL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0b111111 << 13; // 256k
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; };
        test_miss_exception(pagemask, 262140, true, true, None, f)?;
        test_miss_exception(pagemask, 262144, true, true, Some((CauseException::TLBL, false)), f)?;
        Ok(())
    }
}

pub struct ReadMiss1M {}

impl Test for ReadMiss1M {
    fn name(&self) -> &str { "TLB: Read after 1M page, expect TLBL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0b11111111 << 13; // 1M
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; };
        test_miss_exception(pagemask, 1048572, true, true, None, f)?;
        test_miss_exception(pagemask, 1048576, true, true, Some((CauseException::TLBL, false)), f)?;
        Ok(())
    }
}

pub struct ReadMiss4M {}

impl Test for ReadMiss4M {
    fn name(&self) -> &str { "TLB: Read after 4M page, expect TLBL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0b1111111111 << 13; // 4M
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; };
        // This will exceed RDRAM (unless there's an expansion pack), but that doesn't cause an error
        test_miss_exception(pagemask, 4194300, true, true, None, f)?;
        test_miss_exception(pagemask, 4194304, true, true, Some((CauseException::TLBL, false)), f)?;
        Ok(())
    }
}

pub struct ReadMiss16M {}

impl Test for ReadMiss16M {
    fn name(&self) -> &str { "TLB: Read after 16M page, expect TLBL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0b111111111111 << 13; // 16M
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; };
        // This will exceed RDRAM, but that doesn't cause an error
        test_miss_exception(pagemask, 16777212, true, true, None, f)?;
        test_miss_exception(pagemask, 16777216, true, true, Some((CauseException::TLBL, false)), f)?;
        Ok(())
    }
}

pub struct StoreMiss4k {}

impl Test for StoreMiss4k {
    fn name(&self) -> &str { "TLB: Store after 4k page, expect TLBS" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0; // 4k
        let f = |address| { unsafe { (address as *mut u32).write_volatile(0) }; };
        test_miss_exception(pagemask, 4092, true, true, None, f)?;
        test_miss_exception(pagemask, 4096, true, true, Some((CauseException::TLBS, false)), f)?;
        Ok(())
    }
}

pub struct ReadNonValid4k {}

impl Test for ReadNonValid4k {
    fn name(&self) -> &str { "TLB: Read inside 4k page which isn't valid, expect TLBL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0; // 4k
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; };
        test_miss_exception(pagemask, 4092, false, true, Some((CauseException::TLBL, true)), f)?;
        Ok(())
    }
}

pub struct StoreNonValid4k {}

impl Test for StoreNonValid4k {
    fn name(&self) -> &str { "TLB: Store inside 4k page which isn't valid, expect TLBS" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0; // 4k
        let f = |address| { unsafe { (address as *mut u32).write_volatile(0) }; };
        test_miss_exception(pagemask, 4092, false, true, Some((CauseException::TLBS, true)), f)?;
        Ok(())
    }
}

pub struct StoreNonDirty4k {}

impl Test for StoreNonDirty4k {
    fn name(&self) -> &str { "TLB: Store inside 4k page which isn't dirty, expect Mod" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0; // 4k
        let f = |address| { unsafe { (address as *mut u32).write_volatile(0) }; };
        test_miss_exception(pagemask, 4092, true, false, Some((CauseException::Mod, true)), f)?;
        Ok(())
    }
}

pub struct StoreNonDirtyAndNonValid4k {}

impl Test for StoreNonDirtyAndNonValid4k {
    fn name(&self) -> &str { "TLB: Store inside 4k page which isn't dirty and not valid. TLBS wins over Mod" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0; // 4k
        let f = |address| { unsafe { (address as *mut u32).write_volatile(0) }; };
        test_miss_exception(pagemask, 4092, false, false, Some((CauseException::TLBS, true)), f)?;
        Ok(())
    }
}

pub struct LWTLBMissTest32 {}

impl Test for LWTLBMissTest32 {
    fn name(&self) -> &str { "LW tlb miss test (32 bit)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Enable 64 bit kernel addressing mode
        unsafe { crate::cop0::set_status(0x24000000); }

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

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000000, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x8C400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, 0x00000000_00201234, "BadVAddr")?;
        soft_assert_eq(exception_context.cause, 0x8, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;
        soft_assert_eq(exception_context.context, 0x1000, "Context")?;
        soft_assert_eq(exception_context.xcontext, 0x1000, "XContext")?;

        Ok(())
    }
}

pub struct LWTLBMissTest64 {}

impl Test for LWTLBMissTest64 {
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

pub struct LWAddressNotSignExtended64 {}

impl Test for LWAddressNotSignExtended64 {
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

pub struct SWAddressNotSignExtended64 {}

impl Test for SWAddressNotSignExtended64 {
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
