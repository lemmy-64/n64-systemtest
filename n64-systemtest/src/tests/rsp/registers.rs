use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;

use crate::mi;
use crate::rsp::rsp::{RSP, SP_STATUS_HALT, SP_STATUS_SET_SET_HALT};
use crate::rsp::rsp_assembler::{CP0Register, GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Write various memory mapped registers, both from CPU and RSP. Lessons learned:
// - SP.STATUS had various bits to clear and set. What happens if both are set at once? The target bit
//   isn't changed at all.
// - If Semaphore is written to (value doesn't matter), the next read will return 0. Otherwise it returns 1
// - Semaphore doesn't distinguish between CPU and RSP in any way - if the CPU writes something, the RSP will read 0 (and vice versa)
// - The RSP can stop itself by setting STATUS.HALT. When doing this, Status.broke will not be set (unlike BREAK)

pub struct SetClearInterrupt {

}

impl Test for SetClearInterrupt {
    fn name(&self) -> &str { "SP Set/Clear Interrupt" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        RSP::set_interrupt();
        soft_assert_eq(mi::is_sp_interrupt(), true, "MI INTR should contain SP after setting within SP_STATUS")?;
        RSP::clear_interrupt();
        soft_assert_eq(mi::is_sp_interrupt(), false, "MI INTR should not contain SP after clearing within SP_STATUS")?;
        RSP::set_interrupt();
        soft_assert_eq(mi::is_sp_interrupt(), true, "MI INTR should contain SP after setting within SP_STATUS")?;

        // Setting both set and clear at the same time: No change expected
        RSP::set_status(crate::rsp::rsp::SP_STATUS_SET_SET_INTERRUPT | crate::rsp::rsp::SP_STATUS_SET_CLEAR_INTERRUPT);
        soft_assert_eq(mi::is_sp_interrupt(), true, "If both Interrupt set and clear are set, nothing should change")?;

        RSP::clear_interrupt();
        soft_assert_eq(mi::is_sp_interrupt(), false, "MI INTR should not contain SP after clearing within SP_STATUS")?;

        RSP::set_status(crate::rsp::rsp::SP_STATUS_SET_SET_INTERRUPT | crate::rsp::rsp::SP_STATUS_SET_CLEAR_INTERRUPT);
        soft_assert_eq(mi::is_sp_interrupt(), false, "If both Interrupt set and clear are set, nothing should change")?;

        Ok(())
    }
}

fn test_signal(i: u32) -> Result<(), String> {
    RSP::set_signal(i);
    soft_assert_eq(RSP::is_signal(i), true, "SP_STATUS should contain signal after explicitly setting it (1)")?;
    RSP::clear_signal(i);
    soft_assert_eq(RSP::is_signal(i), false, "SP_STATUS should not contain signal after explicitly clearing it (2)")?;
    RSP::set_signal(i);
    soft_assert_eq(RSP::is_signal(i), true, "SP_STATUS should contain signal after explicitly setting it (3)")?;

    // Set and clear at the same time: No change expected
    RSP::set_status(0b11 << (9 + i * 2));
    soft_assert_eq(RSP::is_signal(i), true, "SP_STATUS should not change signal after setting and clearing it at once (4)")?;

    RSP::clear_signal(i);
    soft_assert_eq(RSP::is_signal(i), false, "SP_STATUS should not contain signal after explicitly clearing it (5)")?;

    RSP::set_status(0b11 << (9 + i * 2));
    soft_assert_eq(RSP::is_signal(i), false, "SP_STATUS should not change signal after setting and clearing it at once (6)")?;

    Ok(())
}

pub struct SetClearSignal {

}

impl Test for SetClearSignal {
    fn name(&self) -> &str { "SP Set/Clear Signal" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0u32),
            Box::new(1u32),
            Box::new(2u32),
            Box::new(3u32),
            Box::new(4u32),
            Box::new(5u32),
            Box::new(6u32),
            Box::new(7u32),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        let i =(*value).downcast_ref::<u32>().unwrap();
        test_signal(*i)?;
        Ok(())
    }
}

