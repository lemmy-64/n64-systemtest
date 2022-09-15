use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use arbitrary_int::{u19, u2, u27};

use crate::cop0;
use crate::cop0::{Cause, CauseException, Context, make_entry_hi, make_entry_lo, Status, XContext};
use crate::exception_handler::expect_exception;
use crate::math::bits::Bitmasks64;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;
use crate::uncached_memory::UncachedHeapMemory;

pub struct LWAddressNotSignExtended {}

impl Test for LWAddressNotSignExtended {
    fn name(&self) -> &str { "LW with address not sign extended (64 bit)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Enable 64 bit kernel addressing mode
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }

        // Load from 0x00000000_80xxxxxx causes TLBL, as upper bits are 0
        let mut a = 0x12345678u32;
        let p = &mut a as *mut u32;

        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let exception_context = expect_exception(CauseException::TLBL, 1, || {
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

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000080, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x8C400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, p as u64 & 0xFFFFFFFF, "BadVAddr")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0x8, "Cause")?;
        soft_assert_eq(exception_context.status, 0x240000E2, "Status")?;
        soft_assert_eq(exception_context.context, Context::from_virtual_address(p as u64 & 0xFFFFFFFF), "Context")?;
        soft_assert_eq(exception_context.xcontext, XContext::from_virtual_address(p as u64 & 0xFFFFFFFF), "XContext")?;

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
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }

        // Store to 0x00000000_80xxxxxx causes TLBS, as upper bits are 0
        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }
        let mut a = 0x12345678u32;
        let p = &mut a as *mut u32;
        let exception_context = expect_exception(CauseException::TLBS, 1, || {
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

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000080, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0xAC400000, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.badvaddr, p as u64 & 0xFFFFFFFF, "BadVAddr")?;
        soft_assert_eq(exception_context.cause.raw_value(), 0xC, "Cause")?;
        soft_assert_eq(exception_context.status, 0x240000E2, "Status")?;
        soft_assert_eq(exception_context.context, Context::from_virtual_address(p as u64 & 0xFFFFFFFF), "Context")?;
        soft_assert_eq(exception_context.xcontext, XContext::from_virtual_address(p as u64 & 0xFFFFFFFF), "XContext")?;

        Ok(())
    }
}

/// Tests a LW and catches the exception. If tlb_miss then we expect TLBL; otherwise AdEL
fn test_load_and_catch_exception(address: u64, tlb_miss: bool) -> Result<(), String> {
    unsafe { cop0::clear_tlb(); }

    // Enable 64 bit kernel addressing mode
    unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }

    unsafe { cop0::set_context_64(0); }
    unsafe { cop0::set_xcontext_64(0); }
    unsafe { cop0::set_entry_hi(0); }
    let cause_exception = if tlb_miss { CauseException::TLBL } else { CauseException::AdEL };
    let exception_context = expect_exception(cause_exception, 1, || {
        unsafe {
            asm!("
                .set noat
                // Load 64 bit address
                LD $2, 0 ($3)

                // Actual load that should cause the tlb miss
                LW $0, 0 ($2)
            ", in("$3") &address, out("$2") _)
        }

        Ok(())
    })?;

    soft_assert_eq(exception_context.cause,  Cause::new().with_exception(cause_exception), "Cause")?;
    soft_assert_eq(exception_context.k0_exception_vector, if tlb_miss { 0xFFFFFFFF_80000080 } else { 0xFFFFFFFF_80000180 }, "Exception Vector")?;
    soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
    soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x8C400000, "ExceptPC points to wrong instruction")?;
    soft_assert_eq(exception_context.badvaddr, address, "BadVAddr")?;
    soft_assert_eq(exception_context.status, 0x240000E2, "Status")?;
    let vpn = (address >> 13) & Bitmasks64::M27;
    let r = (address >> 62) & Bitmasks64::M2;
    soft_assert_eq(exception_context.context.raw_value(),  (vpn & Bitmasks64::M19) << 4, "Context")?;
    soft_assert_eq(exception_context.xcontext.raw_value(), (vpn << 4) | (r << 31), "XContext")?;
    soft_assert_eq(exception_context.entry_hi , (vpn << 13) | (r << 62), "EntryHi")?;

    Ok(())
}

