use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::graphics::color::{Color, RGBA1555};
use crate::rdp::fixedpoint::U10_2;
use crate::rdp::modes::{CycleType, Format, Othermode, PixelSize};
use crate::rdp::rdp::{DP_SET_STATUS_CLEAR_FREEZE, DP_SET_STATUS_CLEAR_XBUS, DP_SET_STATUS_SET_FREEZE, DP_SET_STATUS_SET_XBUS, DP_STATUS_COMMAND_BUFFER_READY, DP_STATUS_END_VALID, DP_STATUS_FREEZE, DP_STATUS_PIPE_BUSY, DP_STATUS_START_GCLK, DP_STATUS_START_VALID, DP_STATUS_XBUS, RDP};
use crate::rdp::rdp_assembler::{RDPAssembler, RDPRectangle};
use crate::rsp::rsp::RSP;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq2};
use crate::uncached_memory::UncachedHeapMemory;

pub mod filled_triangle;

// TODO:
//  - Make a test that uses FREEZE. It should not execute the RDP list until the RDP is unfrozen
//  - For freeze test perfection: CURRENT should advance up to START+240 as the commands are
//    dma'ed even if the RDP is frozen
//  - It is possible to DMA two command lists to the RDP. Do a test for double-buffering

fn wait_for_status(goal: u32) -> Result<(), String> {
    for _ in 0..10_000 {
        if RDP::status() == goal {
            return Ok(());
        }
    }

    Err(format!("Time out waiting for RDP status 0x{:x}. RDP status at timeout: 0x{:x}", goal, RDP::status()))
}

pub struct StartAndEndMasking {}

impl Test for StartAndEndMasking {
    fn name(&self) -> &str { "RDP START & END REG (masking)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Set freeze so that we can freely read/write registers without the RDP actually doing anything
        unsafe { RDP::set_status(DP_SET_STATUS_SET_FREEZE); }

        soft_assert_eq((RDP::status() & DP_STATUS_FREEZE) != 0, true, "RDP was told to freeze, but it didn't")?;

        for value in [0xFFF, 0xFF_FFFF, 0x12FF_FFFF, 0x1280_0000, 0xFFFF_FFFF, 0] {
            // Don't assert before writing end! The RDP is in a funny state unless both registers are being written to
            RDP::set_start(value);
            RDP::set_end(value);
            soft_assert_eq2(RDP::start(), value & 0xFF_FFF8, || format!("RDP START isn't masked properly on write (0x{:x} was written while end is 0x{:x})", value, RDP::end()))?;
            soft_assert_eq2(RDP::current(), value & 0xFF_FFF8, || format!("RDP START isn't masked properly on write (0x{:x} was written while end is 0x{:x})", value, RDP::end()))?;
            soft_assert_eq2(RDP::end(), value & 0xFF_FFF8, || format!("RDP END isn't masked properly on write (0x{:x} was written)", value))?;
        }

        // Unfreeze to leave the RDP is a normal state
        unsafe { RDP::set_status(DP_SET_STATUS_CLEAR_FREEZE); }
        soft_assert_eq((RDP::status() & DP_STATUS_FREEZE) != 0, false, "RDP was told to stop being frozen but it's still frozen")?;

        Ok(())
    }
}

pub struct StartIsValidFlag {}

impl Test for StartIsValidFlag {
    fn name(&self) -> &str { "RSP STATUS: start-valid" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let current_before = RDP::current();
        // Set freeze so that we can freely read/write registers without the RDP actually doing anything
        unsafe { RDP::set_status(DP_SET_STATUS_SET_FREEZE); }
        soft_assert_eq((RDP::status() & DP_STATUS_FREEZE) != 0, true, "RDP was told to freeze, but it didn't")?;

        // Pre-req: Ensure START isn't currently valid
        soft_assert_eq((RDP::status() & DP_STATUS_START_VALID) != 0, false, "start-valid should be false when entering the test")?;
        soft_assert_eq((RDP::status() & DP_STATUS_END_VALID) != 0, false, "end-valid should be false when entering the test")?;

        // Set any start address
        RDP::set_start(0x3210);
        soft_assert_eq((RDP::status() & DP_STATUS_START_VALID) != 0, true, "start-valid should be set after writing start-address")?;
        soft_assert_eq((RDP::status() & DP_STATUS_END_VALID) != 0, false, "end-valid should be false after writing start-address")?;
        soft_assert_eq(RDP::start(), 0x3210, "RDP start address after writing")?;

        // While start is valid, further writes to start are ignored
        //RDP::set_start(0x12_3450);
        soft_assert_eq((RDP::status() & DP_STATUS_START_VALID) != 0, true, "start-valid should stay valid")?;
        soft_assert_eq((RDP::status() & DP_STATUS_END_VALID) != 0, false, "end-valid should stay false")?;
        soft_assert_eq(RDP::start(), 0x3210, "Writes to start address should be ignored while start-valid is set")?;

        // Writing start shouldn't change current
        soft_assert_eq(RDP::current(), current_before, "Writes to start should not affect current")?;

        // Write end. This should unset start-valid
        RDP::set_end(0x3210);
        soft_assert_eq((RDP::status() & DP_STATUS_START_VALID) != 0, false, "start-valid should stay valid")?;
        soft_assert_eq((RDP::status() & DP_STATUS_END_VALID) != 0, false, "end-valid should still be false after writing")?;
        soft_assert_eq(RDP::end(), 0x3210, "Reading back end address after write to it")?;
        soft_assert_eq(RDP::current(), 0x3210, "Current should be equal to start (which is also equal to end)")?;

        // Set freeze so that we can freely read/write registers without the RDP actually starting
        unsafe { RDP::set_status(DP_SET_STATUS_CLEAR_FREEZE); }
        soft_assert_eq((RDP::status() & DP_STATUS_FREEZE) != 0, false, "RDP was told to stop being frozen but it's still frozen")?;

        Ok(())
    }
}

