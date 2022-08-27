use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use arbitrary_int::{u2, u27};

use crate::{cop0, MemoryMap, println};
use crate::cop0::{CauseException, make_entry_hi, make_entry_lo};
use crate::exception_handler::expect_exception;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub fn setup_tlb_page(pagemask: u32, valid: bool, dirty: bool) -> u32 {
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
            make_entry_hi(2, u27::extract_u64(virtual_page_base as u64, 13), u2::new(0)));

        // Change EntryHi to confirm it gets set for the exception handler
        cop0::set_entry_hi(make_entry_hi(1, u27::new(0), u2::new(0)));
    }

    virtual_page_base
}

pub fn test_miss_exception<F>(pagemask: u32, offset: u32, valid: bool, dirty: bool, skip_instructions: u64, code: CauseException, check_entry_hi: bool, delay: bool, f: F) -> Result<(), String>
    where F: FnOnce(u32) -> Result<(), String> {

    let virtual_page_base = setup_tlb_page(pagemask, valid, dirty);

    let exception_context = expect_exception(code, skip_instructions, || {
        let result = f(virtual_page_base + offset);
        if result.is_err() {
            println!("Error: {}", result.err().unwrap());
            return Err("Error during exception handling");
        }

        Ok(())
    })?;

    let expected_context = (((virtual_page_base + offset) >> 13) << 4) as u64;
    soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector for TLB exception")?;
    // Testing for the exact ExceptPC is difficult as we can't easily find out the PC of the LW above. Test ballpark and sign-extension at least
    soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC during TLB exception")?;
    soft_assert_eq(exception_context.badvaddr, (virtual_page_base + offset) as u64, "BadVAddr during TLB exception")?;
    soft_assert_eq(exception_context.cause, (if delay { 0x80000000 } else { 0 }) | (code as u32) << 2, "Cause during TLB exception")?;
    soft_assert_eq(exception_context.status, 0x24000002, "Status during TLB exception")?;
    soft_assert_eq(exception_context.context, expected_context, "Context during TLB exception")?;
    soft_assert_eq(exception_context.xcontext, expected_context, "XContext during TLB exception")?;
    if check_entry_hi {
        // The docs say that asid on exception is the asid of the TLB entry, but test don't confirm that. It seems to stay unchanged
        soft_assert_eq(exception_context.entry_hi, make_entry_hi(1, u27::extract_u64(virtual_page_base as u32 as u64, 13), u2::new(0)), "EntryHi during TLB exception")?;
    }

    Ok(())
}

pub fn test_nomiss_exception<F>(pagemask: u32, offset: u32, valid: bool, dirty: bool, f: F) -> Result<(), String>
    where F: FnOnce(u32) -> Result<(), String> {

    let virtual_page_base = setup_tlb_page(pagemask, valid, dirty);

    f(virtual_page_base + offset)?;

    Ok(())
}

pub struct ReadMiss4k {}

impl Test for ReadMiss4k {
    fn name(&self) -> &str { "TLB: Read after 4k page, expect TLBL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0; // 4k
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; Ok(()) };
        test_nomiss_exception(pagemask, 4092, true, true, f)?;
        test_miss_exception(pagemask, 4096, true, true, 1, CauseException::TLBL, false, false, f)?;
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
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; Ok(()) };
        test_nomiss_exception(pagemask, 16380, true, true, f)?;
        test_miss_exception(pagemask, 16384, true, true, 1, CauseException::TLBL, false, false, f)?;
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
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; Ok(()) };
        test_nomiss_exception(pagemask, 65532, true, true, f)?;
        test_miss_exception(pagemask, 65536, true, true, 1, CauseException::TLBL, false, false, f)?;
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
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; Ok(()) };
        test_nomiss_exception(pagemask, 262140, true, true, f)?;
        test_miss_exception(pagemask, 262144, true, true, 1, CauseException::TLBL, false, false, f)?;
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
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; Ok(()) };
        test_nomiss_exception(pagemask, 1048572, true, true, f)?;
        test_miss_exception(pagemask, 1048576, true, true, 1, CauseException::TLBL, false, false, f)?;
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
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; Ok(()) };
        // This will exceed RDRAM (unless there's an expansion pack), but that doesn't cause an error
        test_nomiss_exception(pagemask, 4194300, true, true, f)?;
        test_miss_exception(pagemask, 4194304, true, true, 1, CauseException::TLBL, false, false, f)?;
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
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; Ok(()) };
        // This will exceed RDRAM, but that doesn't cause an error
        test_nomiss_exception(pagemask, 16777212, true, true, f)?;
        test_miss_exception(pagemask, 16777216, true, true, 1, CauseException::TLBL, false, false, f)?;
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
        let f = |address| { unsafe { (address as *mut u32).write_volatile(0) }; Ok(()) };
        test_nomiss_exception(pagemask, 4092, true, true, f)?;
        test_miss_exception(pagemask, 4096, true, true, 1, CauseException::TLBS, false, false, f)?;
        Ok(())
    }
}

