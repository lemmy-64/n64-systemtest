pub mod dma;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::MemoryMap;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Write/reading from SPMEM:
// - SW and LW works as expected
// - SH/SB are broken: They overwrite the whole 32 bit, filling everything that isn't written with zeroes
// - SD is broken: It only writes the upper 32 bit of the value, touching only 4 bytes
// - LB/LH work as expected
// - LD crashes outright (no test for that)
// Going out of bounds wraps the memory around (until the real end of 0x04040000)

pub struct SW {}

impl Test for SW {
    fn name(&self) -> &str { "spmem: SW" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let spmem = MemoryMap::uncached_spmem_address::<u32>(0x0);
        unsafe {
            spmem.write_volatile(0x01234567);
            spmem.add(1).write_volatile(0x89ABCDEF);
        }

        soft_assert_eq(unsafe { spmem.read_volatile() }, 0x01234567, "Reading 32 bit from SPMEM[0]")?;
        soft_assert_eq(unsafe { spmem.add(1).read_volatile() }, 0x89ABCDEF, "Reading 32 bit from SPMEM[4]")?;
        Ok(())
    }
}

pub struct SH {}

impl Test for SH {
    fn name(&self) -> &str { "spmem: SH" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let spmem = MemoryMap::uncached_spmem_address::<u32>(0x0);

        // Preset the memory area
        unsafe {
            spmem.add(0).write_volatile(0xDEADBEEF);
            spmem.add(1).write_volatile(0xBADDECAF);
            spmem.add(2).write_volatile(0xABABABAB);
        }

        unsafe {
            asm!("
                .set noat
                .set noreorder

                LUI $3, 0x1234
                ORI $3, $3, 0x5678
                SH $3, 0($2)
                SH $3, 6($2)
            ", in("$2") spmem, out("$3") _)
        }

        soft_assert_eq(unsafe { spmem.add(0).read_volatile() }, 0x56780000, "Reading 32 bit from SPMEM[0]")?;
        soft_assert_eq(unsafe { spmem.add(1).read_volatile() }, 0x12345678, "Reading 32 bit from SPMEM[4]")?;
        Ok(())
    }
}

pub struct SB {}

impl Test for SB {
    fn name(&self) -> &str { "spmem: SB" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let spmem = MemoryMap::uncached_spmem_address::<u32>(0x0);

        // Preset the memory area
        unsafe {
            spmem.add(0).write_volatile(0);
            spmem.add(1).write_volatile(0);
            spmem.add(2).write_volatile(0);
            spmem.add(3).write_volatile(0);
        }

        unsafe {
            asm!("
                .set noat
                .set noreorder

                LUI $3, 0x1234
                ORI $3, $3, 0x5678
                SB $3, 0($2)
                SB $3, 5($2)
                SB $3, 10($2)
                SB $3, 15($2)
            ", in("$2") spmem, out("$3") _)
        }

        soft_assert_eq(unsafe { spmem.add(0).read_volatile() }, 0x78000000, "Reading 32 bit from SPMEM[0]")?;
        soft_assert_eq(unsafe { spmem.add(1).read_volatile() }, 0x56780000, "Reading 32 bit from SPMEM[4]")?;
        soft_assert_eq(unsafe { spmem.add(2).read_volatile() }, 0x34567800, "Reading 32 bit from SPMEM[8]")?;
        soft_assert_eq(unsafe { spmem.add(3).read_volatile() }, 0x12345678, "Reading 32 bit from SPMEM[12]")?;
        Ok(())
    }
}

pub struct SD {}

impl Test for SD {
    fn name(&self) -> &str { "spmem: SD" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let spmem = MemoryMap::uncached_spmem_address::<u32>(0x0);

        // Preset the memory area
        unsafe {
            spmem.add(0).write_volatile(0xDEADBEEF);
            spmem.add(1).write_volatile(0xBADDECAF);
            spmem.add(2).write_volatile(0xABABABAB);
            spmem.add(3).write_volatile(0xCDCDCDCD);
            spmem.add(4).write_volatile(0xDEDEDEDE);
            spmem.add(5).write_volatile(0xEFEFEFEF);
        }

        let spmem_64 = MemoryMap::uncached_spmem_address::<u64>(0x0);
        unsafe {
            // Write to 1st value
            spmem_64.add(0).write_volatile(0xABCDEF98_76543210);
        }

        soft_assert_eq(unsafe { spmem.add(0).read_volatile() }, 0xABCDEF98, "Reading 32 bit from SPMEM[0]")?;
        soft_assert_eq(unsafe { spmem.add(1).read_volatile() }, 0xBADDECAF, "Reading 32 bit from SPMEM[4]")?;
        soft_assert_eq(unsafe { spmem.add(2).read_volatile() }, 0xABABABAB, "Reading 32 bit from SPMEM[8]")?;
        soft_assert_eq(unsafe { spmem.add(3).read_volatile() }, 0xCDCDCDCD, "Reading 32 bit from SPMEM[12]")?;
        soft_assert_eq(unsafe { spmem.add(4).read_volatile() }, 0xDEDEDEDE, "Reading 32 bit from SPMEM[12]")?;
        soft_assert_eq(unsafe { spmem.add(5).read_volatile() }, 0xEFEFEFEF, "Reading 32 bit from SPMEM[12]")?;
        Ok(())
    }
}

pub struct LB {}

impl Test for LB {
    fn name(&self) -> &str { "spmem: LB" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let spmem = MemoryMap::uncached_spmem_address::<u32>(0x0);

        // Preset the memory area
        unsafe {
            spmem.add(0).write_volatile(0x01234567);
            spmem.add(1).write_volatile(0x89ABCDEF);
        }

        let spmem_8 = MemoryMap::uncached_spmem_address::<u8>(0x0);

        soft_assert_eq(unsafe { spmem_8.add(0).read_volatile() }, 0x01, "Reading 8 bit from SPMEM[0]")?;
        soft_assert_eq(unsafe { spmem_8.add(1).read_volatile() }, 0x23, "Reading 8 bit from SPMEM[1]")?;
        soft_assert_eq(unsafe { spmem_8.add(2).read_volatile() }, 0x45, "Reading 8 bit from SPMEM[2]")?;
        soft_assert_eq(unsafe { spmem_8.add(3).read_volatile() }, 0x67, "Reading 8 bit from SPMEM[3]")?;
        soft_assert_eq(unsafe { spmem_8.add(4).read_volatile() }, 0x89, "Reading 8 bit from SPMEM[4]")?;
        soft_assert_eq(unsafe { spmem_8.add(5).read_volatile() }, 0xAB, "Reading 8 bit from SPMEM[5]")?;
        soft_assert_eq(unsafe { spmem_8.add(6).read_volatile() }, 0xCD, "Reading 8 bit from SPMEM[6]")?;
        soft_assert_eq(unsafe { spmem_8.add(7).read_volatile() }, 0xEF, "Reading 8 bit from SPMEM[7]")?;
        Ok(())
    }
}

pub struct LH {}

impl Test for LH {
    fn name(&self) -> &str { "spmem: LH" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let spmem = MemoryMap::uncached_spmem_address::<u32>(0x0);

        // Preset the memory area
        unsafe {
            spmem.add(0).write_volatile(0x01234567);
            spmem.add(1).write_volatile(0x89ABCDEF);
        }

        let spmem_16 = MemoryMap::uncached_spmem_address::<u16>(0x0);

        soft_assert_eq(unsafe { spmem_16.add(0).read_volatile() }, 0x0123, "Reading 16 bit from SPMEM[0]")?;
        soft_assert_eq(unsafe { spmem_16.add(1).read_volatile() }, 0x4567, "Reading 16 bit from SPMEM[2]")?;
        soft_assert_eq(unsafe { spmem_16.add(2).read_volatile() }, 0x89AB, "Reading 16 bit from SPMEM[3]")?;
        soft_assert_eq(unsafe { spmem_16.add(3).read_volatile() }, 0xCDEF, "Reading 16 bit from SPMEM[4]")?;
        Ok(())
    }
}

pub struct SWOutOfBounds {}

impl Test for SWOutOfBounds {
    fn name(&self) -> &str { "spmem: SW (out of bounds)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // SPMEM DMEM and IMEM repeat froom 0x04000000 to 0x04040000
        let spmem0000 = MemoryMap::uncached_spmem_address::<u32>(0x0000);
        let spmem1000 = MemoryMap::uncached_spmem_address::<u32>(0x1000);
        let spmem_last_cycle = MemoryMap::uncached_spmem_address::<u32>(0x3E000);
        unsafe {
            spmem0000.write_volatile(0x01234567);
            spmem1000.write_volatile(0x89ABCDEF);

            // This one overwrites 0
            spmem_last_cycle.write_volatile(0x76543210);
        }

        soft_assert_eq(unsafe { spmem0000.read_volatile() }, 0x76543210, "Reading 32 bit from SPMEM[0x0000]")?;
        soft_assert_eq(unsafe { spmem1000.read_volatile() }, 0x89ABCDEF, "Reading 32 bit from SPMEM[0x1000]")?;
        soft_assert_eq(unsafe { spmem_last_cycle.read_volatile() }, 0x76543210, "Reading 32 bit from SPMEM[0x3E000]")?;

        Ok(())
    }
}
