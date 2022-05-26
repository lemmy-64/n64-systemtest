use core::fmt::{Debug, Formatter};

use crate::rsp::rsp_assembler::Element;

/// Vector allows flexible access via u8,u16,u32,u64, while exposing a big-endian mapping of those
/// data types. It currently assumes a big endian host system, as it will only ever run on the N64
#[derive(Copy, Clone)]
pub union Vector {
    as_u8: [u8; 16],
    as_u16: [u16; 8],
    as_u32: [u32; 4],
    as_u64: [u64; 2],
}

impl Vector {
    pub const fn new() -> Self {
        Self {
            as_u64: [0, 0],
        }
    }

    pub const fn new_with_broadcast_16(value: u16) -> Self {
        Self {
            as_u16: [value; 8],
        }
    }

    pub const fn new_with_u32_elements(data0: u32, data1: u32, data2: u32, data3: u32) -> Self {
        Self {
            as_u32: [data0, data1, data2, data3],
        }
    }

    pub const fn from_u16(data: [u16; 8]) -> Self {
        Self {
            as_u16: data,
        }
    }

    pub const fn from_u8(data: [u8; 16]) -> Self {
        Self {
            as_u8: data,
        }
    }

    pub fn copy_with_broadcast_16(&self, index: usize) -> Vector {
        let v16 = self.get16(index) as u32;
        let v32 = (v16 << 16) | v16;
        Self::new_with_u32_elements(v32, v32, v32, v32)
    }

    pub fn copy_with_element_specifier_applied(&self, e: Element) -> Vector {
        if e == Element::All || e == Element::All1 {
            *self
        } else {
            let mut v = [0u16; 8];
            for i in 0..8 {
                v[i] = self.get16(e.get_effective_element_index(i));
            }
            Vector::from_u16(v)
        }
    }

    pub fn get32(&self, index: usize) -> u32 { unsafe { self.as_u32[index] } }

    pub fn get16(&self, index: usize) -> u16 { unsafe { self.as_u16[index] } }
    pub fn set16(&mut self, index: usize, value: u16) { unsafe { self.as_u16[index] = value } }

    pub fn get8(&self, index: usize) -> u8 { unsafe { self.as_u8[index] } }
    pub fn set8(&mut self, index: usize, value: u8) { unsafe { self.as_u8[index] = value } }
}

impl Debug for Vector {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries(unsafe { &self.as_u16 })
            .finish()
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (self.as_u64[0] == other.as_u64[0]) && (self.as_u64[1] == other.as_u64[1]) }
    }
}

impl Eq for Vector {}

impl Default for Vector {
    fn default() -> Self { Self::new() }
}

