#![allow(dead_code)]

use bitbybit::bitfield;

const PI_BASE_REG: usize = 0xA460_0000u32 as i32 as usize;
const PI_DRAM_ADDR: *mut u32 = (PI_BASE_REG + 0x0) as *mut u32;
const PI_CART_ADDR: *mut u32 = (PI_BASE_REG + 0x4) as *mut u32;
const PI_RD_LEN: *mut u32 = (PI_BASE_REG + 0x8) as *mut u32;
const PI_WR_LEN: *mut u32 = (PI_BASE_REG + 0xC) as *mut u32;
const PI_STATUS: *mut u32 = (PI_BASE_REG + 0x10) as *mut u32;

#[bitfield(u32, default: 0)]
#[derive(Eq, PartialEq, Debug)]
pub struct PiStatusRead {
    #[bit(3, rw)]
    pub interrupt: bool,

    #[bit(2, rw)]
    pub error: bool,

    #[bit(1, rw)]
    pub io_busy: bool,

    #[bit(0, rw)]
    pub dma_busy: bool,
}

#[bitfield(u32, default: 0)]
pub struct PiStatusWrite {
    #[bit(1, w)]
    pub clear_interrupt: bool,

    #[bit(0, w)]
    pub reset: bool,
}

pub struct Pi {}

impl Pi {
    pub fn set_dram_address(value: u32) {
        unsafe { PI_DRAM_ADDR.write_volatile(value) }
    }

    pub fn dram_address() -> u32 {
        unsafe { PI_DRAM_ADDR.read_volatile() }
    }

    pub fn set_cart_address(value: u32) {
        unsafe { PI_CART_ADDR.write_volatile(value) }
    }

    pub fn cart_address() -> u32 {
        unsafe { PI_CART_ADDR.read_volatile() }
    }

    pub fn set_read_length(value: u32) {
        unsafe { PI_RD_LEN.write_volatile(value) }
    }

    pub fn read_length() -> u32 {
        unsafe { PI_RD_LEN.read_volatile() }
    }

    pub fn set_write_length(value: u32) {
        unsafe { PI_WR_LEN.write_volatile(value) }
    }

    pub fn write_length() -> u32 {
        unsafe { PI_WR_LEN.read_volatile() }
    }

    pub fn set_status(value: PiStatusWrite) {
        unsafe { PI_STATUS.write_volatile(value.raw_value()) }
    }

    pub fn status() -> PiStatusRead {
        PiStatusRead::new_with_raw_value(unsafe { PI_STATUS.read_volatile() })
    }
}