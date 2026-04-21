use alloc::string::String;

use crate::cop0;

pub const ICACHE_INDEX_INVALIDATE: u8 = 0;
pub const ICACHE_HIT_INVALIDATE: u8 = 16;
pub const ICACHE_FILL: u8 = 20;
pub const ICACHE_HIT_WRITEBACK: u8 = 24;
pub const DCACHE_INDEX_WRITEBACK_INVALIDATE: u8 = 1;
pub const DCACHE_INDEX_LOAD_TAG: u8 = 5;
pub const DCACHE_INDEX_STORE_TAG: u8 = 9;
pub const DCACHE_CREATE_DIRTY_EXCLUSIVE: u8 = 13;
pub const DCACHE_HIT_INVALIDATE: u8 = 17;
pub const DCACHE_HIT_WRITEBACK_INVALIDATE: u8 = 21;
pub const DCACHE_HIT_WRITEBACK: u8 = 25;

const ICACHE_BYTES: usize = 16 * 1024;
const ICACHE_LINE_BYTES: usize = 32;
const DCACHE_BYTES: usize = 8 * 1024;
const DCACHE_LINE_BYTES: usize = 16;

pub fn icache_invalidate_all() {
    let base = 0x8000_0000usize;
    for i in (0..ICACHE_BYTES).step_by(ICACHE_LINE_BYTES) {
        unsafe {
            cop0::cache::<ICACHE_INDEX_INVALIDATE, 0>(base + i);
        }
    }
}

pub fn dcache_invalidate_all() {
    let base = 0x8000_0000usize;
    for i in (0..DCACHE_BYTES).step_by(DCACHE_LINE_BYTES) {
        unsafe {
            cop0::cache::<DCACHE_INDEX_WRITEBACK_INVALIDATE, 0>(base + i);
        }
    }
}

pub fn cache_subsystem_reset() {
    icache_invalidate_all();
    dcache_invalidate_all();
    unsafe {
        cop0::set_tag_lo(0);
    }
}

pub fn run_cache_isolated_test(body: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
    let r = body();
    cache_subsystem_reset();
    r
}