pub struct SetClearInterruptOnBreak {

}

impl Test for SetClearInterruptOnBreak {
    fn name(&self) -> &str { "SP Set/Clear Interrupt on Break" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        RSP::set_interrupt_on_break();
        soft_assert_eq(RSP::is_interrupt_on_break(), true, "SP_STATUS should be INTERRUPT_ON_BREAK after explicitly setting it (1)")?;
        RSP::clear_interrupt_on_break();
        soft_assert_eq(RSP::is_interrupt_on_break(), false, "SP_STATUS should not be INTERRUPT_ON_BREAK after explicitly clearing it (2)")?;
        RSP::set_interrupt_on_break();
        soft_assert_eq(RSP::is_interrupt_on_break(), true, "SP_STATUS should be INTERRUPT_ON_BREAK after explicitly setting it (3)")?;

        // Set and clear at the same time: No change expected
        RSP::set_status(crate::rsp::rsp::SP_STATUS_SET_SET_INTERRUPT_ON_BREAK | crate::rsp::rsp::SP_STATUS_SET_CLEAR_INTERRUPT_ON_BREAK);
        soft_assert_eq(RSP::is_interrupt_on_break(), true, "If both INTERRUPT_ON_BREAK set and clear are set, nothing should change (4)")?;

        RSP::clear_interrupt_on_break();
        soft_assert_eq(RSP::is_interrupt_on_break(), false, "SP_STATUS should not be INTERRUPT_ON_BREAK after explicitly clearing it (5)")?;

        RSP::set_status(crate::rsp::rsp::SP_STATUS_SET_SET_INTERRUPT_ON_BREAK | crate::rsp::rsp::SP_STATUS_SET_CLEAR_INTERRUPT_ON_BREAK);
        soft_assert_eq(RSP::is_interrupt_on_break(), false, "If both INTERRUPT_ON_BREAK set and clear are set, nothing should change (6)")?;

        Ok(())
    }
}

pub struct SetClearHalt {

}

impl Test for SetClearHalt {
    fn name(&self) -> &str { "SP Set/Clear Halt" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // For halt, we'll do the same Set-And-Clear-at-the-same-time test. But as that actually starts the RSP, we'll check the RSP PC to see if it ever got launched

        let mut assembler = RSPAssembler::new(0);
        assembler.write_nop();
        assembler.write_break();

        RSP::set_pc(0);

        // Set both CLEAR_HALT and SET_HALT
        RSP::set_status(crate::rsp::rsp::SP_STATUS_SET_CLEAR_HALT |
            crate::rsp::rsp::SP_STATUS_SET_SET_HALT |
            crate::rsp::rsp::SP_STATUS_SET_CLEAR_INTERRUPT |
            crate::rsp::rsp::SP_STATUS_SET_CLEAR_INTERRUPT_ON_BREAK);
        RSP::wait_until_rsp_is_halted();

        soft_assert_eq(RSP::pc(), 0x0, "If both SET_HALT and CLEAR_HALT are set, the RSP isn't supposed to run")?;

        // As a sanity-test, just do CLEAR_HALT. That should actually launch the RSP
        // Set both CLEAR_HALT and SET_HALT
        RSP::set_status(crate::rsp::rsp::SP_STATUS_SET_CLEAR_HALT);
        RSP::wait_until_rsp_is_halted();

        soft_assert_eq(RSP::pc(), 0x8, "RSP PC isn't as expected after running")?;

        Ok(())
    }
}

pub struct SPRegisterReadAccessOnRSP {

}

