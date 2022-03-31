const SP_BASE_REG: *mut u32 = 0xA404_0000 as *mut u32;
const SP_PC_REG: *mut u32 = 0xA408_0000 as *mut u32;

#[allow(dead_code)]
enum RegisterOffset {
    SPAddress = 0x00,
    DRAMAddress = 0x04,
    ReadLength = 0x08,
    WriteLength = 0x0C,
    Status = 0x10,
    DMAFull = 0x14,
    DMABusy = 0x18,
    Semaphore = 0x1C,
}

const SP_STATUS_HALT: u32 = 0b1;

const SP_STATUS_SET_CLEAR_HALT: u32 = 0b1;
const SP_STATUS_SET_CLEAR_BROKE: u32 = 0b100;
const SP_STATUS_SET_CLEAR_INTERRUPT: u32 = 0b1000;
const SP_STATUS_SET_CLEAR_INTERRUPT_ON_BREAK: u32 = 0b1_0000_000;

pub struct RSP {
}

impl RSP {
    pub fn status() -> u32 {
        unsafe { SP_BASE_REG.add(RegisterOffset::Status as usize >> 2).read_volatile() }
    }

    fn set_status(value: u32) {
        unsafe { SP_BASE_REG.add(RegisterOffset::Status as usize >> 2).write_volatile(value) }
    }

    fn set_pc(value: u32) {
        unsafe { SP_PC_REG.write_volatile(value) }
    }

    pub fn pc() -> u32 {
        unsafe { SP_PC_REG.read_volatile() }
    }

    pub fn run(start_offset: u32) {
        Self::set_pc(start_offset);

        // Clear status and clear interrupt just in case
        Self::set_status(SP_STATUS_SET_CLEAR_HALT |
            SP_STATUS_SET_CLEAR_INTERRUPT |
            SP_STATUS_SET_CLEAR_INTERRUPT_ON_BREAK);
    }

    pub fn wait_until_done() {
        while (Self::status() & SP_STATUS_HALT) == 0 {

        }
    }

    pub fn run_and_wait(start_offset: u32) {
        Self::run(start_offset);
        Self::wait_until_done();
    }

    pub fn clear_broke() {
        Self::set_status(SP_STATUS_SET_CLEAR_BROKE);
    }
}
