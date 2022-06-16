use core::mem::size_of;

use crate::rsp::spmem::SPMEM;

pub struct DMEMWriter {
    offset: usize,
}

impl DMEMWriter {
    pub const fn new(start_offset: usize) -> Self {
        Self { offset: start_offset & 0xFFC }
    }

    pub fn write(&mut self, value: u32) {
        SPMEM::write(self.offset | 0x1000, value);
        self.offset = (self.offset + size_of::<u32>()) & 0xFFC;
    }

    pub fn offset(&self) -> usize { return self.offset; }
}