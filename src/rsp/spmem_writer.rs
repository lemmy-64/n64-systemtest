use core::mem::size_of;

use crate::rsp::spmem::SPMEM;

pub struct SPMEMWriter {
    offset: usize,
}

impl SPMEMWriter {
    pub const fn new(start_offset: usize) -> Self {
        Self { offset: start_offset }
    }

    pub fn write(&mut self, value: u32) {
        SPMEM::write(self.offset, value);
        self.offset += size_of::<u32>();
    }
}