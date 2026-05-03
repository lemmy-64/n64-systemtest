use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::cop0;
use super::cache_common;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

const ICACHE_LOAD_TAG: u8 = 4;
const ICACHE_STORE_TAG: u8 = 8;

fn icache_tag_lo(valid: bool, physical_address: u32) -> u32 {
    let pstate = if valid { 2u32 } else { 0 };
    let pfn = (physical_address >> 12) & 0x000F_FFFF;
    (pstate << 6) | (pfn << 8)
}

fn tag_lo_cpcs(tag_lo: u32) -> u32 {
    (tag_lo >> 6) & 3
}

fn run_icache_test(body: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
    cache_common::run_cache_isolated_test(body)
}

pub struct IcacheStoreTagThenLoadTag;

impl Test for IcacheStoreTagThenLoadTag {
    fn name(&self) -> &str { "CACHE icache store tag then load tag (TagLo)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_icache_test(|| {
            let addr = 0x8000_1000usize;
            let phys = 0x0000_1000u32;
            let want = icache_tag_lo(true, phys);
            unsafe {
                cop0::cache::<{ cop0::ICACHE_INDEX_INVALIDATE }, 0>(addr);
                cop0::set_tag_lo(want);
                cop0::cache::<ICACHE_STORE_TAG, 0>(addr);
                cop0::cache::<ICACHE_LOAD_TAG, 0>(addr);
                cop0::sync();
            }
            let got = cop0::tag_lo();
            soft_assert_eq(got, want, "TagLo after store tag + load tag")?;
            Ok(())
        })
    }
}

pub struct IcacheIndexInvalidateClearsValidInTagLo;

impl Test for IcacheIndexInvalidateClearsValidInTagLo {
    fn name(&self) -> &str { "CACHE icache index invalidate clears valid (TagLo via load tag)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_icache_test(|| {
            let addr = 0x8000_2000usize;
            let phys = 0x0000_2000u32;
            let before_store = icache_tag_lo(true, phys);
            unsafe {
                cop0::cache::<{ cop0::ICACHE_INDEX_INVALIDATE }, 0>(addr);
                cop0::set_tag_lo(before_store);
                cop0::cache::<ICACHE_STORE_TAG, 0>(addr);
                cop0::cache::<ICACHE_LOAD_TAG, 0>(addr);
                cop0::sync();
            }
            soft_assert_eq(tag_lo_cpcs(cop0::tag_lo()), 2, "PState before invalidate (valid)")?;
            unsafe {
                cop0::cache::<{ cop0::ICACHE_INDEX_INVALIDATE }, 0>(addr);
                cop0::cache::<ICACHE_LOAD_TAG, 0>(addr);
                cop0::sync();
            }
            let t = cop0::tag_lo();
            soft_assert_eq(t, icache_tag_lo(false, phys), "TagLo after index invalidate + load tag (invalid, PFN unchanged)")?;
            Ok(())
        })
    }
}

pub struct IcacheHitInvalidateClearsValidWhenLineHits;

impl Test for IcacheHitInvalidateClearsValidWhenLineHits {
    fn name(&self) -> &str { "CACHE icache hit invalidate clears valid when line hits" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_icache_test(|| {
            let addr = 0x8000_3000usize;
            let phys = 0x0000_3000u32;
            unsafe {
                cop0::cache::<{ cop0::ICACHE_INDEX_INVALIDATE }, 0>(addr);
                cop0::set_tag_lo(icache_tag_lo(true, phys));
                cop0::cache::<ICACHE_STORE_TAG, 0>(addr);
                cop0::cache::<ICACHE_LOAD_TAG, 0>(addr);
                cop0::sync();
            }
            soft_assert_eq(tag_lo_cpcs(cop0::tag_lo()), 2, "line valid before hit invalidate")?;
            unsafe {
                cop0::cache::<{ cop0::ICACHE_HIT_INVALIDATE }, 0>(addr);
                cop0::cache::<ICACHE_LOAD_TAG, 0>(addr);
                cop0::sync();
            }
            soft_assert_eq(
                cop0::tag_lo(),
                icache_tag_lo(false, phys),
                "TagLo after hit invalidate + load tag (invalid, PFN unchanged)",
            )?;
            Ok(())
        })
    }
}
