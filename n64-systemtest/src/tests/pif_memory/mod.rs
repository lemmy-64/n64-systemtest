use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::MemoryMap;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Write/reading from PIFRAM:
// - SW and LW works as expected
// - SH/SB are broken: They overwrite the whole 32 bit, filling everything that isn't written with zeroes
// - LB/LH work as expected
// - LD and SD are untested

pub struct SW {}

impl Test for SW {
    fn name(&self) -> &str { "pifram: SW" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pifram = MemoryMap::uncached_pifram_address::<u32>(0x0);
        unsafe {
            pifram.write_volatile(0x01634567);
        }

        soft_assert_eq(unsafe { pifram.read_volatile() }, 0x01634567, "Reading 32 bit from PIFRAM[0]")?;
        Ok(())
    }
}

pub struct SH0 {}

impl Test for SH0 {
    fn name(&self) -> &str { "pifram: SH (offset 0)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pifram = MemoryMap::uncached_pifram_address::<u32>(0x0);

        unsafe {
            asm!("
                .set noat
                .set noreorder

                LUI $3, 0x4241
                ORI $3, $3, 0x31E2
                SH $3, 0($2)
            ", in("$2") pifram, out("$3") _)
        }

        soft_assert_eq(unsafe { pifram.add(0).read_volatile() }, 0x31E20000, "Reading 32 bit from PIFRAM[0]")?;
        Ok(())
    }
}

pub struct SH2 {}

impl Test for SH2 {
    fn name(&self) -> &str { "pifram: SH (offset 2)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pifram = MemoryMap::uncached_pifram_address::<u32>(0x0);

        unsafe {
            asm!("
                .set noat
                .set noreorder

                LUI $3, 0x4241
                ORI $3, $3, 0x31A2
                SH $3, 2($2)
            ", in("$2") pifram, out("$3") _)
        }

        soft_assert_eq(unsafe { pifram.add(0).read_volatile() }, 0x424131A2, "Reading 32 bit from PIFRAM[0]")?;
        Ok(())
    }
}

pub struct SB {}

impl Test for SB {
    fn name(&self) -> &str { "pifram: SB" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pifram = MemoryMap::uncached_pifram_address::<u32>(0x0);

        unsafe {
            asm!("
                .set noat
                .set noreorder

                LUI $3, 0x1234
                ORI $3, $3, 0x5678
                SB $3, 1($2)
            ", in("$2") pifram, out("$3") _)
        }

        soft_assert_eq(unsafe { pifram.add(0).read_volatile() }, 0x56780000, "Reading 32 bit from PIFRAM[0]")?;
        Ok(())
    }
}

pub struct LB {}

impl Test for LB {
    fn name(&self) -> &str { "pifram: LB" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pifram = MemoryMap::uncached_pifram_address::<u32>(0x0);

        // Preset the memory area
        unsafe {
            pifram.add(0).write_volatile(0x01634567);
        }

        let pifram_8 = MemoryMap::uncached_pifram_address::<u8>(0x0);

        soft_assert_eq(unsafe { pifram_8.add(0).read_volatile() }, 0x01, "Reading 8 bit from PIFRAM[0]")?;
        soft_assert_eq(unsafe { pifram_8.add(1).read_volatile() }, 0x63, "Reading 8 bit from PIFRAM[1]")?;
        soft_assert_eq(unsafe { pifram_8.add(2).read_volatile() }, 0x45, "Reading 8 bit from PIFRAM[2]")?;
        soft_assert_eq(unsafe { pifram_8.add(3).read_volatile() }, 0x67, "Reading 8 bit from PIFRAM[3]")?;
        Ok(())
    }
}

pub struct LH {}

impl Test for LH {
    fn name(&self) -> &str { "pifram: LH" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let pifram = MemoryMap::uncached_pifram_address::<u32>(0x0);

        // Preset the memory area
        unsafe {
            pifram.add(0).write_volatile(0x01634567);
        }

        let pifram_16 = MemoryMap::uncached_pifram_address::<u16>(0x0);

        soft_assert_eq(unsafe { pifram_16.add(0).read_volatile() }, 0x0163, "Reading 16 bit from PIFRAM[0]")?;
        soft_assert_eq(unsafe { pifram_16.add(1).read_volatile() }, 0x4567, "Reading 16 bit from PIFRAM[2]")?;
        Ok(())
    }
}
