/// Write the length of the text here
const ISVIEWER_WRITE_LEN: *mut u32 = 0xB3FF0014 as *mut u32;

// Write text data into this buffer
const ISVIEWER_BUFFER_START: *mut u8 = 0xB3FF0020 as *mut u8;
const ISVIEWER_BUFFER_LENGTH: usize = 0x200;

// This method simply prints text without synchronization. This should only be used from within
// the exception handler which can't wait for a lock
pub fn text_out(s: &str) {
    for chunk in s.as_bytes().chunks(ISVIEWER_BUFFER_LENGTH) {
        for i in 0..chunk.len() {
            let b = chunk[i];
            unsafe { ISVIEWER_BUFFER_START.add(i).write_volatile(b) };
        }
        unsafe { ISVIEWER_WRITE_LEN.write_volatile(chunk.len() as u32); }
    }
}
