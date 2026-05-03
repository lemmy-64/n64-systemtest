use crate::pi::Pi;
use crate::cop0;
use crate::memory_map::MemoryMap;

const SC64_CART: u32 = 0x1000_0000;
const DEBUG_TOP: u32 = 0x0400_0000;
const USB_IO_RESERVE: u32 = 8 * 1024 * 1024;
const DEBUG_OFF: u32 = DEBUG_TOP - USB_IO_RESERVE;
const DBG_PHYS: u32 = SC64_CART + DEBUG_OFF;

const REG_CMD: *mut u32 = 0xBFFF_0000 as *mut u32;
const REG_D0: *mut u32 = 0xBFFF_0004 as *mut u32;
const REG_D1: *mut u32 = 0xBFFF_0008 as *mut u32;
const REG_ID: *mut u32 = 0xBFFF_000C as *mut u32;
const REG_KEY: *mut u32 = 0xBFFF_0010 as *mut u32;

const SR_CMD_BUSY: u32 = 1 << 31;
const SR_CMD_ERR: u32 = 1 << 30;
const USB_WRITE_STATUS_BUSY: u32 = 1 << 31;
const CFG_ROM_WRITE_ENABLE: u32 = 1;
const DATATYPE_TEXT: u32 = 1;
const BLOCK: usize = 512;
const POLL_MAX: u32 = 500_000;
const DMA_ALIGN: usize = 2;

#[repr(align(16))]
struct Aligned {
    bytes: [u8; BLOCK],
}

static mut DMA_BUF: Aligned = Aligned { bytes: [0; BLOCK] };

#[inline(always)]
fn pi_wait() {
    while Pi::status().io_busy() {}
}

fn exec_cmd(cmd: u32, d0: Option<u32>, d1: Option<u32>) -> Result<[u32; 2], ()> {
    unsafe {
        match (d0, d1) {
            (Some(a0), Some(a1)) => {
                pi_wait();
                REG_D0.write_volatile(a0);
                pi_wait();
                REG_D1.write_volatile(a1);
            }
            _ => {}
        }
        pi_wait();
        REG_CMD.write_volatile(cmd);
        let sr = loop {
            pi_wait();
            let s = REG_CMD.read_volatile();
            if s & SR_CMD_BUSY == 0 {
                break s;
            }
        };
        if sr & SR_CMD_ERR != 0 {
            return Err(());
        }
        pi_wait();
        let r0 = REG_D0.read_volatile();
        pi_wait();
        let r1 = REG_D1.read_volatile();
        Ok([r0, r1])
    }
}

fn set_writable(enable: u32) -> Result<u32, ()> {
    let r = exec_cmd(b'C' as u32, Some(CFG_ROM_WRITE_ENABLE), Some(enable))?;
    Ok(r[1])
}

fn copy_to_dbg(off: usize, chunk: &[u8]) {
    let n = chunk.len();
    let d = (n + DMA_ALIGN - 1) & !(DMA_ALIGN - 1);
    unsafe {
        let dst = DMA_BUF.bytes.as_mut_ptr();
        for i in 0..n {
            dst.add(i).write_volatile(chunk[i]);
        }
        for i in n..d {
            dst.add(i).write_volatile(0);
        }
        cop0::dcache_hit_writeback_invalidate_range(dst as usize, d);
        let dram = MemoryMap::uncached_to_physical_mut(MemoryMap::uncached_mut(dst)) as u32;
        Pi::set_dram_address(dram);
        Pi::set_cart_address(DBG_PHYS + off as u32);
        Pi::set_read_length(d as u32 - 1);
        while Pi::status().dma_busy() {}
    }
}

pub(crate) fn detect() -> bool {
    unsafe {
        pi_wait();
        REG_KEY.write_volatile(0);
        pi_wait();
        REG_KEY.write_volatile(0x5F554E4C);
        pi_wait();
        REG_KEY.write_volatile(0x4F434B5F);
        pi_wait();
        REG_ID.read_volatile() == 0x53437632
    }
}

pub(crate) fn text_out(s: &str) {
    let b = s.as_bytes();
    if b.is_empty() {
        return;
    }
    let st = match exec_cmd(b'U' as u32, None, None) {
        Ok(r) => r,
        Err(_) => return,
    };
    if st[0] & USB_WRITE_STATUS_BUSY != 0 {
        return;
    }
    let restore = match set_writable(1) {
        Ok(v) => v,
        Err(_) => return,
    };
    let mut off = 0usize;
    while off < b.len() {
        let n = core::cmp::min(BLOCK, b.len() - off);
        copy_to_dbg(off, &b[off..off + n]);
        off += n;
    }
    let _ = set_writable(restore);
    let header = (DATATYPE_TEXT << 24) | (b.len() as u32 & 0xFFFFFF);
    if exec_cmd(b'M' as u32, Some(DBG_PHYS), Some(header)).is_err() {
        return;
    }
    let mut c = 0u32;
    loop {
        if c >= POLL_MAX {
            break;
        }
        c += 1;
        let r = exec_cmd(b'U' as u32, None, None);
        match r {
            Ok(v) if v[0] & USB_WRITE_STATUS_BUSY == 0 => break,
            Ok(_) | Err(_) => {}
        }
    }
}
