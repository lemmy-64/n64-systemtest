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

pub const SP_STATUS_HALT: u32 = 0b1;
pub const SP_STATUS_DMA_BUSY: u32 = 0b100;
pub const SP_STATUS_INTERRUPT_ON_BREAK: u32 = 0b1000000;

pub const SP_STATUS_SET_CLEAR_HALT: u32 = 0b1;
pub const SP_STATUS_SET_SET_HALT: u32 = 0b10;
pub const SP_STATUS_SET_CLEAR_BROKE: u32 = 0b100;
pub const SP_STATUS_SET_CLEAR_INTERRUPT: u32 = 0b1000;
pub const SP_STATUS_SET_SET_INTERRUPT: u32 = 0b1_0000;
pub const SP_STATUS_SET_CLEAR_INTERRUPT_ON_BREAK: u32 = 0b1_0000_000;
pub const SP_STATUS_SET_SET_INTERRUPT_ON_BREAK: u32 = 0b10_0000_000;

pub struct RSP {
}

impl RSP {
    fn set_register(reg: RegisterOffset, value: u32) {
        unsafe { SP_BASE_REG.add(reg as usize >> 2).write_volatile(value) }
    }

    fn get_register(reg: RegisterOffset) -> u32 {
        unsafe { SP_BASE_REG.add(reg as usize >> 2).read_volatile() }
    }

    pub fn set_sp_address(value: u32) {
        Self::set_register(RegisterOffset::SPAddress, value);
    }

    pub fn sp_address() -> u32 {
        Self::get_register(RegisterOffset::SPAddress)
    }

    pub fn set_dram_address(value: u32) {
        Self::set_register(RegisterOffset::DRAMAddress, value);
    }

    pub fn dram_address() -> u32 {
        Self::get_register(RegisterOffset::DRAMAddress)
    }

    fn set_read_length(value: u32) {
        Self::set_register(RegisterOffset::ReadLength, value);
    }

    fn set_write_length(value: u32) {
        Self::set_register(RegisterOffset::WriteLength, value);
    }

    pub fn status() -> u32 {
        Self::get_register(RegisterOffset::Status)
    }

    pub fn set_status(value: u32) {
        Self::set_register(RegisterOffset::Status, value);
    }

    pub fn set_pc(value: u32) {
        unsafe { SP_PC_REG.write_volatile(value) }
    }

    pub fn pc() -> u32 {
        unsafe { SP_PC_REG.read_volatile() }
    }

    pub fn start_running(pc: usize) {
        Self::set_pc(pc as u32);

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

    pub fn start_dma_cpu_to_sp(from: *const u8, spmem: u32, length: u32) {
        Self::set_sp_address(spmem);
        Self::set_dram_address(from as usize as u32);
        Self::set_read_length(length);
    }

    pub fn wait_until_rsp_is_halted() {
        while (Self::status() & SP_STATUS_HALT) == 0 {

        }
    }

    pub fn wait_until_rsp_is_halted_and_dma_completed() {
        while (Self::status() & (SP_STATUS_DMA_BUSY | SP_STATUS_HALT)) != SP_STATUS_HALT {

        }
    }

    pub fn wait_until_dma_completed() {
        while (Self::status() & SP_STATUS_DMA_BUSY) != 0 {

        }
    }

    pub fn run_and_wait(pc: usize) {
        Self::start_running(pc);
        Self::wait_until_rsp_is_halted();
    }

    pub const fn get_set_signal_bit(i: u32) -> u32 {
        assert!(i < 8);
        1 << (10 + i * 2)
    }

    pub const fn get_clear_signal_bit(i: u32) -> u32 {
        assert!(i < 8);
        1 << (9 + i * 2)
    }

    pub const fn get_is_signal_bit(i: u32) -> u32 {
        assert!(i < 8);
        1 << (7 + i)
    }

    pub fn set_signal(i: u32) {
        Self::set_status(Self::get_set_signal_bit(i));
    }

    pub fn clear_signal(i: u32) {
        Self::set_status(Self::get_clear_signal_bit(i));
    }

    pub fn is_signal(i: u32) -> bool {
        (Self::status() & Self::get_is_signal_bit(i)) != 0
    }

    pub fn set_interrupt() {
        Self::set_status(SP_STATUS_SET_SET_INTERRUPT);
    }

    pub fn clear_interrupt() {
        Self::set_status(SP_STATUS_SET_CLEAR_INTERRUPT);
    }

    pub fn set_interrupt_on_break() {
        Self::set_status(SP_STATUS_SET_SET_INTERRUPT_ON_BREAK);
    }

    pub fn clear_interrupt_on_break() {
        Self::set_status(SP_STATUS_SET_CLEAR_INTERRUPT_ON_BREAK);
    }

    pub fn is_interrupt_on_break() -> bool {
        (Self::status() & SP_STATUS_INTERRUPT_ON_BREAK) != 0
    }

    pub fn clear_broke() {
        Self::set_status(SP_STATUS_SET_CLEAR_BROKE);
    }

    pub fn set_semaphore(value: u32) {
        Self::set_register(RegisterOffset::Semaphore, value);
    }

    pub fn semaphore() -> u32 {
        Self::get_register(RegisterOffset::Semaphore)
    }

}
