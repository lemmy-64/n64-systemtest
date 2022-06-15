use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use crate::cop0::CauseException;
use crate::exception_handler::expect_exception;
use crate::math::bits::Bitmasks64;
use crate::{cop0, MemoryMap};

use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq2};

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
