use crate::rsp::spmem_writer::SPMEMWriter;

pub struct RSPAssembler {
    writer: SPMEMWriter,
}

impl RSPAssembler {
    pub const fn new(start_offset: usize) -> Self {
        // IMEM starts at 0x1000
        Self { writer: SPMEMWriter::new(start_offset | 0x1000) }
    }

    fn write_special(&mut self, function: u32) {
        assert!(function < 0b111111);
        self.writer.write::<u32>(function);
    }

    pub fn write_nop(&mut self) {
        self.write_special(0);
    }

    pub fn write_break(&mut self) {
        self.write_special(13);
    }
}