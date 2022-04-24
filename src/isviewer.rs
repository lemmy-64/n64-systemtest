use crate::pi;

/// Write the length of the text here
const ISVIEWER_WRITE_LEN: *mut u32 = 0xB3FF0014 as *mut u32;

// Write text data into this buffer
const ISVIEWER_BUFFER_START: *mut u32 = 0xB3FF0020 as *mut u32;
const ISVIEWER_BUFFER_LENGTH: usize = 0x200;

fn pi_wait() {
    while pi::is_io_busy() {}
}


// This method simply prints text without synchronization. This should only be used from within
// the exception handler which can't wait for a lock
pub fn text_out(s: &str) {
    for chunk in s.as_bytes().chunks(ISVIEWER_BUFFER_LENGTH) {
        // Write
        let mut value = 0u32;
        let mut shift = 24u32;
        let mut i = 0;
        for byte in chunk {
            value |= (*byte as u32) << shift;
            if shift == 0 {
                pi_wait();
                unsafe { ISVIEWER_BUFFER_START.add(i).write_volatile(value) };
                i += 1;
                shift = 24;
                value = 0;
            } else {
                shift -= 8;
            }
        }
        if shift < 24 {
            pi_wait();
            unsafe { ISVIEWER_BUFFER_START.add(i).write_volatile(value) };
        }
        pi_wait();
        unsafe {
            ISVIEWER_WRITE_LEN.write_volatile(chunk.len() as u32);
        }
    }
}
