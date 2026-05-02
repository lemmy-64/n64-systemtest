use core::arch::asm;
use core::sync::atomic::{AtomicU8, Ordering};

const COP0_OPCODE: u32 = 0x10;
const COP0_RS_CO: u32 = 0x10;
const XDETECT_FUNCT: u32 = 0x20;
const XLOG_FUNCT: u32 = 0x25;
const XIOCTL_FUNCT: u32 = 0x2c;

const XDETECT_CODE_EXTENSIONS_20_3F: u16 = 0x1;
const XLOG_CODE_LENGTH_IN_RT: u16 = 0x1;

const XIOCTL_EXIT: u16 = 0x1;
const XIOCTL_FAST: u16 = 0x2;

const CAPS_UNKNOWN: u8 = 0;
const CAPS_UNSUPPORTED: u8 = 1;
const CAPS_SUPPORTED: u8 = 2;

static EMUX_LOG_CAPS: AtomicU8 = AtomicU8::new(CAPS_UNKNOWN);

const fn encode_xdetect(rd: u8, rt: u8, code: u16) -> u32 {
    (COP0_OPCODE << 26)
        | (COP0_RS_CO << 21)
        | (((rd as u32) & 0x1f) << 20)
        | (((rt as u32) & 0x1f) << 15)
        | (((code as u32) & 0x1ff) << 6)
        | XDETECT_FUNCT
}

const fn encode_xlog(rd: u8, rt: u8, code: u16) -> u32 {
    (COP0_OPCODE << 26)
        | (COP0_RS_CO << 21)
        | (((rd as u32) & 0x1f) << 20)
        | (((rt as u32) & 0x1f) << 15)
        | (((code as u32) & 0x1ff) << 6)
        | XLOG_FUNCT
}

const fn encode_xioctl(code: u16) -> u32 {
    (COP0_OPCODE << 26) | (COP0_RS_CO << 21) | (((code as u32) & 0x1ff) << 6) | XIOCTL_FUNCT
}

#[inline(always)]
fn xdetect_extensions_20_3f() -> u32 {
    let result: u32;
    unsafe {
        asm!(
            ".set noat",
            ".set noreorder",
            ".word {instruction}",
            instruction = const encode_xdetect(8, 0, XDETECT_CODE_EXTENSIONS_20_3F),
            lateout("$8") result,
            options(nostack, preserves_flags)
        );
    }
    result
}

#[inline(always)]
fn xlog(s: &str) {
    let ptr = s.as_ptr();
    let len = s.len();
    if len == 0 {
        return;
    }

    unsafe {
        asm!(
            ".set noat",
            ".set noreorder",
            "move $8, {ptr}",
            "move $9, {len}",
            ".word {instruction}",
            ptr = in(reg) ptr,
            len = in(reg) len,
            instruction = const encode_xlog(8, 9, XLOG_CODE_LENGTH_IN_RT),
            out("$8") _,
            out("$9") _,
            options(nostack, preserves_flags)
        );
    }
}

fn emux_log_supported() -> bool {
    match EMUX_LOG_CAPS.load(Ordering::Relaxed) {
        CAPS_SUPPORTED => true,
        CAPS_UNSUPPORTED => false,
        _ => {
            let extensions_mask = xdetect_extensions_20_3f();
            let supported = (extensions_mask & (1 << (XLOG_FUNCT - 0x20))) != 0;
            EMUX_LOG_CAPS.store(
                if supported { CAPS_SUPPORTED } else { CAPS_UNSUPPORTED },
                Ordering::Relaxed
            );
            supported
        }
    }
}

pub fn text_out(s: &str) -> bool {
    if !emux_log_supported() {
        return false;
    }
    xlog(s);
    true
}

#[inline(always)]
fn xioctl(code: u16) {
    unsafe {
        match code {
            XIOCTL_EXIT => asm!(
                ".set noat",
                ".set noreorder",
                ".word {instruction}",
                instruction = const encode_xioctl(XIOCTL_EXIT),
                options(nostack, preserves_flags)
            ),
            XIOCTL_FAST => asm!(
                ".set noat",
                ".set noreorder",
                ".word {instruction}",
                instruction = const encode_xioctl(XIOCTL_FAST),
                options(nostack, preserves_flags)
            ),
            _ => {}
        }
    }
}

pub fn xioctl_fast() {
    xioctl(XIOCTL_FAST);
}

pub fn xioctl_exit() {
    xioctl(XIOCTL_EXIT);
}
