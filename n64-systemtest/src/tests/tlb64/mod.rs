use alloc::{format, vec};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::{cop0, MemoryMap};
use crate::cop0::{CauseException, make_entry_hi, make_entry_lo};
use crate::exception_handler::expect_exception;
use crate::math::bits::Bitmasks64;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq2};
use crate::uncached_memory::UncachedHeapMemory;

pub mod exceptions;

#[derive(Default, Debug)]
#[repr(C)]
struct AllLoadsResult {
    lb: u64,
    lbu: u64,
    lh: u64,
    lhu: u64,
    lw: u64,
    lwu: u64,
    ld: u64,
    lwc1: u64,
    ldc1: u64,
    ll: u64,
    lld: u64,
    lwl: [u64; 4],
    lwr: [u64; 4],
    ldl: [u64; 8],
    ldr: [u64; 8],
}

impl AllLoadsResult {
    fn assert_equal(&self, expected: AllLoadsResult) -> Result<(), String> {
        soft_assert_eq(self.lb, expected.lb, "LB result")?;
        soft_assert_eq(self.lbu, expected.lbu, "LBU result")?;
        soft_assert_eq(self.lh, expected.lh, "LH result")?;
        soft_assert_eq(self.lhu, expected.lhu, "LHU result")?;
        soft_assert_eq(self.lw, expected.lw, "LW result")?;
        soft_assert_eq(self.lwu, expected.lwu, "LWU result")?;
        soft_assert_eq(self.ld, expected.ld, "LD result")?;
        soft_assert_eq(self.lwc1, expected.lwc1, "LWC1 result")?;
        soft_assert_eq(self.ldc1, expected.ldc1, "LDC1 result")?;
        soft_assert_eq(self.ll, expected.ll, "LL result")?;
        soft_assert_eq(self.lld, expected.lld, "LLD result")?;
        soft_assert_eq(self.lwl, expected.lwl, "LWL result")?;
        soft_assert_eq(self.lwr, expected.lwr, "LWR result")?;
        soft_assert_eq(self.ldl, expected.ldl, "LDL result")?;
        soft_assert_eq(self.ldr, expected.ldr, "LDR result")?;

        Ok(())
    }
}

