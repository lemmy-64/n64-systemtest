use core::sync::atomic::{AtomicU8, Ordering};

const UNK: u8 = 0;
const NONE: u8 = 1;
const EMUX: u8 = 2;
const ISV: u8 = 3;
const SC64: u8 = 4;

static BACK: AtomicU8 = AtomicU8::new(UNK);

fn backend() -> u8 {
    match BACK.load(Ordering::Relaxed) {
        UNK => {}
        b => return b,
    }
    let b = if crate::emux::emux_log_supported() {
        EMUX
    } else if crate::isviewer::detect() {
        ISV
    } else if crate::sc64::detect() {
        SC64
    } else {
        NONE
    };
    BACK.store(b, Ordering::Relaxed);
    b
}

pub fn text_out(s: &str) {
    match backend() {
        EMUX => {
            let _ = crate::emux::text_out(s);
        }
        ISV => crate::isviewer::text_out(s),
        SC64 => crate::sc64::text_out(s),
        _ => {}
    }
}
