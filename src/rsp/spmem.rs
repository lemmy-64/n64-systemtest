use crate::MemoryMap;

pub struct SPMEM {}

impl SPMEM {
    pub fn write<T>(addr: usize, value: T) {
        if addr < 0x2000 {
            let spmem = MemoryMap::uncached_spmem_address::<T>(addr);
            unsafe {
                spmem.write_volatile(value);
            }
        }
    }

    pub fn read<T: Default>(addr: usize) -> T {
        if addr < 0x2000 {
            let spmem = MemoryMap::uncached_spmem_address::<T>(addr);
            unsafe {
                spmem.read_volatile()
            }
        } else {
            T::default()
        }
    }
}