fn do_all_loads(ptr64: u64) -> AllLoadsResult {
    let mut result: AllLoadsResult = Default::default();

    unsafe {
        asm!("
            LD $3, 0($2)  // get actual pointer

            LB $4, 0($3)
            SD $4, 0($5)

            LBU $4, 0($3)
            SD $4, 8($5)

            LH $4, 0($3)
            SD $4, 16($5)

            LHU $4, 0($3)
            SD $4, 24($5)

            LW $4, 0($3)
            SD $4, 32($5)

            LWU $4, 0($3)
            SD $4, 40($5)

            LD $4, 0($3)
            SD $4, 48($5)

            LWC1 $4, 0($3)
            MFC1 $4, $4
            SD $4, 56($5)

            LDC1 $4, 0($3)
            DMFC1 $4, $4
            SD $4, 64($5)

            LL $4, 0($3)
            SD $4, 72($5)

            LLD $4, 0($3)
            SD $4, 80($5)

            LWL $4, 0($3)
            SD $4, 88($5)
            LWL $4, 1($3)
            SD $4, 96($5)
            LWL $4, 2($3)
            SD $4, 104($5)
            LWL $4, 3($3)
            SD $4, 112($5)

            LUI $4, 0x0102
            ORI $4, 0x0304
            LWR $4, 0($3)
            SD $4, 120($5)
            LWR $4, 1($3)
            SD $4, 128($5)
            LWR $4, 2($3)
            SD $4, 136($5)
            LWR $4, 3($3)
            SD $4, 144($5)

            LDL $4, 0($3)
            SD $4, 152($5)
            LDL $4, 1($3)
            SD $4, 160($5)
            LDL $4, 2($3)
            SD $4, 168($5)
            LDL $4, 3($3)
            SD $4, 176($5)
            LDL $4, 4($3)
            SD $4, 184($5)
            LDL $4, 5($3)
            SD $4, 192($5)
            LDL $4, 6($3)
            SD $4, 200($5)
            LDL $4, 7($3)
            SD $4, 208($5)

            LUI $4, 0x0102
            ORI $4, 0x0304
            LDR $4, 0($3)
            SD $4, 216($5)
            LDR $4, 1($3)
            SD $4, 224($5)
            LDR $4, 2($3)
            SD $4, 232($5)
            LDR $4, 3($3)
            SD $4, 240($5)
            LDR $4, 4($3)
            SD $4, 248($5)
            LDR $4, 5($3)
            SD $4, 256($5)
            LDR $4, 6($3)
            SD $4, 264($5)
            LDR $4, 7($3)
            SD $4, 272($5)

        ", in("$2") &ptr64, out("$3") _, out("$4") _, in("$5") &mut result)
    }

    result
}

const TEST_DATA: [u64; 3] = [0xBADD_ECAF_0123_4567, 0x89AB_CDEF_0011_2233, 0x4455_6677__8899_AABB];

const EXPECTED: AllLoadsResult = AllLoadsResult {
    lb: 0xFFFFFFFF_FFFFFFBA,
    lbu: 0xBA,
    lh: 0xFFFFFFFF_FFFFBADD,
    lhu: 0xBADD,
    lw: 0xFFFFFFFF_BADDECAF,
    lwu: 0xBADDECAF,
    ld: 0xBADDECAF_01234567,
    lwc1: 0xFFFFFFFF_BADDECAF,
    ldc1: 0xBADDECAF_01234567,
    ll: 0xFFFFFFFF_BADDECAF,
    lld: 0xBADDECAF_01234567,
    lwl: [0xFFFFFFFF_BADDECAF, 0xFFFFFFFF_DDECAFAF, 0xFFFFFFFF_ECAFAFAF, 0xFFFFFFFF_AFAFAFAF],
    lwr: [0x010203BA, 0x0102BADD, 0x01BADDEC, 0xFFFFFFFF_BADDECAF],
    ldl: [0xBADDECAF_01234567, 0xDDECAF01_23456767, 0xECAF0123_45676767, 0xAF012345_67676767, 0x1234567_67676767, 0x23456767_67676767, 0x45676767_67676767, 0x67676767_67676767],
    ldr: [0x010203BA, 0x0102BADD, 0x01BADDEC, 0xBADDECAF, 0xBA_DDECAF01, 0xBADD_ECAF0123, 0xBADDEC_AF012345, 0xBADDECAF_01234567],
};

pub struct AllLoads32BitAddress {}

impl Test for AllLoads32BitAddress {
    fn name(&self) -> &str { "Loads from 32 bit address (0x80xxxxxxxx) while using 64 bit addressing mode" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Enable 64 bit kernel addressing mode
        unsafe { cop0::set_status(0x240000E0); }

        let all_loads_regular = do_all_loads((&TEST_DATA[0]) as *const u64 as isize as u64);

        all_loads_regular.assert_equal(EXPECTED)?;

        Ok(())
    }
}

pub struct AllLoads32BitAddressUncached {}

impl Test for AllLoads32BitAddressUncached {
    fn name(&self) -> &str { "Loads from 32 bit address uncached (0xA0xxxxxxxx) while using 64 bit addressing mode" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Enable 64 bit kernel addressing mode
        unsafe { cop0::set_status(0x240000E0); }

        let all_loads_regular = do_all_loads(MemoryMap::uncached(&TEST_DATA[0]) as *const u64 as isize as u64);

        all_loads_regular.assert_equal(EXPECTED)?;

        Ok(())
    }
}

pub struct AllLoads0x90 {}

impl Test for AllLoads0x90 {
    fn name(&self) -> &str { "Loads from 64 bit address (0x90xxxxxx_xxxxxxxx) while using 64 bit addressing mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Enable 64 bit kernel addressing mode
        unsafe { cop0::set_status(0x240000E0); }

        let address = ((&TEST_DATA[0] as *const u64 as usize as u64) & Bitmasks64::M29) | 0x90000000_00000000;
        let all_loads_regular = do_all_loads(address);

        all_loads_regular.assert_equal(EXPECTED)?;

        Ok(())
    }
}

pub struct AllLoads0x98 {}

impl Test for AllLoads0x98 {
    fn name(&self) -> &str { "Loads from 64 bit address (0x98xxxxxx_xxxxxxxx) while using 64 bit addressing mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Enable 64 bit kernel addressing mode
        unsafe { cop0::set_status(0x240000E0); }

        let address = (&TEST_DATA[0] as *const u64 as usize as u64) & Bitmasks64::M29 | 0x98000000_00000000;
        let all_loads_regular = do_all_loads(address);

        all_loads_regular.assert_equal(EXPECTED)?;

        Ok(())
    }
}

fn test_limits(largest_valid_address: u64, expected_lladdr: u64, smallest_exception_address: u64) -> Result<(), String> {
    // Enable 64 bit kernel addressing mode
    unsafe { cop0::set_status(0x240000E0); }

    // Load should be without exception
    unsafe {
        asm!("
            LD $3, 0($2)  // get actual pointer
            LL $4, 0($3)
            NOP
        ", in("$2") &largest_valid_address, out("$3") _, out("$4") _);
    }

    soft_assert_eq2(cop0::lladdr(), expected_lladdr, || format!("LLAddr after reading with LL from 0x{:x}", largest_valid_address))?;

    expect_exception(CauseException::AdEL, 1, || {
        unsafe {
            asm!("
                 LD $3, 0($2)  // get actual pointer
                 LW $4, 0($3)
             ", in("$2") &smallest_exception_address, out("$3") _, out("$4") _);
        }
        Ok(())
    })?;

    Ok(())
}

pub struct LimitsOf0x90 {}

impl Test for LimitsOf0x90 {
    fn name(&self) -> &str { "Loads from 64 bit address (0x90xxxxxx_xxxxxxxx) (limits)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // You'd think that reading from 0x9000_0000_8000_0000u64 (address above plus 1) would give an address
        // error, but the console dies. One bit higher works as expected
        test_limits(0x9000_0000_7FFF_FFF8u64, 0x7FF_FFFF, 0x9000_0001_0000_0000u64)
    }
}

fn test(tlb_address: u64, vpn: u32, r: u8) -> Result<(), String> {
    // Enable 64 bit kernel addressing mode
    unsafe { cop0::set_status(0x240000E0); }

    let mut data = UncachedHeapMemory::<u32>::new_with_align((16 * 1024) >> 2, 16 * 1024);

    // Create a test_value to write and expect back. Derive it from the address but NOT it so that it doesn't look like an address
    let test_value_begin = ((!tlb_address) ^ (tlb_address >> 32)) as u32;
    let test_value_end = test_value_begin + 1234567;

    const END_OF_PAGE_OFFSET: usize = 16 * 1024 - 4;
    assert!(data.count() * 4 - 4 == END_OF_PAGE_OFFSET);

    unsafe {
        cop0::clear_tlb();
        cop0::write_tlb(
            10,
            0b11 << 13,
            make_entry_lo(true, true, false, 0, (data.start_phyiscal() >> 12) as u32),
            make_entry_lo(true, false, false, 0, 0),
            make_entry_hi(0, vpn, r));

        cop0::cache64::<1, 0>(tlb_address);
        cop0::cache64::<1, 0>(tlb_address + END_OF_PAGE_OFFSET as u64);

        // Write some value without TLB
        data.write(0, test_value_begin);
        data.write(END_OF_PAGE_OFFSET >> 2, test_value_end);
    }

    // Read it back using the TLB. Have to use asm as Rust doesn't handle 64-bit pointers
    let mut value_begin: u32;
    let mut value_end: u32;
    unsafe {
        asm!("
            .set noat
            LD $2, 0 ($3)

            LW $4, 0 ($2)
            DADDIU $2, $2, {OFFSET}
            LW $5, 0 ($2)
        ", OFFSET = const END_OF_PAGE_OFFSET, in("$3") &tlb_address, out("$2") _, out("$4") value_begin, out("$5") value_end)
    }

    soft_assert_eq(value_begin, test_value_begin, "Value read back through TLB mapped memory (begin)")?;
    soft_assert_eq(value_end, test_value_end, "Value read back through TLB mapped memory (end)")?;

    Ok(())
}

pub struct TLB64Read {}

impl Test for TLB64Read {
    fn name(&self) -> &str { "TLB: Use TLB for reading (64 bit addressing mode)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((0x00000000_0DEA0000u64, 0x0000_DEA0u32 >> 1, 0u8)),
            Box::new((0x00000000_DEA00000u64, 0x000D_EA00u32 >> 1, 0u8)),
            Box::new((0x00000003_F0000000u64, 0x003F_0000u32 >> 1, 0u8)),
            Box::new((0x00000003_F0000000u64, 0x003F_0000u32 >> 1, 0u8)),
            Box::new((0x00000007_F0000000u64, 0x007F_0000u32 >> 1, 0u8)),
            Box::new((0x0000003F_F0000000u64, 0x03FF_0000u32 >> 1, 0u8)),
            Box::new((0x000000FF_F0000000u64, 0x0FFF_0000u32 >> 1, 0u8)),

            Box::new((0x400000FF_10000000u64, 0x0FF1_0000u32 >> 1, 1u8)),
            Box::new((0x400000FF_FF200000u64, 0x0FFF_F200u32 >> 1, 1u8)),

            Box::new((0xC0000000_00000000u64, 0x0000_0000u32 >> 1, 3u8)),
            Box::new((0xC00000FF_20000000u64, 0x0FF2_0000u32 >> 1, 3u8)),
            Box::new((0xC00000FF_40000000u64, 0x0FF4_0000u32 >> 1, 3u8)),
            Box::new((0xC00000FF_70000000u64, 0x0FF7_0000u32 >> 1, 3u8)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(u64, u32, u8)>() {
            Some((address, vpn, r)) => {
                test(*address, *vpn, *r)
            }
            _ => Err("Value is not valid".to_string())
        }
    }
}

/// This verifies that code that lies within the TLB mapped area can be executed.
/// For this test to succeed, PC needs to be 64 bit wide
pub struct TLB64Execute {}

impl Test for TLB64Execute {
    fn name(&self) -> &str { "TLB: Execute code from a tlb mapped 64 bit location" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Enable 64 bit kernel addressing mode
        unsafe { cop0::set_status(0x240000E0); }
        let pagemask = 0b11 << 13; // 16k

        let mut data64 = UncachedHeapMemory::<u32>::new_with_align((16 * 1024) >> 2, 16 * 1024);
        let mut data32 = UncachedHeapMemory::<u32>::new_with_align((16 * 1024) >> 2, 16 * 1024);

        unsafe { cop0::clear_tlb(); }
        unsafe { cop0::set_context_64(0); }
        unsafe { cop0::set_xcontext_64(0); }

        let virtual_address = 0xFF_8000_0000u64 | (data32.start_phyiscal() as u32 as u64);

        // Setup two pages:
        // - A page using 64 bit addressing mode, which is the one that is supposed to be hit
        // - A page fallback page that is hit by emulators that incorrectly think the PC is 32 bit
        unsafe {
            // The 64 bit mapping (the one we want to hit)
            cop0::write_tlb(
                4,
                pagemask,
                make_entry_lo(true, true, false, 0, (data64.start_phyiscal() >> 12) as u32),
                make_entry_lo(true, true, false, 0, (data64.start_phyiscal() >> 12) as u32),
                make_entry_hi(2, (virtual_address >> 13) as u32, 0));

            // The 32 bit fallback mapping (so that we don't just die)
            cop0::write_tlb(
                5,
                pagemask,
                make_entry_lo(true, true, false, 0, (data32.start_phyiscal() >> 12) as u32),
                make_entry_lo(true, true, false, 0, (data32.start_phyiscal() >> 12) as u32),
                make_entry_hi(2, ((virtual_address & Bitmasks64::M32) >> 13) as u32, 0));
        }

        unsafe {
            // Write a small function into the tlb mapped area, at the end. It sets V0 and returns to A0
            data64.write(0, 0x24020040);  // ADDIU V0, R0, 64
            data64.write(1, 0x00800008);  // JR A0
            data64.write(2, 0x00000000);  // NOP (delay slot)

            // Write a small function into the tlb mapped area, at the end. It sets V0 and returns to A0
            data32.write(0, 0x24020020);  // ADDIU V0, R0, 32
            data32.write(1, 0x00800008);  // JR A0
            data32.write(2, 0x00000000);  // NOP (delay slot)

            // Invalidate the code so that it can be executed
            cop0::cache64::<1, 0>(virtual_address);
            cop0::cache64::<0, 0>(virtual_address);

            cop0::cache::<1, 0>(virtual_address as usize);
            cop0::cache::<0, 0>(virtual_address as usize);

            let mut result: u32;
            asm!("
                TNE $0, $0
                LD $2, 0 ($3)
                JALR $4, $2
                NOP
            ", in("$3") &virtual_address, out("$2") result, out("$4") _);

            soft_assert_eq(result, 64, "Return value of function in TLB mapped space")?;
        }

        Ok(())
    }
}
