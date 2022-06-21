use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use core::cmp::min;

use crate::cop0;
use crate::cop0::{make_entry_hi, make_entry_lo};
use crate::memory_map::MemoryMap;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_greater_or_equal, soft_assert_less};

pub mod exceptions;

// TODO: TLBWR

pub struct WiredRandom {}

impl Test for WiredRandom {
    fn name(&self) -> &str { "Wired/Random" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for wired in 0..32 {
            unsafe { cop0::set_wired(wired); }
            soft_assert_eq(cop0::wired(), wired, "WIRED after setting")?;

            // Keep getting random values. Ensure we hit a couple of different values
            // and ensure we're never out of bounds
            let mut value_seen = [false; 32];
            let mut count_values_seen = 0;
            let mut counter = 0;
            let expected_unique_value_count = min(10, 32 - wired as usize);
            while count_values_seen < expected_unique_value_count {
                let random = cop0::random();
                // Ensure it is within the expected range
                soft_assert_greater_or_equal(random, wired, "RANDOM must be >= WIRED")?;
                soft_assert_less(random, 32, "RANDOM must be < 32")?;

                // Count unique values
                if !value_seen[random as usize] {
                    value_seen[random as usize] = true;
                    count_values_seen += 1;
                }

                // Prevent infinite loop on broken implementations
                counter += 1;
                if counter == 10_000 {
                    return Err(format!("RANDOM (with WIRED={}) was expected to return at least {} unique values, but we timed out", wired, expected_unique_value_count).into());
                }

                // As Random merely counts down a value (and is therefore not actually random), it is possible
                // to go in-phase and never return. Introduce some noise to get around that
                for _ in 0..(counter & 1023) | random {
                    unsafe { asm!("NOP") };
                }
            }
        }
        Ok(())
    }
}

pub struct WiredOutOfBoundsRandom {}

