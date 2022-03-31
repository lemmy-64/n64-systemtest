use core::mem::size_of;

use crate::rsp::spmem::SPMEM;

pub struct SPMEMWriter {
    offset: usize,
}

impl SPMEMWriter {
    pub const fn new(start_offset: usize) -> Self {
        Self { offset: start_offset }
    }

    pub fn write<T>(&mut self, value: T) {
        SPMEM::write(self.offset, value);
        self.offset += size_of::<T>();
    }
}