use crate::rdp::rdp_assembler::RDPAssembler;

const DP_BASE_REG: *mut u32 = 0xA410_0000 as *mut u32;

#[allow(dead_code)]
enum RegisterOffset {
    Start = 0x00,
    End = 0x04,
    Current = 0x08,
    Status = 0x0C,
}

pub const DP_STATUS_XBUS: u32 = 0x1;
pub const DP_STATUS_FREEZE: u32 = 0x2;
pub const DP_STATUS_START_GCLK: u32 = 0x8;
pub const DP_STATUS_PIPE_BUSY: u32 = 0x20;
pub const DP_STATUS_COMMAND_BUFFER_READY: u32 = 0x80;
pub const DP_STATUS_END_VALID: u32 = 0x200;
pub const DP_STATUS_START_VALID: u32 = 0x400;

pub const DP_SET_STATUS_CLEAR_XBUS: u32 = 0x1;
pub const DP_SET_STATUS_SET_XBUS: u32 = 0x2;
pub const DP_SET_STATUS_CLEAR_FREEZE: u32 = 0x4;
pub const DP_SET_STATUS_SET_FREEZE: u32 = 0x8;

pub struct RDP {
}

impl RDP {
    fn set_register(reg: RegisterOffset, value: u32) {
        unsafe { DP_BASE_REG.add(reg as usize >> 2).write_volatile(value) }
    }

    fn get_register(reg: RegisterOffset) -> u32 {
        unsafe { DP_BASE_REG.add(reg as usize >> 2).read_volatile() }
    }

    pub fn set_start(value: u32) {
        Self::set_register(RegisterOffset::Start, value);
    }

    pub fn start() -> u32 {
        Self::get_register(RegisterOffset::Start)
    }

    pub fn set_end(value: u32) {
        Self::set_register(RegisterOffset::End, value);
    }

    pub fn end() -> u32 {
        Self::get_register(RegisterOffset::End)
    }

    pub fn current() -> u32 {
        Self::get_register(RegisterOffset::Current)
    }

    pub unsafe fn set_status(value: u32) {
        Self::set_register(RegisterOffset::Status, value);
    }

    pub fn status() -> u32 { Self::get_register(RegisterOffset::Status) }

    /// Runs the RDP and immediately returns
    /// This is marked as unsafe as the memory that is being written to by the RDP (framebuffer/depth-buffer)
    /// might not be valid by the time this finishes
    pub unsafe fn start_running(start: usize, end: usize) {
        Self::set_start(start as u32);
        Self::set_end(end as u32);
    }

    pub fn wait_until_finished(end: usize) {
        while Self::current() != (end as u32) {
            // la da di, la de dau
        }
    }

    /// Runs the RDP and waits until it is done.
    pub fn run_and_wait(assembler: &RDPAssembler) {
        let end = assembler.end();
        unsafe { Self::start_running(assembler.start(), end); }
        Self::wait_until_finished(end);
    }
}
