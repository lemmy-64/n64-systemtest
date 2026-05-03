use crate::pi::Pi;

const WRITE_LEN: *mut u32 = 0xB3FF0014 as *mut u32;
const BUF: *mut u32 = 0xB3FF0020 as *mut u32;
const CHUNK: usize = 0x200;

#[inline(always)]
fn pi_wait() {
    while Pi::status().io_busy() {}
}

fn pack(dst: *mut u32, chunk: &[u8]) {
    let mut v = 0u32;
    let mut sh = 24;
    let mut i = 0usize;
    for &b in chunk {
        v |= (b as u32) << sh;
        if sh == 0 {
            pi_wait();
            unsafe { dst.add(i).write_volatile(v) };
            i += 1;
            sh = 24;
            v = 0;
        } else {
            sh -= 8;
        }
    }
    if sh < 24 {
        pi_wait();
        unsafe { dst.add(i).write_volatile(v) };
    }
}

pub(crate) fn detect() -> bool {
    const M: u32 = 0x12345678;
    unsafe {
        pi_wait();
        BUF.write_volatile(M);
        pi_wait();
        BUF.read_volatile() == M
    }
}

pub(crate) fn text_out(s: &str) {
    for chunk in s.as_bytes().chunks(CHUNK) {
        pack(BUF, chunk);
        pi_wait();
        unsafe { WRITE_LEN.write_volatile(chunk.len() as u32) };
    }
}
