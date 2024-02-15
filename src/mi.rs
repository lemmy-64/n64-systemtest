use bitbybit::bitfield;

const MI_BASE_REG: *mut u32 = 0xA430_0000 as *mut u32;

#[allow(dead_code)]
enum RegisterOffset {
    Mode = 0x00,
    Version = 0x04,
    Interrupt = 0x08,
    InterruptMask = 0x0C,
}

#[bitfield(u32, default: 0)]
pub struct InterruptMaskWrite {
    #[bit(11, w)]
    setDP: bool,

    #[bit(10, w)]
    clearDP: bool,

    #[bit(9, w)]
    setPI: bool,

    #[bit(8, w)]
    clearPI: bool,

    #[bit(7, w)]
    setVI: bool,

    #[bit(6, w)]
    clearVI: bool,

    #[bit(5, w)]
    setAI: bool,

    #[bit(4, w)]
    clearAI: bool,

    #[bit(3, w)]
    setSI: bool,

    #[bit(2, w)]
    clearSI: bool,

    #[bit(1, w)]
    setSP: bool,

    #[bit(0, w)]
    clearSP: bool,
}

#[bitfield(u32, default: 0)]
pub struct Interrupt {
    #[bit(5, rw)]
    dp: bool,

    #[bit(4, rw)]
    pi: bool,

    #[bit(3, rw)]
    vi: bool,

    #[bit(2, rw)]
    ai: bool,

    #[bit(1, rw)]
    si: bool,

    #[bit(0, rw)]
    sp: bool,
}


fn read(reg: RegisterOffset) -> u32 {
    unsafe { MI_BASE_REG.add(reg as usize >> 2).read_volatile() }
}

fn write(reg: RegisterOffset, value: u32) {
    unsafe { MI_BASE_REG.add(reg as usize >> 2).write_volatile(value) }
}

fn interrupt() -> Interrupt {
    Interrupt::new_with_raw_value(read(RegisterOffset::Interrupt))
}

pub fn set_interrupt_mask(value: InterruptMaskWrite) {
    write(RegisterOffset::InterruptMask, value.raw_value());
}

pub fn clear_interrupt_mask() {
    set_interrupt_mask(InterruptMaskWrite::new()
        .with_clearDP(true)
        .with_clearPI(true)
        .with_clearVI(true)
        .with_clearAI(true)
        .with_clearSI(true)
        .with_clearSP(true)
    );
}

#[allow(dead_code)]
pub fn interrupt_mask() -> Interrupt {
    Interrupt::new_with_raw_value(read(RegisterOffset::InterruptMask))
}

pub fn is_sp_interrupt() -> bool { interrupt().sp() }