impl Test for SPRegisterReadAccessOnRSP {
    fn name(&self) -> &str { "SP Register Read Access From RSP" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Perform a DMA so that the the first four registers are filled with known values
        RSP::start_dma_cpu_to_sp(0x10 as *const u8, 0x50, 15);
        RSP::wait_until_dma_completed();

        let mut assembler = RSPAssembler::new(16);
        assembler.write_nop();
        assembler.write_mfc0(CP0Register::SPAddress, GPR::S0);
        assembler.write_mfc0(CP0Register::DRAMAddress, GPR::S1);
        assembler.write_mfc0(CP0Register::ReadLength, GPR::S2);
        assembler.write_mfc0(CP0Register::WriteLength, GPR::S3);
        assembler.write_mfc0(CP0Register::SPStatus, GPR::S4);
        assembler.write_mfc0(CP0Register::DmaFull, GPR::S5);
        assembler.write_mfc0(CP0Register::DmaBusy, GPR::S6);
        assembler.write_sw(GPR::S0, GPR::R0, 0x0);
        assembler.write_sw(GPR::S1, GPR::R0, 0x4);
        assembler.write_sw(GPR::S2, GPR::R0, 0x8);
        assembler.write_sw(GPR::S3, GPR::R0, 0xC);
        assembler.write_sw(GPR::S4, GPR::R0, 0x10);
        assembler.write_sw(GPR::S5, GPR::R0, 0x14);
        assembler.write_sw(GPR::S6, GPR::R0, 0x18);
        assembler.write_break();

        RSP::clear_broke();
        RSP::set_signal(4); // so that status below won't be 0
        RSP::run_and_wait(16);
        RSP::clear_signal(4);

        soft_assert_eq(RSP::sp_address(), 0x60, "SP sp-address (read via CPU)")?;
        soft_assert_eq(SPMEM::read(0x0), 0x60, "SP sp-address (read via RSP MFC0)")?;

        soft_assert_eq(RSP::dram_address(), 0x20, "SP dram-address (read via CPU)")?;
        soft_assert_eq(SPMEM::read(0x4), 0x20, "SP dram-address (read via RSP MFC0)")?;

        soft_assert_eq(SPMEM::read(0x8), 0xFF8, "SP read length")?;
        soft_assert_eq(SPMEM::read(0xC), 0xFF8, "SP write length")?;
        soft_assert_eq(SPMEM::read(0x10), 0x800, "SP status")?;
        soft_assert_eq(SPMEM::read(0x14), 0, "SP dmaFull")?;
        soft_assert_eq(SPMEM::read(0x18), 0, "SP dmaBusy")?;

        Ok(())
    }
}

pub struct SemaphoreRegisterCPUOnly {

}

impl Test for SemaphoreRegisterCPUOnly {
    fn name(&self) -> &str { "SP Semaphore Register (CPU only)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Writing any value to the semaphore register means that the next value read will be 0. Then 1
        for test_write_value in [0u32, 1, 0xFFFFFFFF] {
            RSP::set_semaphore(test_write_value);
            soft_assert_eq(RSP::semaphore(), 0, "First read after write")?;
            soft_assert_eq(RSP::semaphore(), 1, "2nd read after write")?;
            soft_assert_eq(RSP::semaphore(), 1, "3rd read after write")?;
            soft_assert_eq(RSP::semaphore(), 1, "4th read after write")?;
            soft_assert_eq(RSP::semaphore(), 1, "5th read after write")?;
        }

        // Also try writing twice without reading. Same thing should happen
        RSP::set_semaphore(6);
        RSP::set_semaphore(6);
        soft_assert_eq(RSP::semaphore(), 0, "First read after write")?;
        soft_assert_eq(RSP::semaphore(), 1, "2nd read after write")?;
        soft_assert_eq(RSP::semaphore(), 1, "3rd read after write")?;

        Ok(())
    }
}

pub struct SemaphoreRegisterRSPOnly {

}