pub struct TLBAndAddressError64 {}

impl Test for TLBAndAddressError64 {
    fn name(&self) -> &str { "LW TLB Miss or Address Exception (64 bit addressing mode)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0x00000000_00000000u64)),
            Box::new((true, 0x00000000_0020FFF4u64)),
            Box::new((true, 0x00000000_8020FFF4u64)),
            Box::new((true, 0x00000080_0020FFF4u64)),
            Box::new((true, 0x000000FF_0020FFF4u64)),
            Box::new((true, 0x000000FF_80000004u64)),
            Box::new((true, 0x000000FF_FF000004u64)),
            Box::new((true, 0x000000FF_FFFFFFFCu64)),
            Box::new((false, 0x00000100_0020FFF4u64)),

            Box::new((true, 0x40000000_00000000u64)),
            Box::new((true, 0x40000000_0020FFF4u64)),
            Box::new((true, 0x40000000_8020FFF4u64)),
            Box::new((true, 0x40000080_0020FFF4u64)),
            Box::new((true, 0x400000FF_FFFFFFFCu64)),
            Box::new((false, 0x40000100_0020FFF4u64)),

            Box::new((true, 0xC0000000_00000000u64)),
            Box::new((true, 0xC0000000_0020FFF4u64)),
            Box::new((true, 0xC0000000_8020FFF4u64)),
            Box::new((true, 0xC0000080_0020FFF4u64)),
            Box::new((true, 0xC00000FF_00000004u64)),
            Box::new((true, 0xC00000FF_7FFFFFFCu64)),
            Box::new((false, 0xC00000FF_80000000u64)),
            Box::new((false, 0xC00000FF_F0000000u64)),
            Box::new((false, 0xC00000FF_F0000004u64)),
            Box::new((false, 0xC00000FF_FFFFFFF4u64)),
            Box::new((false, 0xC0000100_0020FFF4u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(bool, u64)>() {
            Some((expect_tlb_miss, address)) => {
                test_load_and_catch_exception(*address, *expect_tlb_miss)
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

fn test_tlb_miss(address: u64, vpn: u27, r: u2) -> Result<(), String> {
    // Enable 64 bit kernel addressing mode
    unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }

    let mut data = UncachedHeapMemory::<u32>::new_with_align((16 * 1024) >> 2, 16 * 1024);

    unsafe {
        cop0::clear_tlb();
        cop0::write_tlb(
            10,
            0b11 << 13,
            make_entry_lo(true, true, false, 0, (data.start_phyiscal() >> 12) as u32),
            make_entry_lo(true, false, false, 0, 0),
            make_entry_hi(0, vpn, r));
    }

    // Read it back using the TLB. Have to use asm as Rust doesn't handle 64-bit pointers
    let exception_context = expect_exception(CauseException::TLBL, 1, || {
        unsafe {
            asm!("
                .set noat
                LD $2, 0 ($3)
                LW $4, 0 ($2)
        ", in("$3") &address, out("$2") _, out("$4") _)
        }

        Ok(())
    })?;

    soft_assert_eq(exception_context.cause, Cause::new().with_exception(CauseException::TLBL), "Cause")?;
    soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000080, "Exception Vector")?;
    soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
    soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, 0x8C440000, "ExceptPC points to wrong instruction")?;
    soft_assert_eq(exception_context.badvaddr, address, "BadVAddr")?;
    soft_assert_eq(exception_context.status, 0x240000E2, "Status")?;
    let vpn19 = u19::extract_u64(address, 13);
    let vpn27 = u27::extract_u64(address, 13);
    let r = u2::extract_u64(address, 62);
    soft_assert_eq(exception_context.context, Context::new().with_bad_vpn2(vpn19), "Context")?;
    soft_assert_eq(exception_context.xcontext, XContext::new().with_bad_vpn2(vpn27).with_r(r), "XContext")?;
    soft_assert_eq(exception_context.entry_hi , make_entry_hi(0, vpn27, r), "EntryHi")?;

    Ok(())
}

pub struct TLB64MissDueToR64Bit {}

impl Test for TLB64MissDueToR64Bit {
    fn name(&self) -> &str { "TLB: Expect TLB miss on R mismatch (64 bit addressing mode)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // These would match if r was 0
            Box::new((0x00000000_0DEA0000u64, u27::new(0x0000_DEA0u32 >> 1), u2::new(1))),
            Box::new((0x00000000_DEA00000u64, u27::new(0x000D_EA00u32 >> 1), u2::new(1))),
            Box::new((0x00000003_F0000000u64, u27::new(0x003F_0000u32 >> 1), u2::new(1))),
            Box::new((0x00000003_F0000000u64, u27::new(0x003F_0000u32 >> 1), u2::new(2))),
            Box::new((0x00000007_F0000000u64, u27::new(0x007F_0000u32 >> 1), u2::new(2))),
            Box::new((0x0000003F_F0000000u64, u27::new(0x03FF_0000u32 >> 1), u2::new(3))),
            Box::new((0x000000FF_F0000000u64, u27::new(0x0FFF_0000u32 >> 1), u2::new(3))),

            // These would match if r was 1
            Box::new((0x400000FF_10000000u64, u27::new(0x0FF1_0000u32 >> 1), u2::new(0))),
            Box::new((0x400000FF_FF200000u64, u27::new(0x0FFF_F200u32 >> 1), u2::new(0))),
            Box::new((0x400000FF_10000000u64, u27::new(0x0FF1_0000u32 >> 1), u2::new(2))),
            Box::new((0x400000FF_FF200000u64, u27::new(0x0FFF_F200u32 >> 1), u2::new(2))),
            Box::new((0x400000FF_10000000u64, u27::new(0x0FF1_0000u32 >> 1), u2::new(3))),
            Box::new((0x400000FF_FF200000u64, u27::new(0x0FFF_F200u32 >> 1), u2::new(3))),

            // These would match if r was 3
            Box::new((0xC0000000_00000000u64, u27::new(0x0000_0000u32 >> 1), u2::new(0))),
            Box::new((0xC00000FF_20000000u64, u27::new(0x0FF2_0000u32 >> 1), u2::new(0))),
            Box::new((0xC00000FF_40000000u64, u27::new(0x0FF4_0000u32 >> 1), u2::new(0))),
            Box::new((0xC00000FF_70000000u64, u27::new(0x0FF7_0000u32 >> 1), u2::new(0))),
            Box::new((0xC0000000_00000000u64, u27::new(0x0000_0000u32 >> 1), u2::new(1))),
            Box::new((0xC00000FF_20000000u64, u27::new(0x0FF2_0000u32 >> 1), u2::new(1))),
            Box::new((0xC00000FF_40000000u64, u27::new(0x0FF4_0000u32 >> 1), u2::new(1))),
            Box::new((0xC00000FF_70000000u64, u27::new(0x0FF7_0000u32 >> 1), u2::new(1))),
            Box::new((0xC0000000_00000000u64, u27::new(0x0000_0000u32 >> 1), u2::new(2))),
            Box::new((0xC00000FF_20000000u64, u27::new(0x0FF2_0000u32 >> 1), u2::new(2))),
            Box::new((0xC00000FF_40000000u64, u27::new(0x0FF4_0000u32 >> 1), u2::new(2))),
            Box::new((0xC00000FF_70000000u64, u27::new(0x0FF7_0000u32 >> 1), u2::new(2))),

// These would match if we only looked at 32 bit of the address
            Box::new((0x00000001_F0000000u64, u27::new(0x003F_0000u32 >> 1), u2::new(0))),
            Box::new((0x00000002_F0000000u64, u27::new(0x003F_0000u32 >> 1), u2::new(0))),
            Box::new((0x0000007F_F0000000u64, u27::new(0x0FFF_0000u32 >> 1), u2::new(0))),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u64, u27, u2)>() {
            Some((address, vpn, r)) => {
                test_tlb_miss(*address, *vpn, *r)
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}