pub struct StatusFlagsDuringRun {}

impl Test for StatusFlagsDuringRun {
    fn name(&self) -> &str { "RDP STATUS: Flags during a run" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const WIDTH: usize = 8;
        const HEIGHT: usize = 8;
        let mut framebuffer = UncachedHeapMemory::<RGBA1555>::new_with_init_value(WIDTH * HEIGHT, RGBA1555::BLACK);

        // Assemble a simple RDP program that fulls the framebuffer with RGBA1555::BLUE
        let mut assembler = RDPAssembler::new();
        let rect = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_u32(WIDTH as u32 - 1), U10_2::from_u32(HEIGHT as u32 - 1));
        assembler.set_framebuffer_image(Format::RGBA, PixelSize::Bits16, WIDTH - 1, &mut framebuffer);
        assembler.set_scissor(&rect);
        assembler.set_othermode(Othermode::new()
            .with_cycle_type(CycleType::Fill));
        assembler.set_fillcolor16(RGBA1555::BLUE, RGBA1555::BLUE);
        assembler.filled_rectangle(&rect);
        assembler.sync_pipe();
        assembler.sync_full();
        assembler.sync_full();

        // Set start and end to beginning
        RDP::set_start(assembler.start() as u32);
        RDP::set_end(assembler.start() as u32);
        soft_assert_eq(RDP::current(), assembler.start() as u32, "RDP current should be equal to START after writing END")?;

        wait_for_status(DP_STATUS_COMMAND_BUFFER_READY | DP_STATUS_PIPE_BUSY | DP_STATUS_START_GCLK)?;
        soft_assert_eq(RDP::current(), assembler.start() as u32, "RDP current should be equal to START after writing END")?;

        // Move forward one instruction. Hardware will briefly set DP_STATUS_DMA_BUSY as it copies an instruction over
        // For now we don't assert for that as few emulators probably implement this exactly
        RDP::set_end(assembler.start() as u32 + 8);
        wait_for_status(DP_STATUS_COMMAND_BUFFER_READY | DP_STATUS_PIPE_BUSY | DP_STATUS_START_GCLK)?;

        // Advance until the right before the sync_full instructions
        RDP::set_end(assembler.end() as u32 - 16);
        wait_for_status(DP_STATUS_COMMAND_BUFFER_READY | DP_STATUS_PIPE_BUSY | DP_STATUS_START_GCLK)?;

        // Advance until the first sync_full instruction
        RDP::set_end(assembler.end() as u32 - 8);
        wait_for_status(DP_STATUS_COMMAND_BUFFER_READY)?;

        // Test a single pixel in the framebuffer to ensure the filled_rectangle actually happened. Other tests will test that specific command more in-depth
        soft_assert_eq(framebuffer.read(0), RGBA1555::BLUE, "Auxiliary framebuffer should be filled with BLUE")?;

        // Advance until the second sync_full instruction
        RDP::set_end(assembler.end() as u32);
        wait_for_status(DP_STATUS_COMMAND_BUFFER_READY)?;

        Ok(())
    }
}

pub struct RunFromDMEM {}

impl Test for RunFromDMEM {
    fn name(&self) -> &str { "RDP STATUS: Run from DMEM (xbus)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const WIDTH: usize = 8;
        const HEIGHT: usize = 8;
        let mut framebuffer = UncachedHeapMemory::<RGBA1555>::new_with_init_value(WIDTH * HEIGHT, RGBA1555::BLACK);

        // Assemble a simple RDP program that fills the framebuffer with RGBA1555::GREEN
        let mut assembler = RDPAssembler::new();
        let rect = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_u32(WIDTH as u32 - 1), U10_2::from_u32(HEIGHT as u32 - 1));
        assembler.set_framebuffer_image(Format::RGBA, PixelSize::Bits16, WIDTH - 1, &mut framebuffer);
        assembler.set_scissor(&rect);
        assembler.set_othermode(Othermode::new()
            .with_cycle_type(CycleType::Fill));
        assembler.set_fillcolor16(RGBA1555::GREEN, RGBA1555::GREEN);
        assembler.filled_rectangle(&rect);
        assembler.sync_pipe();
        assembler.sync_full();

        // DMA to DMEM
        let length = (assembler.end() - assembler.start()) as u32;
        RSP::start_dma_cpu_to_sp(assembler.start() as *const u8, 0, length);
        RSP::wait_until_dma_completed();

        unsafe { RDP::set_status(DP_SET_STATUS_SET_XBUS); }

        // Set start and end to beginning
        RDP::set_start(0);
        RDP::set_end(length);

        wait_for_status(DP_STATUS_COMMAND_BUFFER_READY | DP_STATUS_XBUS)?;

        unsafe { RDP::set_status(DP_SET_STATUS_CLEAR_XBUS); }

        soft_assert_eq(RDP::current(), length, "RDP current should be equal to START after writing END")?;

        soft_assert_eq(framebuffer.read(0), RGBA1555::GREEN, "Auxiliary framebuffer should be filled with GREEN")?;

        Ok(())
    }
}