impl Test for WiredOutOfBoundsRandom {
    fn name(&self) -> &str { "Wired OOB/Random" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for wired in [32, 60, 61, 62, 63, 64, 65, 93, 94, 95, 96, 97] {
            unsafe { cop0::set_wired(wired); }
            soft_assert_eq(cop0::wired(), wired & 63, "WIRED preserves only lower bits")?;

            if (wired & 63) >= 32 {
                let mut min = 100;
                let mut max = 0;
                let mut counter = 0;
                loop {
                    let random = cop0::random();
                    if random < min {
                        min = random;
                    }
                    if random > max {
                        max = random;
                    }

                    // We expect to hit this pretty soon
                    if min < 10 && max > 54 {
                        break;
                    }

                    // Prevent infinite loop on broken implementations
                    counter += 1;
                    if counter == 10_000 {
                        return Err(format!("If WIRED>31, RANDOM can be [0..63]. Expected to see most of the range (at least 10..54), but saw [{}..{}]", min, max));
                    }

                    // As Random merely counts down a value (and is therefore not actually random), it is possible
                    // to go in-phase and never return. Introduce some noise to get around that
                    for _ in 0..(counter & 1023) | random {
                        unsafe { asm!("NOP") };
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct WriteRandomExpectIgnored {}

impl Test for WriteRandomExpectIgnored {
    fn name(&self) -> &str { "Write Random expect ignored" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cop0::set_random(100);
        let random = cop0::random();
        soft_assert_less(random, 64, "Random write should be ignored")?;

        Ok(())
    }
}

pub struct IndexMasking {}

impl Test for IndexMasking {
    fn name(&self) -> &str { "Index (masking)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0, 1, 5, 15, 30, 31, 32, 63, 64, 64, 1020, 63102, 0x0F000000, 0xFFFF0002, 0xFFFFFFFF, 0] {
            unsafe { cop0::set_index(value); }
            let expected = value & 0x8000003F;
            let readback = cop0::index();
            soft_assert_eq(readback, expected, format!("Index was written as {} and came back as {} but was expected as {}", value, readback, expected).as_str())?;
        }
        Ok(())
    }
}

pub struct EntryLo0Masking {}

impl Test for EntryLo0Masking {
    fn name(&self) -> &str { "EntryLo0 (masking)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0, 1, 5, 15, 30, 31, 32, 63, 64, 64, 1020, 63102, 0x0F000000, 0xFFFF0002, 0xFFFFFFFF, 0] {
            unsafe { cop0::set_entry_lo0(value); }
            let expected = value & 0x3FFFFFFF;
            let readback = cop0::entry_lo0();
            soft_assert_eq(readback, expected, format!("EntryLo0 was written as {} and came back as {} but was expected as {}", value, readback, expected).as_str())?;
        }
        Ok(())
    }
}

pub struct EntryLo0Masking64 {}

impl Test for EntryLo0Masking64 {
    fn name(&self) -> &str { "EntryLo0 (masking 64bit)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0, 1, 5, 15, 30, 31, 32, 63, 64, 64, 1020, 63102, 0x0F000000, 0xFFFF0002, 0x12345678_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF, 0] {
            unsafe { cop0::set_entry_lo0_64(value); }
            let expected = value & 0x3FFFFFFF;
            let readback = cop0::entry_lo0_64();
            soft_assert_eq(readback, expected, format!("EntryLo0 was written as {} and came back as {} but was expected as {}", value, readback, expected).as_str())?;
        }
        Ok(())
    }
}

pub struct EntryLo1Masking {}

impl Test for EntryLo1Masking {
    fn name(&self) -> &str { "EntryLo1 (masking)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0, 1, 5, 15, 30, 31, 32, 63, 64, 64, 1020, 63102, 0x0F000000, 0xFFFF0002, 0xFFFFFFFF, 0] {
            unsafe { cop0::set_entry_lo1(value); }
            let expected = value & 0x3FFFFFFF;
            let readback = cop0::entry_lo1();
            soft_assert_eq(readback, expected, format!("EntryLo1 was written as {} and came back as {} but was expected as {}", value, readback, expected).as_str())?;
        }
        Ok(())
    }
}

pub struct EntryLo1Masking64 {}

impl Test for EntryLo1Masking64 {
    fn name(&self) -> &str { "EntryLo1 (masking 64bit)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0, 1, 5, 15, 30, 31, 32, 63, 64, 64, 1020, 63102, 0x0F000000, 0xFFFF0002, 0x12345678_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF, 0] {
            unsafe { cop0::set_entry_lo1_64(value); }
            let expected = value & 0x3FFFFFFF;
            let readback = cop0::entry_lo1_64();
            soft_assert_eq(readback, expected, format!("EntryLo1 was written as {} and came back as {} but was expected as {}", value, readback, expected).as_str())?;
        }
        Ok(())
    }
}

pub struct EntryHiMasking {}

impl Test for EntryHiMasking {
    fn name(&self) -> &str { "EntryHi (masking)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0, 1, 5, 15, 30, 31, 32, 63, 64, 64, 1020, 63102, 0x0F000000, 0xFFFF0002, 0xFFFFFFFF, 0x12345678_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF, 0] {
            unsafe { cop0::set_entry_hi(value); }
            let expected = value & 0xC00000FF_FFFFE0FF;
            let readback = cop0::entry_hi();
            soft_assert_eq(readback, expected, format!("EntryHi was written as 0x{:x}", value).as_str())?;
        }
        Ok(())
    }
}

pub struct PageMaskMasking {}

impl Test for PageMaskMasking {
    fn name(&self) -> &str { "PageMask (masking)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0, 1, 5, 15, 30, 31, 32, 63, 64, 64, 1020, 63102, 0x0F000000, 0xFFFF0002, 0xFFFFFFFF, 0b0111 << 13, 0x017fc000, 0] {
            unsafe { cop0::set_pagemask(value); }
            let expected = value & 0x1FFE000;
            let readback = cop0::pagemask();
            soft_assert_eq(readback, expected, format!("PageMask was written as {} and came back as {} but was expected as {}", value, readback, expected).as_str())?;
        }
        Ok(())
    }
}

pub struct ConfigReadWrite {}

