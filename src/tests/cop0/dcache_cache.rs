use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::cop0;
use crate::cop0::{
    DCACHE_CREATE_DIRTY_EXCLUSIVE, DCACHE_HIT_INVALIDATE, DCACHE_HIT_WRITEBACK,
    DCACHE_HIT_WRITEBACK_INVALIDATE, DCACHE_INDEX_LOAD_TAG, DCACHE_INDEX_STORE_TAG,
    DCACHE_INDEX_WRITEBACK_INVALIDATE,
};
use crate::MemoryMap;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

use super::cache_common;

#[repr(C, align(16))]
struct DcacheLineSlot {
    words: [u32; 4],
}

static mut DCACHE_LINE_SLOT: DcacheLineSlot = DcacheLineSlot { words: [0; 4] };

fn dcache_line_ptrs() -> (usize, *mut u32) {
    unsafe {
        let p = DCACHE_LINE_SLOT.words.as_mut_ptr();
        (p as usize, MemoryMap::uncached_mut(p))
    }
}

fn dcache_tag_lo(valid: bool, _dirty: bool, physical_address: u32) -> u32 {
    let pstate = if valid { 3u32 } else { 0 };
    let pfn = (physical_address >> 12) & 0x000F_FFFF;
    (pstate << 6) | (pfn << 8)
}

fn tag_lo_primary_state(tag_lo: u32) -> u32 {
    (tag_lo >> 6) & 3
}

pub struct DcacheLoadWordMatchesUncachedInit;

impl Test for DcacheLoadWordMatchesUncachedInit {
    fn name(&self) -> &str { "DCACHE: cached LW observes word written uncached" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, uncached) = dcache_line_ptrs();
            unsafe {
                cop0::cache::<DCACHE_INDEX_WRITEBACK_INVALIDATE, 0>(cached);
                uncached.write_volatile(0xABCDEF01u32);
            }
            let w: u32;
            unsafe {
                asm!("lw {out}, 0({addr})", out = out(reg) w, addr = in(reg) cached);
            }
            soft_assert_eq(w, 0xABCDEF01u32, "LW after line fill")?;
            Ok(())
        })
    }
}

pub struct DcacheStaleAfterUncachedStore;

impl Test for DcacheStaleAfterUncachedStore {
    fn name(&self) -> &str { "DCACHE: cached LW stale after uncached SW until invalidate" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, uncached) = dcache_line_ptrs();
            unsafe {
                cop0::cache::<DCACHE_INDEX_WRITEBACK_INVALIDATE, 0>(cached);
                uncached.write_volatile(0x01020304u32);
            }
            let w1: u32;
            unsafe {
                asm!("lw {out}, 0({addr})", out = out(reg) w1, addr = in(reg) cached);
            }
            soft_assert_eq(w1, 0x01020304u32, "fill")?;
            unsafe {
                uncached.write_volatile(0xAABBCCDDu32);
            }
            let w2: u32;
            unsafe {
                asm!("lw {out}, 0({addr})", out = out(reg) w2, addr = in(reg) cached);
            }
            soft_assert_eq(w2, 0x01020304u32, "cached still stale")?;
            soft_assert_eq(unsafe { uncached.read_volatile() }, 0xAABBCCDDu32, "uncached sees RAM")?;
            unsafe {
                cop0::cache::<DCACHE_HIT_INVALIDATE, 0>(cached);
            }
            let w3: u32;
            unsafe {
                asm!("lw {out}, 0({addr})", out = out(reg) w3, addr = in(reg) cached);
            }
            soft_assert_eq(w3, 0xAABBCCDDu32, "after hit invalidate refill")?;
            Ok(())
        })
    }
}

pub struct DcacheHitWritebackWritesDirtyToRam;

impl Test for DcacheHitWritebackWritesDirtyToRam {
    fn name(&self) -> &str { "DCACHE: hit writeback pushes dirty line to RAM" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, uncached) = dcache_line_ptrs();
            unsafe {
                cop0::cache::<DCACHE_INDEX_WRITEBACK_INVALIDATE, 0>(cached);
                uncached.write_volatile(0xAAAAAAAAu32);
            }
            let fill: u32;
            unsafe {
                asm!("lw {out}, 0({addr})", out = out(reg) fill, addr = in(reg) cached);
            }
            soft_assert_eq(fill, 0xAAAAAAAAu32, "line fill")?;
            unsafe {
                asm!(
                    "sw {val}, 0({addr})",
                    val = in(reg) 0xBBBBBBBBu32,
                    addr = in(reg) cached,
                );
            }
            soft_assert_eq(unsafe { uncached.read_volatile() }, 0xAAAAAAAAu32, "RAM before writeback")?;
            unsafe {
                cop0::cache::<DCACHE_HIT_WRITEBACK, 0>(cached);
            }
            soft_assert_eq(unsafe { uncached.read_volatile() }, 0xBBBBBBBBu32, "RAM after hit writeback")?;
            Ok(())
        })
    }
}

pub struct DcacheStoreTagThenLoadTag;

