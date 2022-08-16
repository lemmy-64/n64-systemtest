use alloc::boxed::Box;
use alloc::{format, vec};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use arbitrary_int::u12;

use crate::graphics::color::{Color, RGBA5551};
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
//  - Similar to RSP side, it is possible to set and clear bits at the same time. Write a test to see what happens

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

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(0x1238u32),
            Box::new(0u32),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<u32>() {
            Some(start_value) => {
                // Note: This test leaves the RDP in a weird state if it sets start but not end. There,
                // do all asserts at the end
                let current_before = RDP::current();
                // Set freeze so that we can freely read/write registers without the RDP actually doing anything
                unsafe { RDP::set_status(DP_SET_STATUS_SET_FREEZE); }
                soft_assert_eq((RDP::status() & DP_STATUS_FREEZE) != 0, true, "RDP was told to freeze, but it didn't")?;

                // Pre-req: Ensure START isn't currently valid
                soft_assert_eq((RDP::status() & DP_STATUS_START_VALID) != 0, false, "start-valid should be false when entering the test")?;
                soft_assert_eq((RDP::status() & DP_STATUS_END_VALID) != 0, false, "end-valid should be false when entering the test")?;

                // 1: Set start
                RDP::set_start(*start_value);
                let status1 = RDP::status();
                let start1 = RDP::start();

                // 2: While start is valid, further writes to start are ignored
                RDP::set_start(0x12_3450);
                let status2 = RDP::status();
                let start2 = RDP::start();
                let current2 = RDP::current();

                // 3: Write end. This should unset start-valid
                RDP::set_end(*start_value);
                let status3 = RDP::status();
                let end3 = RDP::start();
                let current3 = RDP::current();

                // Unfreeze. START=END, so nothing should happen
                unsafe { RDP::set_status(DP_SET_STATUS_CLEAR_FREEZE); }
                soft_assert_eq((RDP::status() & DP_STATUS_FREEZE) != 0, false, "RDP was told to stop being frozen but it's still frozen")?;

                // Now verify the states during the test
                soft_assert_eq((status1 & DP_STATUS_START_VALID) != 0, true, "start-valid should be set after writing start-address")?;
                soft_assert_eq((status1 & DP_STATUS_END_VALID) != 0, false, "end-valid should be false after writing start-address")?;
                soft_assert_eq(start1, *start_value, "RDP start address after writing")?;

                soft_assert_eq((status2 & DP_STATUS_START_VALID) != 0, true, "Writing start after start should keep start-valid true")?;
                soft_assert_eq((status2 & DP_STATUS_END_VALID) != 0, false, "Writing start while start-valid should not set end-valid")?;
                soft_assert_eq(start2, *start_value, "Writes to start address should be ignored while start-valid is set")?;
                soft_assert_eq(current2, current_before, "Writes to start should not affect current")?;

                soft_assert_eq((status3 & DP_STATUS_START_VALID) != 0, false, "After writing end, start-valid should be cleared")?;
                soft_assert_eq((status3 & DP_STATUS_END_VALID) != 0, false, "After writing end, end-valid should not be set (at least not while frozen)")?;
                soft_assert_eq(end3, *start_value, "Reading back end address after write to it (3)")?;
                soft_assert_eq(current3, *start_value, "Current should be equal to start (which is also equal to end) (3)")?;


                Ok(())
            }
            _ => Err("Value is not valid".to_string())
        }
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
        let mut framebuffer = UncachedHeapMemory::<RGBA5551>::new_with_init_value(WIDTH * HEIGHT, RGBA5551::BLACK);

        // Assemble a simple RDP program that fulls the framebuffer with RGBA1555::BLUE
        let mut assembler = RDPAssembler::new();
        let rect = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_u32(WIDTH as u32 - 1), U10_2::from_u32(HEIGHT as u32 - 1));
        assembler.set_framebuffer_image(Format::RGBA, PixelSize::Bits16, u12::new((WIDTH - 1).try_into().unwrap()), &mut framebuffer);
        assembler.set_scissor(&rect);
        assembler.set_othermode(Othermode::new()
            .with_cycle_type(CycleType::Fill));
        assembler.set_fillcolor16(RGBA5551::BLUE, RGBA5551::BLUE);
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
        soft_assert_eq(framebuffer.read(0), RGBA5551::BLUE, "Auxiliary framebuffer should be filled with BLUE")?;

        // Advance until the second sync_full instruction
        RDP::set_end(assembler.end() as u32);
        wait_for_status(DP_STATUS_COMMAND_BUFFER_READY)?;

        Ok(())
    }
}

