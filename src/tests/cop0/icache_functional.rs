use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::cop0;
use crate::cop0::{ICACHE_FILL, ICACHE_HIT_INVALIDATE, ICACHE_HIT_WRITEBACK, ICACHE_INDEX_INVALIDATE};
use crate::MemoryMap;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

use super::cache_common;

const JR_RA: u32 = 0x03E0_0008;

#[inline]
fn ori_v0_imm(imm: u16) -> u32 {
    (0x0Du32 << 26) | (2u32 << 16) | u32::from(imm)
}

#[repr(C, align(32))]
struct IcacheExecSlot {
    insns: [u32; 8],
}

static mut ICACHE_EXEC_SLOT: IcacheExecSlot = IcacheExecSlot { insns: [0; 8] };

fn icache_exec_stub_ptrs() -> (usize, *mut u32) {
    unsafe {
        let p = ICACHE_EXEC_SLOT.insns.as_mut_ptr();
        (p as usize, MemoryMap::uncached_mut(p))
    }
}

fn write_exec_stub_uncached(uncached: *mut u32, imm: u16) {
    unsafe {
        uncached.write_volatile(ori_v0_imm(imm));
        uncached.add(1).write_volatile(JR_RA);
        uncached.add(2).write_volatile(0);
    }
}

fn call_stub_v0(stub_cached: usize) -> u32 {
    let v0: u32;
    unsafe {
        asm!(
            ".set noat",
            ".set noreorder",
            "daddiu $25, $31, 0",
            "jalr {st}",
            "nop",
            "daddiu $31, $25, 0",
            st = in(reg) stub_cached,
            out("$2") v0,
            out("$25") _,
        );
    }
    v0
}

pub struct IcacheFetchUsesMemoryImage;

impl Test for IcacheFetchUsesMemoryImage {
    fn name(&self) -> &str { "ICACHE: fetch matches instruction words in RAM" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, uncached) = icache_exec_stub_ptrs();
            unsafe {
                cop0::cache::<ICACHE_INDEX_INVALIDATE, 0>(cached);
                write_exec_stub_uncached(uncached, 0xCAFE);
            }
            let got = call_stub_v0(cached);
            soft_assert_eq(
                got,
                0xCAFEu32,
                "JALR into line should execute ORI immediate from filled icache line",
            )?;
            Ok(())
        })
    }
}

pub struct IcacheUncachedPatchStallsUntilInvalidate;

impl Test for IcacheUncachedPatchStallsUntilInvalidate {
    fn name(&self) -> &str { "ICACHE: uncached patch does not affect fetched line until invalidate" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, uncached) = icache_exec_stub_ptrs();
            unsafe {
                cop0::cache::<ICACHE_INDEX_INVALIDATE, 0>(cached);
                write_exec_stub_uncached(uncached, 0x1111);
            }
            soft_assert_eq(call_stub_v0(cached), 0x1111u32, "first run")?;
            write_exec_stub_uncached(uncached, 0x2222);
            soft_assert_eq(
                call_stub_v0(cached),
                0x1111u32,
                "stale icache after uncached patch",
            )?;
            unsafe {
                cop0::cache::<ICACHE_HIT_INVALIDATE, 0>(cached);
            }
            soft_assert_eq(call_stub_v0(cached), 0x2222u32, "after hit invalidate")?;
            Ok(())
        })
    }
}

pub struct IcacheFillOpcodeLoadsLineFromRam;

impl Test for IcacheFillOpcodeLoadsLineFromRam {
    fn name(&self) -> &str { "CACHE icache fill (0x14) loads line from memory" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, uncached) = icache_exec_stub_ptrs();
            unsafe {
                cop0::cache::<ICACHE_INDEX_INVALIDATE, 0>(cached);
                write_exec_stub_uncached(uncached, 0x3333);
                cop0::cache::<ICACHE_FILL, 0>(cached);
            }
            soft_assert_eq(call_stub_v0(cached), 0x3333u32, "after explicit CACHE fill")?;
            Ok(())
        })
    }
}

pub struct IcacheHitWritebackPushesLineToRam;

impl Test for IcacheHitWritebackPushesLineToRam {
    fn name(&self) -> &str { "CACHE icache hit writeback (0x18) pushes line to RAM" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        cache_common::run_cache_isolated_test(|| {
            let (cached, uncached) = icache_exec_stub_ptrs();
            unsafe {
                cop0::cache::<ICACHE_INDEX_INVALIDATE, 0>(cached);
                write_exec_stub_uncached(uncached, 0x4444);
            }
            soft_assert_eq(call_stub_v0(cached), 0x4444u32, "icache line hot")?;
            write_exec_stub_uncached(uncached, 0x5555);
            soft_assert_eq(unsafe { uncached.read_volatile() }, ori_v0_imm(0x5555), "RAM patched")?;
            unsafe {
                cop0::cache::<ICACHE_HIT_WRITEBACK, 0>(cached);
            }
            soft_assert_eq(
                unsafe { uncached.read_volatile() },
                ori_v0_imm(0x4444),
                "RAM after icache hit writeback",
            )?;
            soft_assert_eq(call_stub_v0(cached), 0x4444u32, "icache still holds original")?;
            Ok(())
        })
    }
}
