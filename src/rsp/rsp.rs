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
const SP_STATUS_DMA_BUSY: u32 = 0b100;

const SP_STATUS_SET_CLEAR_HALT: u32 = 0b1;
const SP_STATUS_SET_CLEAR_BROKE: u32 = 0b100;
const SP_STATUS_SET_CLEAR_INTERRUPT: u32 = 0b1000;
const SP_STATUS_SET_CLEAR_INTERRUPT_ON_BREAK: u32 = 0b1_0000_000;

pub struct RSP {
}

impl RSP {
    fn set_register(reg: RegisterOffset, value: u32) {
        unsafe { SP_BASE_REG.add(reg as usize >> 2).write_volatile(value) }
    }

    fn get_register(reg: RegisterOffset) -> u32 {
        unsafe { SP_BASE_REG.add(reg as usize >> 2).read_volatile() }
    }

    fn set_sp_address(value: u32) {
        Self::set_register(RegisterOffset::SPAddress, value);
    }

    fn set_dram_address(value: u32) {
        Self::set_register(RegisterOffset::DRAMAddress, value);
    }

    fn set_write_length(value: u32) {
        Self::set_register(RegisterOffset::WriteLength, value);
    }

    pub fn status() -> u32 {
        Self::get_register(RegisterOffset::Status)
    }

    fn set_status(value: u32) {
        Self::set_register(RegisterOffset::Status, value);
    }

    pub fn set_pc(value: u32) {
        unsafe { SP_PC_REG.write_volatile(value) }
    }

    pub fn pc() -> u32 {
        unsafe { SP_PC_REG.read_volatile() }
    }

    pub fn start_running(pc: u32) {
        Self::set_pc(pc);

        // Clear status and clear interrupt just in case
        Self::set_status(SP_STATUS_SET_CLEAR_HALT |
            SP_STATUS_SET_CLEAR_INTERRUPT |
            SP_STATUS_SET_CLEAR_INTERRUPT_ON_BREAK);
    }

    #[allow(dead_code)]
    pub unsafe fn start_dma_sp_to_cpu(spmem: u32, to: *mut u8, length: u32) {
        Self::set_sp_address(spmem);
        Self::set_dram_address(to as usize as u32);
        Self::set_write_length(length);
    }

    pub fn wait_until_rsp_is_halted() {
        while (Self::status() & SP_STATUS_HALT) == 0 {

        }
    }

    pub fn wait_until_rsp_is_halted_and_dma_completed() {
        while (Self::status() & (SP_STATUS_DMA_BUSY | SP_STATUS_HALT)) != SP_STATUS_HALT {

        }
    }

    pub fn run_and_wait(pc: u32) {
        Self::start_running(pc);
        Self::wait_until_rsp_is_halted();
    }

    pub fn clear_broke() {
        Self::set_status(SP_STATUS_SET_CLEAR_BROKE);
    }
}
