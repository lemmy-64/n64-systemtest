// Memory map:
// 0x8000_0000 to (a little more): bss
// until 2mb: heap (including the framebuffer)
// 2mb-3mb: available for tests
// growing down from the end: stack

static mut MEMORY_SIZE: usize = 0;

pub struct MemoryMap {

}

impl MemoryMap {
    pub const HEAP_END: usize = 2 * 1024 * 1024;
    pub const HEAP_END_VIRTUAL_CACHED: usize = 0x8000_0000 | MemoryMap::HEAP_END;
    pub const HEAP_END_VIRTUAL_UNCACHED: usize = 0xA000_0000 | MemoryMap::HEAP_END;

    pub const PHYSICAL_SPMEM_BASE: usize = 0x0400_0000;

    /// Call very early (before setting up exception handlers) during boot to set memory size
    pub(super) fn init() {
        unsafe {
            assert_eq!(MEMORY_SIZE, 0);
            let value = *(0x8000_0318 as *mut usize);
            MEMORY_SIZE = value;
        };
    }

    /// Returns the total memory size of this device (either 4MB or 8MB)
    pub fn memory_size() -> usize {
        // MEMORY_SIZE is only set during early boot and then never again, so this should be safe
        unsafe { MEMORY_SIZE }
    }

    /// Returns an uncached pointer of the given pointer (e.g. 0xA000_1234 is returned for 0x8000_1234
    pub fn uncached<T>(p: *const T) -> *const T {
        let memory_address = p as usize;
        assert_eq!(memory_address & 0xE000_0000, 0x8000_0000);
        ((memory_address & 0x1FFF_FFFF) | 0xA000_0000) as *const T
    }

    /// Returns the cartridge (rom) address of a given constant
    pub fn uncached_cart_address<T>(p: *const T) -> *const T {
        // The bootcode copies from 0x10001000 to 0x8000_0400. If we have some other pointer,
        // it doesn't come from the cart
        let memory_address = p as usize;
        assert!(memory_address >= 0x8000_0400);
        assert!(memory_address < 0x8000_0400 + 1 * 1024 * 1024);

        Self::uncached((memory_address + 0x10001000 - 0x400) as *const T)
    }

    pub fn physical_to_uncached_mut<T>(address: usize) -> *mut T {
        (address | 0xA000_0000) as *mut T
    }

    pub fn uncached_spmem_address<T>(sp_address: usize) -> *mut T {
        Self::physical_to_uncached_mut::<T>(Self::PHYSICAL_SPMEM_BASE + sp_address)
    }
}