fn run_from_dmem_test<F: FnOnce(u32) -> (u32, u32)>(get_dmem_range: F) -> Result<(), String> {
    const WIDTH: usize = 8;
    const HEIGHT: usize = 8;
    let mut framebuffer = UncachedHeapMemory::<RGBA5551>::new_with_init_value(WIDTH * HEIGHT, RGBA5551::BLACK);

    // Assemble a simple RDP program that fills the framebuffer with RGBA1555::GREEN
    let mut assembler = RDPAssembler::new();
    let rect = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_u32(WIDTH as u32 - 1), U10_2::from_u32(HEIGHT as u32 - 1));
    assembler.set_framebuffer_image(Format::RGBA, PixelSize::Bits16, u12::new((WIDTH - 1).try_into().unwrap()), &mut framebuffer);
    assembler.set_scissor(&rect);
    assembler.set_othermode(Othermode::new()
        .with_cycle_type(CycleType::Fill));
    assembler.set_fillcolor16(RGBA5551::GREEN, RGBA5551::GREEN);
    assembler.filled_rectangle(&rect);
    assembler.sync_pipe();
    assembler.sync_full();

    let length = (assembler.end() - assembler.start()) as u32;
    let (dmem_start, dmem_end) = get_dmem_range(length);

    soft_assert_eq((dmem_end - dmem_start) & 0xFFF, length, "(end-start) must be equal to length")?;

    // DMA to DMEM
    let length = (assembler.end() - assembler.start()) as u32;
    RSP::start_dma_cpu_to_sp(assembler.start() as *const u8, dmem_start, length);
    RSP::wait_until_dma_completed();

    unsafe { RDP::set_status(DP_SET_STATUS_SET_XBUS); }

    // Set start and end to beginning
    RDP::set_start(dmem_start);
    RDP::set_end(dmem_end);

    wait_for_status(DP_STATUS_COMMAND_BUFFER_READY | DP_STATUS_XBUS)?;

    unsafe { RDP::set_status(DP_SET_STATUS_CLEAR_XBUS); }

    soft_assert_eq(RDP::current(), dmem_end, "RDP current should be equal to END after writing END (and waiting for the RDP to finish)")?;

    soft_assert_eq(framebuffer.read(0), RGBA5551::GREEN, "Auxiliary framebuffer should be filled with GREEN")?;

    Ok(())
}

pub struct RunFromDMEM {}

impl Test for RunFromDMEM {
    fn name(&self) -> &str { "RDP STATUS: Run from DMEM (xbus)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_from_dmem_test(|length| (0, length))
    }
}

pub struct RunFromDMEMEnd {}

impl Test for RunFromDMEMEnd {
    fn name(&self) -> &str { "RDP STATUS: Run from DMEM (xbus) (end of dmem)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_from_dmem_test(|length| (0x1000 - length, 0x1000))
    }
}

/// Overflowing DMEM is possible, as long as END is not masked. It seems
/// the RDP is doing an internal START < END check
pub struct RunFromDMEMOverflow {}

impl Test for RunFromDMEMOverflow {
    fn name(&self) -> &str { "RDP STATUS: Run from DMEM (xbus) (overflowing dmem)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_from_dmem_test(|length| (0xFF0, 0xFF0 + length))
    }
}