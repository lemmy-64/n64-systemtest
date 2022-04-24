const PI_BASE_REG: *mut u32 = 0xA460_0000 as *mut u32;

#[allow(dead_code)]
enum RegisterOffset {
    Status = 0x10,
}

fn read(reg: RegisterOffset) -> u32 {
    unsafe { PI_BASE_REG.add(reg as usize >> 2).read_volatile() }
}

fn status() -> u32 {
    read(RegisterOffset::Status)
}

pub fn is_io_busy() -> bool { (status() & 2) != 0 }