/// This verifies that code that lies within the TLB mapped area can be executed
pub struct ExecuteTLBMapped4k {}

impl Test for ExecuteTLBMapped4k {
    fn name(&self) -> &str { "TLB: Execute code mapped in 4k page" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0; // 4k
        test_nomiss_exception(pagemask, 4096, true, true, |address| {
            unsafe {
                // Write a small function into the tlb mapped area, at the end. It sets V0 and returns to A0
                ((address - 12) as *mut u32).write_volatile(0x2402FACE); // ADDIU V0, R0, 0xFACE
                ((address - 8) as *mut u32).write_volatile(0x00800008);  // JR A0
                ((address - 4) as *mut u32).write_volatile(0x00000000);  // NOP (delay slot)

                // Invalidate the code so that it can be executed
                cop0::cache::<1, 0>((address - 32) as usize);
                cop0::cache::<1, 0>((address - 16) as usize);
                cop0::cache::<0, 0>((address - 32) as usize);

                let mut result: u32;
                asm!("
                    JALR $4, $3
                ", in("$3") (address - 12), out("$2") result, out("$4") _);

                soft_assert_eq(result, 0xFFFF_FACE, "Return value of function in TLB mapped space")?;
            }

            Ok(())
        })?;
        Ok(())
    }
}

/// This verifies that the exception when trying to execute code from a tlb area that doesn't exist is accurate
pub struct ExecuteTLBMappedMiss {}

impl Test for ExecuteTLBMappedMiss {
    fn name(&self) -> &str { "TLB: Execute code that is not currently tlb mapped" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let virtual_page_base = setup_tlb_page(0, true, true);

        let fault_address = virtual_page_base + 4096;
        let exception_context = expect_exception(CauseException::TLBL, -4i64 as u64, || {
            // We'll execute directly into fault_address, which will immediately cause a TLB miss exception
            // The exception will then go backwards a few instructions (-4), at which point we have valid
            // code that causes a proper return
            unsafe {
                ((fault_address - 20) as *mut u32).write_volatile(0x24420001); // ADDIU V0, V0, 1
                ((fault_address - 16) as *mut u32).write_volatile(0x24420002); // ADDIU V0, V0, 2 <-- The exception should land us here
                ((fault_address - 12) as *mut u32).write_volatile(0x24420004); // ADDIU V0, V0, 4
                ((fault_address - 8) as *mut u32).write_volatile(0x00800008);  // JR A0
                ((fault_address - 4) as *mut u32).write_volatile(0x00000000);  // NOP (delay slot)

                // Invalidate the code so that it can be executed
                cop0::cache::<1, 0>((fault_address - 32) as usize);
                cop0::cache::<1, 0>((fault_address - 16) as usize);
                cop0::cache::<0, 0>((fault_address - 32) as usize);

                // Call into the next page. The exception handler will then resume by applying a negative offset so that we exit out gracefully
                let mut result: u32;
                asm!("
                    ADDIU $2, $0, 0
                    JALR $4, $3
                ", out("$2") result, in("$3") fault_address, out("$4") _);

                if result != 6 {
                    return Err("Didn't return correct value. Most likely, ExceptPC during TLB exception was wrong");
                };
            }

            Ok(())
        })?;
        let expected_context = ((fault_address >> 13) << 4) as u64;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector for TLB exception")?;
        soft_assert_eq(exception_context.exceptpc, fault_address as u64, "ExceptPC during TLB exception")?;
        soft_assert_eq(exception_context.badvaddr, fault_address as u64, "BadVAddr during TLB exception")?;
        soft_assert_eq(exception_context.cause, (CauseException::TLBL as u32) << 2, "Cause during TLB exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during TLB exception")?;
        soft_assert_eq(exception_context.context, expected_context, "Context during TLB exception")?;
        soft_assert_eq(exception_context.xcontext, expected_context, "XContext during TLB exception")?;
        Ok(())
    }
}

/// This verifies that the exception when trying to execute code from a tlb area that doesn't exist is accurate
pub struct ExecuteTLBMappedMissInDelay {}

impl Test for ExecuteTLBMappedMissInDelay {
    fn name(&self) -> &str { "TLB: Execute mapped branch with has a non-mapped delay slot" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let virtual_page_base = setup_tlb_page(0, true, true);

        let fault_address = virtual_page_base + 4096;
        let exception_context = expect_exception(CauseException::TLBL, -4i64 as u64, || {
            // We'll execute the last instruction in the tlb block, which is a JALR. It has a delay slot
            // in unmapped space, which should fire an exception. Once we get that, we'll go back a few instructions
            // and return
            unsafe {
                ((fault_address - 24) as *mut u32).write_volatile(0x24420001); // ADDIU $2, $2, 1
                ((fault_address - 20) as *mut u32).write_volatile(0x24420002); // ADDIU $2, $2, 2 <-- The exception should land us here
                ((fault_address - 16) as *mut u32).write_volatile(0x24420004); // ADDIU $2, $2, 4
                ((fault_address - 12) as *mut u32).write_volatile(0x00800008); // JR $4
                ((fault_address - 8) as *mut u32).write_volatile(0x00000000);  // NOP
                ((fault_address - 4) as *mut u32).write_volatile(0x00603009);  // JALR $6, $3 (with a delay slot in unmapped space)

                // Invalidate the code so that it can be executed
                cop0::cache::<1, 0>((fault_address - 32) as usize);
                cop0::cache::<1, 0>((fault_address - 16) as usize);
                cop0::cache::<0, 0>((fault_address - 32) as usize);

                // Call into the next page. The exception handler will then resume by applying a negative offset so that we exit out gracefully
                let mut result: u32;
                let mut fault_jalr_ra: u32;
                asm!("
                    LI $2, 0
                    LI $6, 0
                    JALR $4, $3
                ", out("$2") result, in("$3") (fault_address - 4), out("$4") _, out("$6") fault_jalr_ra);

                if result != 6 {
                    return Err("Didn't return correct value. Most likely, ExceptPC during TLB exception was wrong");
                };
                if fault_jalr_ra != fault_address + 4 {
                    return Err("Even-though the delay instruction isn't tlb mapped, JALR should still set RA");
                };
            }

            Ok(())
        })?;
        let expected_context = ((fault_address >> 13) << 4) as u64;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector for TLB exception")?;
        soft_assert_eq(exception_context.exceptpc, (fault_address - 4) as u64, "ExceptPC during TLB exception")?;
        soft_assert_eq(exception_context.badvaddr, fault_address as u64, "BadVAddr during TLB exception")?;
        soft_assert_eq(exception_context.cause, 0x80000008, "Cause during TLB exception")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status during TLB exception")?;
        soft_assert_eq(exception_context.context, expected_context, "Context during TLB exception")?;
        soft_assert_eq(exception_context.xcontext, expected_context, "XContext during TLB exception")?;
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
        let f = |address| { unsafe { (address as *mut u32).read_volatile() }; Ok(()) };
        test_miss_exception(pagemask, 4092, false, true, 1, CauseException::TLBL, true, false, f)?;
        Ok(())
    }
}

pub struct ReadNonValid4kInDelay {}

impl Test for ReadNonValid4kInDelay {
    fn name(&self) -> &str { "TLB: Read inside 4k page which isn't valid from a delay slot, expect TLBL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pagemask = 0; // 4k
        test_miss_exception(pagemask, 4092, false, true, 3, CauseException::TLBL, true, true, |address| {
            let mut result: u32;
            let mut unchanged: u32 = 0x12345678;
            unsafe {
                asm!("
                    .set noreorder
                    LI $2, 0
                    BEQ $0, $0, 4
                    LWC1 $4, 0($3)
                    ADDIU $2, $2, 1  // BEQ target (but doesn't matter due to exception)
                    ADDIU $2, $2, 2  // Target after exception handler due to skip_instructions
                ", out("$2") result, in("$3") address, inout("$4") unchanged);
            }

            soft_assert_eq(result, 2, "Exception handler didn't return at the right location. Most likely the ExceptPC was wrong")?;
            soft_assert_eq(unchanged, 0x12345678, "LWC1 shouldn't overwrite target register as it should tlb fault")?;

            Ok(())
        })?;
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
        let f = |address| { unsafe { (address as *mut u32).write_volatile(0) }; Ok(()) };
        test_miss_exception(pagemask, 4092, false, true, 1, CauseException::TLBS, true, false, f)?;
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
        let f = |address| { unsafe { (address as *mut u32).write_volatile(0) }; Ok(()) };
        test_miss_exception(pagemask, 4092, true, false, 1, CauseException::Mod, true, false, f)?;
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
        let f = |address| { unsafe { (address as *mut u32).write_volatile(0) }; Ok(()) };
        test_miss_exception(pagemask, 4092, false, false, 1, CauseException::TLBS, true, false, f)?;
        Ok(())
    }
}

pub struct LWTLBMissTest32 {}

impl Test for LWTLBMissTest32 {
    fn name(&self) -> &str { "LW tlb miss test (32 bit)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
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
