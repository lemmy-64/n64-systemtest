use crate::MemoryMap;

pub struct SPMEM {}

impl SPMEM {
    pub fn write(addr: usize, value: u32) {
        let spmem = MemoryMap::uncached_spmem_address::<u32>(addr);
        unsafe {
            spmem.write_volatile(value);
        }
    }

    pub fn write_vector_16(addr: usize, vec: &[u16; 8]) {
        for i in 0..4 {
            Self::write(addr + (i << 2), ((vec[i << 1] as u32) << 16) | (vec[(i << 1) + 1] as u32));
        }
    }

    pub fn read(addr: usize) -> u32 {
        let spmem = MemoryMap::uncached_spmem_address::<u32>(addr);
        unsafe {
            spmem.read_volatile()
        }
    }

    pub fn read_vector_16(addr: usize) -> [u16; 8] {
        let mut vec: [u16; 8] = Default::default();
        for i in 0..4 {
            let v = Self::read(addr + (i << 2));
            vec[(i << 1)] = (v >> 16) as u16;
            vec[(i << 1) + 1] = v as u16;
        }
        vec
    }

}
