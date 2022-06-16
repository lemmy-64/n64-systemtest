const MI_BASE_REG: *mut u32 = 0xA430_0000 as *mut u32;

const INTR_SP: u32 = 1;

#[allow(dead_code)]
enum RegisterOffset {
    Mode = 0x00,
    Version = 0x04,
    Interrupt = 0x08,
    InterruptMask = 0x0C,
}

fn read(reg: RegisterOffset) -> u32 {
    unsafe { MI_BASE_REG.add(reg as usize >> 2).read_volatile() }
}

fn interrupt() -> u32 {
    read(RegisterOffset::Interrupt)
}

pub fn is_sp_interrupt() -> bool { (interrupt() & INTR_SP) != 0 }