impl Test for DcacheStoreTagThenLoadTag {
    fn name(&self) -> &str { "CACHE dcache store tag then load tag (TagLo)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, _) = dcache_line_ptrs();
            let phys = MemoryMap::uncached_to_physical_mut(cached as *mut u32) as u32;
            let want = dcache_tag_lo(true, false, phys);
            unsafe {
                cop0::cache::<DCACHE_INDEX_WRITEBACK_INVALIDATE, 0>(cached);
                cop0::set_tag_lo(want);
                cop0::cache::<DCACHE_INDEX_STORE_TAG, 0>(cached);
                cop0::cache::<DCACHE_INDEX_LOAD_TAG, 0>(cached);
                cop0::sync();
            }
            soft_assert_eq(cop0::tag_lo(), want, "TagLo after store tag + load tag")?;
            Ok(())
        })
    }
}

pub struct DcacheIndexWbInvClearsValidInTagLo;

impl Test for DcacheIndexWbInvClearsValidInTagLo {
    fn name(&self) -> &str { "CACHE dcache index writeback invalidate clears valid (TagLo via load tag)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, _) = dcache_line_ptrs();
            let phys = MemoryMap::uncached_to_physical_mut(cached as *mut u32) as u32;
            let before_store = dcache_tag_lo(true, false, phys);
            unsafe {
                cop0::cache::<DCACHE_INDEX_WRITEBACK_INVALIDATE, 0>(cached);
                cop0::set_tag_lo(before_store);
                cop0::cache::<DCACHE_INDEX_STORE_TAG, 0>(cached);
                cop0::cache::<DCACHE_INDEX_LOAD_TAG, 0>(cached);
                cop0::sync();
            }
            soft_assert_eq(tag_lo_primary_state(cop0::tag_lo()), 3, "PState before invalidate (valid)")?;
            unsafe {
                cop0::cache::<DCACHE_INDEX_WRITEBACK_INVALIDATE, 0>(cached);
                cop0::cache::<DCACHE_INDEX_LOAD_TAG, 0>(cached);
                cop0::sync();
            }
            let t = cop0::tag_lo();
            soft_assert_eq(t, dcache_tag_lo(false, false, phys), "TagLo after wb-invalidate + load tag")?;
            Ok(())
        })
    }
}

pub struct DcacheCreateDirtyExclusiveThenStore;

impl Test for DcacheCreateDirtyExclusiveThenStore {
    fn name(&self) -> &str { "CACHE dcache create dirty exclusive (0x0D) then store/writeback" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, uncached) = dcache_line_ptrs();
            unsafe {
                cop0::cache::<DCACHE_INDEX_WRITEBACK_INVALIDATE, 0>(cached);
                uncached.write_volatile(0u32);
                uncached.add(1).write_volatile(0u32);
                uncached.add(2).write_volatile(0u32);
                uncached.add(3).write_volatile(0u32);
                cop0::cache::<DCACHE_CREATE_DIRTY_EXCLUSIVE, 0>(cached);
                cop0::sync();
            }
            unsafe {
                cop0::cache::<DCACHE_INDEX_LOAD_TAG, 0>(cached);
                cop0::sync();
            }
            soft_assert_eq(
                tag_lo_primary_state(cop0::tag_lo()),
                3,
                "DPCS after create dirty exclusive (valid dirty)",
            )?;
            unsafe {
                asm!(
                    "sw {val}, 0({addr})",
                    val = in(reg) 0xC0FFEEu32,
                    addr = in(reg) cached,
                );
                cop0::cache::<DCACHE_HIT_WRITEBACK, 0>(cached);
            }
            soft_assert_eq(unsafe { uncached.read_volatile() }, 0xC0FFEEu32, "RAM after hit writeback")?;
            Ok(())
        })
    }
}

pub struct DcacheHitWritebackInvalidate;

impl Test for DcacheHitWritebackInvalidate {
    fn name(&self) -> &str { "CACHE dcache hit writeback invalidate (0x15)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, uncached) = dcache_line_ptrs();
            let phys = MemoryMap::uncached_to_physical_mut(cached as *mut u32) as u32;
            unsafe {
                cop0::cache::<DCACHE_INDEX_WRITEBACK_INVALIDATE, 0>(cached);
                uncached.write_volatile(0xAAAABBBBu32);
            }
            let fill: u32;
            unsafe {
                asm!("lw {out}, 0({addr})", out = out(reg) fill, addr = in(reg) cached);
            }
            soft_assert_eq(fill, 0xAAAABBBBu32, "line fill")?;
            unsafe {
                asm!(
                    "sw {val}, 0({addr})",
                    val = in(reg) 0xCCCCDDDDu32,
                    addr = in(reg) cached,
                );
            }
            soft_assert_eq(unsafe { uncached.read_volatile() }, 0xAAAABBBBu32, "RAM before 0x15")?;
            unsafe {
                cop0::cache::<DCACHE_HIT_WRITEBACK_INVALIDATE, 0>(cached);
            }
            soft_assert_eq(unsafe { uncached.read_volatile() }, 0xCCCCDDDDu32, "RAM after 0x15")?;
            unsafe {
                cop0::cache::<DCACHE_INDEX_LOAD_TAG, 0>(cached);
                cop0::sync();
            }
            soft_assert_eq(
                cop0::tag_lo(),
                dcache_tag_lo(false, false, phys),
                "TagLo invalid after hit wb-invalidate + load tag",
            )?;
            Ok(())
        })
    }
}
