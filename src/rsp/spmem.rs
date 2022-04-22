use crate::math::vector::Vector;
use crate::MemoryMap;

pub struct SPMEM {}

impl SPMEM {
    pub fn write(addr: usize, value: u32) {
        let spmem = MemoryMap::uncached_spmem_address::<u32>(addr);
        unsafe {
            spmem.write_volatile(value);
        }
    }

    pub fn write_vector16_into_dmem(addr: usize, vec: &[u16; 8]) {
        assert!((addr & 3) == 0);
        for i in 0..4 {
            Self::write((addr + (i << 2)) & 0xFFC, ((vec[i << 1] as u32) << 16) | (vec[(i << 1) + 1] as u32));
        }
    }

    pub fn write_vector8_into_dmem<const COUNT: usize>(addr: usize, vec: &[u8; COUNT]) {
        assert!((addr & 3) == 0);
        assert!((COUNT & 3) == 0);
        for i in 0..COUNT >> 2 {
            Self::write((addr + (i << 2)) & 0xFFC, ((vec[i << 2] as u32) << 24) |
                ((vec[(i << 2) + 1] as u32) << 16) |
                ((vec[(i << 2) + 2] as u32) << 8) |
                (vec[(i << 2) + 3] as u32));
        }
    }

    // Over time we'll want to migrate to this one
    pub fn write_vector_into_dmem(addr: usize, vec: &Vector) {
        assert!((addr & 3) == 0);
        for i in 0..4 {
            Self::write((addr + (i << 2)) & 0xFFC, vec.get32(i));
        }
    }

    pub fn read(addr: usize) -> u32 {
        let spmem = MemoryMap::uncached_spmem_address::<u32>(addr);
        unsafe {
            spmem.read_volatile()
        }
    }

    pub fn read_vector16_from_dmem(addr: usize) -> [u16; 8] {
        assert!((addr & 3) == 0);
        let mut vec: [u16; 8] = Default::default();
        for i in 0..4 {
            let v = Self::read((addr + (i << 2)) & 0xFFC);
            vec[(i << 1)] = (v >> 16) as u16;
            vec[(i << 1) + 1] = v as u16;
        }
        vec
    }

    pub fn read_vector16_from_dmem_or_imem(addr: usize) -> [u16; 8] {
        assert!((addr & 3) == 0);
        let mut vec: [u16; 8] = Default::default();
        for i in 0..4 {
            let v = Self::read(addr + (i << 2));
            vec[(i << 1)] = (v >> 16) as u16;
            vec[(i << 1) + 1] = v as u16;
        }
        vec
    }

    pub fn read_vector8_from_dmem(addr: usize) -> [u8; 16] {
        assert!((addr & 3) == 0);
        let mut vec: [u8; 16] = Default::default();
        for i in 0..4 {
            let v = Self::read((addr + (i << 2)) & 0xFFC);
            vec[(i << 2)] = (v >> 24) as u8;
            vec[(i << 2) + 1] = (v >> 16) as u8;
            vec[(i << 2) + 2] = (v >> 8) as u8;
            vec[(i << 2) + 3] = v as u8;
        }
        vec
    }

    // Over time, we'll want to move more stuff to this Vector (instead of the arrays above)
    pub fn read_vector_from_dmem(addr: usize) -> Vector {
        assert!((addr & 3) == 0);
        Vector::new_with_u32_elements(
            Self::read(addr & 0xFFC),
            Self::read((addr + 4) & 0xFFC),
            Self::read((addr + 8) & 0xFFC),
            Self::read((addr + 12) & 0xFFC)
        )
    }
}