impl Test for SemaphoreRegisterRSPOnly {
    fn name(&self) -> &str { "SP Semaphore Register (RSP only)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut assembler = RSPAssembler::new(0);
        assembler.write_nop();
        assembler.write_mtc0(CP0Register::Semaphore, GPR::R0);
        assembler.write_mfc0(CP0Register::Semaphore, GPR::S0);
        assembler.write_mfc0(CP0Register::Semaphore, GPR::S1);
        assembler.write_mfc0(CP0Register::Semaphore, GPR::S2);
        assembler.write_mfc0(CP0Register::Semaphore, GPR::S3);
        assembler.write_mfc0(CP0Register::Semaphore, GPR::S4);
        assembler.write_sw(GPR::S0, GPR::R0, 0x0);
        assembler.write_sw(GPR::S1, GPR::R0, 0x4);
        assembler.write_sw(GPR::S2, GPR::R0, 0x8);
        assembler.write_sw(GPR::S3, GPR::R0, 0xC);
        assembler.write_sw(GPR::S4, GPR::R0, 0x10);
        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x0), 0, "First read after write")?;
        soft_assert_eq(SPMEM::read(0x4), 1, "2nd read after write")?;
        soft_assert_eq(SPMEM::read(0x8), 1, "3rd read after write")?;
        soft_assert_eq(SPMEM::read(0xC), 1, "4th read after write")?;
        soft_assert_eq(SPMEM::read(0x10), 1, "5th read after write")?;

        Ok(())
    }
}

pub struct SemaphoreRegisterMixed {

}

impl Test for SemaphoreRegisterMixed {
    fn name(&self) -> &str { "SP Semaphore Register (CPU and RSP)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut assembler = RSPAssembler::new(0);
        assembler.write_nop();
        assembler.write_mfc0(CP0Register::Semaphore, GPR::S0);
        assembler.write_mfc0(CP0Register::Semaphore, GPR::S1);
        assembler.write_mfc0(CP0Register::Semaphore, GPR::S2);
        assembler.write_mfc0(CP0Register::Semaphore, GPR::S3);
        assembler.write_mfc0(CP0Register::Semaphore, GPR::S4);
        assembler.write_sw(GPR::S0, GPR::R0, 0x0);
        assembler.write_sw(GPR::S1, GPR::R0, 0x4);
        assembler.write_sw(GPR::S2, GPR::R0, 0x8);
        assembler.write_break();

        // Write to semaphore from CPU
        RSP::set_semaphore(0);

        // Then run RSP
        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x0), 0, "1st RSP read after CPU write")?;
        soft_assert_eq(SPMEM::read(0x4), 1, "2nd RSP read after CPU write")?;
        soft_assert_eq(SPMEM::read(0x8), 1, "3rd RSP read after CPU write")?;

        // Then read semaphore from CPU
        soft_assert_eq(RSP::semaphore(), 1, "First CPU read after RSP finished")?;

        Ok(())
    }
}

pub struct SemaphoreRegisterMixed2 {

}

impl Test for SemaphoreRegisterMixed2 {
    fn name(&self) -> &str { "SP Semaphore Register (CPU and RSP)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // ensure it is 1 by reading once
        RSP::semaphore();

        soft_assert_eq(RSP::semaphore(), 1, "Initial Semaphore value")?;

        let mut assembler = RSPAssembler::new(0);
        assembler.write_nop();
        assembler.write_mtc0(CP0Register::Semaphore, GPR::R0);
        assembler.write_break();

        // Then run RSP
        RSP::run_and_wait(0);

        soft_assert_eq(RSP::semaphore(), 0, "1st CPU read after RSP write")?;
        soft_assert_eq(RSP::semaphore(), 1, "2nd CPU read after RSP write")?;
        soft_assert_eq(RSP::semaphore(), 2, "3rd CPU read after RSP write")?;

        Ok(())
    }
}

pub struct RSPHaltItselfWithoutBreak {

}

impl Test for RSPHaltItselfWithoutBreak {
    fn name(&self) -> &str { "SP halt itself through Status.halt" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        RSP::clear_broke();
        let mut assembler = RSPAssembler::new(0);
        assembler.write_nop();
        assembler.write_li(GPR::AT, SP_STATUS_SET_SET_HALT);
        assembler.write_mtc0(CP0Register::SPStatus, GPR::AT);
        assembler.write_nop();
        assembler.write_nop();
        assembler.write_nop();
        assembler.write_break();  // this should never get executed
        RSP::run_and_wait(0);

        soft_assert_eq(RSP::status(), SP_STATUS_HALT, "RSP status after self-halt without BREAK")?;

        Ok(())
    }
}

