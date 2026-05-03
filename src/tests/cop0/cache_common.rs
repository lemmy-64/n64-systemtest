use alloc::string::String;

use crate::cop0;

pub fn cache_subsystem_reset() {
    cop0::icache_invalidate_all();
    cop0::dcache_invalidate_all();
    unsafe {
        cop0::set_tag_lo(0);
    }
}

pub fn run_cache_isolated_test(body: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
    let r = body();
    cache_subsystem_reset();
    r
}