impl Test for ConfigReadWrite {
    fn name(&self) -> &str { "Config" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {

        // The following bits seem to be writeable:
        // 0-3
        // 15: Must be written as 1, otherwise screen corruption
        // 24-27: Must be written as 0, otherwise screen corruption
        // Everything else: Constant
        // Default value is 0x7006_E463

        for value in [0x0000_8000, 0xF0FF_FFFF, 0x7006_E463] {
            unsafe { cop0::set_config(value); }
            let expected = (value & 0x0F00_800F) | 0x7006_6460;
            let readback = cop0::config();
            soft_assert_eq(readback, expected, format!("Config was written as {} and came back as {} but was expected as {}", value, readback, expected).as_str())?;
        }
        Ok(())
    }
}

pub struct TLBWriteReadPageMask {}

impl Test for TLBWriteReadPageMask {
    fn name(&self) -> &str { "PageMask after TLBWI/TLBR" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {

        // Tuples of values to write and value that is expected when read back
        let values_expected = [
            // normal values
            (0b0000000000 << 13, 0b0000000000 << 13),  // 4k
            (0b0000000011 << 13, 0b0000000011 << 13),  // 16k
            (0b0000001111 << 13, 0b0000001111 << 13),  // 64k
            (0b0000111111 << 13, 0b0000111111 << 13),  // 256k
            (0b0011111111 << 13, 0b0011111111 << 13),  // 1M
            (0b1111111111 << 13, 0b1111111111 << 13),  // 4M
            // one bit missing - these get truncated
            (0b00000000001 << 13, 0b00000000000 << 13),
            (0b00000000111 << 13, 0b00000000011 << 13),
            (0b00000011111 << 13, 0b00000001111 << 13),
            (0b00001111111 << 13, 0b00000111111 << 13),
            (0b00111111111 << 13, 0b00011111111 << 13),
            // Just the higher bit of each pair is set - these count
            (0b00000000010 << 13, 0b00000000011 << 13),
            (0b00000000100 << 13, 0b00000000000 << 13),
            (0b00000001000 << 13, 0b00000001100 << 13),
            (0b00000100000 << 13, 0b00000110000 << 13),
            // Some higher bits set (which count) and some lower bits (which are ignored). First 11 is ignored (too many positions)
            (0b00_11_00_01_10_10_01 << 13, 0b00_11_00_00_11_11_00 << 13),
            (0x017fc000, 0x1ffe000),
        ];

        assert!(values_expected.len() < 32);

        for index in 0..values_expected.len() {
            unsafe {
                cop0::write_tlb(index as u32, values_expected[index].0, 0, 0, 0);
            }
        }
        for index in 0..values_expected.len() {
            unsafe {
                cop0::set_index(index as u32);
                cop0::tlbr();
                soft_assert_eq(cop0::pagemask(), values_expected[index].1, format!("PageMask readback for index {} (0x{:x} written) unexpected", index, values_expected[index].0).as_str())?;
            }
        }
        // Zero everything back out
        for index in 0..values_expected.len() {
            unsafe {
                cop0::write_tlb(index as u32, 0, 0, 0, 0);
            }
        }
        Ok(())
    }
}

pub struct TLBWriteReadBackEntry {}

impl TLBWriteReadBackEntry {
    fn test(&self, pagemask: u32, entry_lo0: (u32, u32), entry_lo1: (u32, u32), entry_hi: (u64, u64)) -> Result<(), String> {
        unsafe {
            cop0::write_tlb(0, pagemask, entry_lo0.0, entry_lo1.0, entry_hi.0);

            cop0::set_index(0);
            // Set to something else to ensure it actually is read back
            cop0::set_entry_lo0(0xFFFFFFFF);
            cop0::set_entry_lo1(0xFFFFFFFF);
            cop0::set_entry_hi(0xFFFFFFFF_FFFFFFFF);
            cop0::tlbr();
            soft_assert_eq(cop0::entry_lo0(), entry_lo0.1, format!("TLB readback for EntryLo0 (0x{:x} written) unexpected", entry_lo0.0).as_str())?;
            soft_assert_eq(cop0::entry_lo1(), entry_lo1.1, format!("TLB readback for EntryLo1 (0x{:x} written) unexpected", entry_lo1.0).as_str())?;
            soft_assert_eq(cop0::entry_hi(), entry_hi.1, format!("TLB readback for EntryHi (0x{:x} written) unexpected", entry_hi.0).as_str())?;
        }
        Ok(())
    }
}

impl Test for TLBWriteReadBackEntry {
    fn name(&self) -> &str { "EntryLo0/1/Hi after TLBWI/TLBR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        self.test(0b00_00_00_00_00 << 13, (0x00000000, 0x00000000), (0x00000000, 0x00000000), (0x00000000_00000000, 0x00000000_00000000))?;
        self.test(0b00_00_00_00_00 << 13, (0x3FFFFFFF, 0x03FFFFFE), (0x00000000, 0x00000000), (0x00000000_00000000, 0x00000000_00000000))?;
        self.test(0b00_00_00_00_00 << 13, (0x00000000, 0x00000000), (0x3FFFFFFF, 0x03FFFFFE), (0x00000000_00000000, 0x00000000_00000000))?;

        self.test(0b00_00_00_00_00 << 13, (0x3FFFFFFF, 0x03FFFFFF), (0x3FFFFFFF, 0x03FFFFFF), (0x00000000_FFFFE0FF, 0x00000000_FFFFE0FF))?;
        self.test(0b00_00_00_00_00 << 13, (0x3FFFFFFF, 0x03FFFFFF), (0x3FFFFFFF, 0x03FFFFFF), (0xC00000FF_FFFFE0FF, 0xC00000FF_FFFFE0FF))?;
        self.test(0b00_00_00_00_11 << 13, (0x3FFFFFFF, 0x03FFFFFF), (0x3FFFFFFF, 0x03FFFFFF), (0x00000000_FFFFE0FF, 0x00000000_FFFF80FF))?;
        self.test(0b00_00_00_11_11 << 13, (0x3FFFFFFF, 0x03FFFFFF), (0x3FFFFFFF, 0x03FFFFFF), (0x00000000_FFFFE0FF, 0x00000000_FFFE00FF))?;
        self.test(0b00_00_11_11_11 << 13, (0x3FFFFFFF, 0x03FFFFFF), (0x3FFFFFFF, 0x03FFFFFF), (0x00000000_FFFFE0FF, 0x00000000_FFF800FF))?;
        self.test(0b00_11_11_11_11 << 13, (0x3FFFFFFF, 0x03FFFFFF), (0x3FFFFFFF, 0x03FFFFFF), (0x00000000_FFFFE0FF, 0x00000000_FFE000FF))?;
        self.test(0b11_11_11_11_11 << 13, (0x3FFFFFFF, 0x03FFFFFF), (0x3FFFFFFF, 0x03FFFFFF), (0x00000000_FFFFE0FF, 0x00000000_FF8000FF))?;
        self.test(0b11_11_11_11_11 << 13, (0x3FFFFFFF, 0x03FFFFFF), (0x3FFFFFFF, 0x03FFFFFF), (0xC00000FF_FFFFE0FF, 0xC00000FF_FF8000FF))?;

        self.test(0b11_11_11_11_00 << 13, (0x3FFFFFFF, 0x03FFFFFF), (0x3FFFFFFF, 0x03FFFFFF), (0x00000000_FFFFE0FF, 0x00000000_FF8060FF))?;
        self.test(0b11_11_11_10_01 << 13, (0x3FFFFFFF, 0x03FFFFFF), (0x3FFFFFFF, 0x03FFFFFF), (0x00000000_FFFFE0FF, 0x00000000_FF8060FF))?;

        Ok(())
    }
}

pub struct TLBUseTestRead0 {}

impl Test for TLBUseTestRead0 {
    fn name(&self) -> &str { "TLB: Use and test, match via global (0)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::clear_tlb(); }
        // Setup 16k mapping from 0x0DEA0000 to MemoryMap::HEAP_END
        unsafe {
            cop0::write_tlb(
                10,
                0b11 << 13,
                make_entry_lo(true, true, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_lo(true, false, false, 0, 0),
                make_entry_hi(1, 0xDEA0 >> 1, 0))
        }

        // Set a different ASID to make sure it doesn't match
        unsafe { cop0::set_entry_hi(2); }

        // Write a value without TLB
        unsafe {
            (MemoryMap::HEAP_END_VIRTUAL_UNCACHED as *mut u32).write_volatile(0xBADF00D);
            ((MemoryMap::HEAP_END_VIRTUAL_UNCACHED + 0x18) as *mut u32).write_volatile(0xDECAFBAD);
            ((MemoryMap::HEAP_END_VIRTUAL_UNCACHED + 0x3FFC) as *mut u32).write_volatile(0x12345678); // last value inside of the 16kb page
        }

        // Read it back using the TLB
        soft_assert_eq(unsafe { (0x0DEA0000 as *mut u32).read_volatile() }, 0xBADF00D, "Value read back through TLB mapped memory")?;
        soft_assert_eq(unsafe { (0x0DEA0018 as *mut u32).read_volatile() }, 0xDECAFBAD, "Value read back through TLB mapped memory")?;
        soft_assert_eq(unsafe { (0x0DEA3FFC as *mut u32).read_volatile() }, 0x12345678, "Value read back through TLB mapped memory")?;

        Ok(())
    }
}

pub struct TLBUseTestRead1 {}

impl Test for TLBUseTestRead1 {
    fn name(&self) -> &str { "TLB: Use and test reading, match via global (1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::clear_tlb(); }
        // Setup 16k mapping from 0x0DEA0000 to MemoryMap::HEAP_END
        unsafe {
            cop0::write_tlb(
                10,
                0b11 << 13,
                make_entry_lo(true, false, false, 0, 0),
                make_entry_lo(true, true, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_hi(1, 0xDEA0 >> 1, 0))
        }

        // Set a different ASID to make sure it doesn't match
        unsafe { cop0::set_entry_hi(2); }

        // Write a value without TLB
        unsafe {
            (MemoryMap::HEAP_END_VIRTUAL_UNCACHED as *mut u32).write_volatile(0xBADF00D);
            ((MemoryMap::HEAP_END_VIRTUAL_UNCACHED + 0x18) as *mut u32).write_volatile(0xDECAFBAD);
            ((MemoryMap::HEAP_END_VIRTUAL_UNCACHED + 0x3FFC) as *mut u32).write_volatile(0x12345678); // last value inside of the 16kb page
        }

        // Read it back using the TLB
        soft_assert_eq(unsafe { (0x0DEA4000 as *mut u32).read_volatile() }, 0xBADF00D, "Value read back through TLB mapped memory")?;
        soft_assert_eq(unsafe { (0x0DEA4018 as *mut u32).read_volatile() }, 0xDECAFBAD, "Value read back through TLB mapped memory")?;
        soft_assert_eq(unsafe { (0x0DEA7FFC as *mut u32).read_volatile() }, 0x12345678, "Value read back through TLB mapped memory")?;

        Ok(())
    }
}

pub struct TLBUseTestReadMatchViaASID {}

impl Test for TLBUseTestReadMatchViaASID {
    fn name(&self) -> &str { "TLB: Use and test reading, match via ASID" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::clear_tlb(); }
        // 16k
        unsafe {
            cop0::write_tlb(
                10,
                0b11 << 13,
                make_entry_lo(false, true, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_lo(false, false, false, 0, 0),
                make_entry_hi(1, 0xDEA0 >> 1, 0))
        }

        // Set a matching ASID - this will make the CPU ignore the global bit
        unsafe { cop0::set_entry_hi(1); }

        // Write a value without TLB
        unsafe { (0xA0200018 as *mut u32).write_volatile(0xDECAFBAD); }

        // Read it back using the TLB
        soft_assert_eq(unsafe { (0x0DEA0018 as *mut u32).read_volatile() }, 0xDECAFBAD, "Value read back through TLB mapped memory")?;

        Ok(())
    }
}

pub struct TLBPMatch {}

impl Test for TLBPMatch {
    fn name(&self) -> &str { "TLB: Use and match via TLBP" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::clear_tlb(); }
        unsafe {
            cop0::write_tlb(
                4,
                0b11 << 13,
                make_entry_lo(false, false, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_lo(false, false, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_hi(1, 0xDEA3, 1));

            cop0::write_tlb(
                29,
                0b1111 << 13,
                make_entry_lo(true, false, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_lo(false, false, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_hi(1, 0x7FF_0000, 2));
            cop0::write_tlb(
                30,
                0b1111 << 13,
                make_entry_lo(false, false, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_lo(true, false, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_hi(2, 0x0001, 3));
            cop0::write_tlb(
                31,
                0b1111 << 13,
                make_entry_lo(true, false, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_lo(true, false, false, 0, (MemoryMap::HEAP_END >> 12) as u32),
                make_entry_hi(3, 0x7FF_0002, 0));
        }

        // TLBP ignores entry_lo and pagemask. Set them all to 0 to prove
        unsafe {
            cop0::set_pagemask(0);
            cop0::set_entry_lo0(0);
            cop0::set_entry_lo1(0);
        }

        // An entry is a match if all three are true:
        // - ASID matches OR both lo0 and lo1 are global
        // - VPN (masked) matches
        // - R matches

        // Match via asid, but not global
        unsafe { cop0::set_index(1); }
        unsafe { cop0::set_entry_hi(make_entry_hi(1, 0xDEA3, 1)); }
        cop0::tlbp();
        soft_assert_eq(cop0::index(), 4, "TLBP result (match via ASID)")?;

        // Use wrong R and observe that it no longer matches
        unsafe { cop0::set_entry_hi(make_entry_hi(1, 0xDEA3, 0)); }
        cop0::tlbp();
        soft_assert_eq(cop0::index(), 0x80000000, "TLBP result (match via ASID, incorrect R)")?;

        // Match both global, with asid incorrect
        unsafe { cop0::set_entry_hi(make_entry_hi(1, 0x7FF_0002, 0)); }
        cop0::tlbp();
        soft_assert_eq(cop0::index(), 31, "TLBP result (ASID mismatch, but global is set)")?;

        // Try to match with only one global on (with asid incorrect). This shouldn't match
        unsafe { cop0::set_entry_hi(make_entry_hi(0, 0x0001, 3)); }
        cop0::tlbp();
        soft_assert_eq(cop0::index(), 0x80000000, "TLBP result (ASID mismatch, and only one of the global bits enabled)")?;

        // Again, but this time the other global bit
        unsafe { cop0::set_entry_hi(make_entry_hi(0, 0x7FF_0000, 2)); }
        cop0::tlbp();
        soft_assert_eq(cop0::index(), 0x80000000, "TLBP result (ASID mismatch, and only one of the global bits enabled)")?;

        // Finally, actually check the masked VPN
        unsafe { cop0::set_entry_hi(make_entry_hi(1, 0xDEA0, 1)); }
        cop0::tlbp();
        soft_assert_eq(cop0::index(), 4, "TLBP result (masked VPN 1)")?;

        unsafe { cop0::set_entry_hi(make_entry_hi(1, 0xDEA1, 1)); }
        cop0::tlbp();
        soft_assert_eq(cop0::index(), 4, "TLBP result (masked VPN 1)")?;

        unsafe { cop0::set_entry_hi(make_entry_hi(1, 0xDEA2, 1)); }
        cop0::tlbp();
        soft_assert_eq(cop0::index(), 4, "TLBP result (masked VPN 2)")?;

        unsafe { cop0::set_entry_hi(make_entry_hi(1, 0xDEA4, 1)); }
        cop0::tlbp();
        soft_assert_eq(cop0::index(), 0x80000000, "TLBP result (masked VPN 3)")?;

        Ok(())
    }
}

