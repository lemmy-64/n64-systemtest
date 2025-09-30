use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq2;
use crate::uncached_memory::UncachedHeapMemory;

// TODO: Tests for SWL/SWR/SDL/SDR?

fn test_repeat(step: u32, func: fn(u32, u32, u64, i32, &mut UncachedHeapMemory<u64>) -> Result<(), String>) -> Result<(), String> {
    // align to cache line
    let mut buf = UncachedHeapMemory::<u64>::new_with_align(32, 256);
    let value = 0x1234_5678_9ABC_DEF1;
    let mi_regs = 0xA430_0000u32 as i32;
    for length in 1..=0x80 {
        let mut start = 0;
        while start <= 16 {
            // Fill with 0xFF to detect masking done by 8-and 16-bit writes
            for i in 0..buf.count() {
                buf.write(i, !0);
            }
            func(start, length, value, mi_regs, &mut buf)?;
            start += step;
        }
    }

    Ok(())
}

pub struct SB {}

impl Test for SB {
    fn name(&self) -> &str { "MI Repeat: SB" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_repeat(1, |start, length, value, mi_regs, buf| {
            unsafe {
                let ptr = buf.as_ptr().cast::<u8>().add(start as usize);
                asm!("
                    .set noat
                    .set noreorder
                    LD {tmp}, 0({value})
                    SW {len}, 0({mi})
                    SB {tmp}, 0({ptr})
                ", tmp = out(reg) _, len = in(reg) 0x100 | (length - 1), mi = in(reg) mi_regs, ptr = in(reg) ptr, value = in(reg) &value)
            }
            let unalignstart = start & 3;
            let value = (value as u32) << (24 - (unalignstart * 8));
            let end = start + length.saturating_sub(start&7);
            let unalignend = length & 3;
            for i in 0..buf.count() as u32 * 2 {
                let n = unsafe { buf.as_ptr().cast::<u32>().add(i as usize).read_volatile() };
                let check = if start != end && i >= start/4 && i < (end+3)/4 {
                    let mut check = value;
                    if unalignstart != 0 && i == start/4 {
                        let shift = unalignstart*8;
                        check = (check & (!0 >> shift)) | (!0 << (32-shift));
                    }
                    if unalignend != 0 && i == end/4 {
                        let shift = unalignend*8;
                        check = (check & (!0 << (32-shift))) | (!0 >> shift);
                    }
                    check
                } else {
                    !0
                };
                soft_assert_eq2(n, check, || format!("Repeat for {}..{} at offset {}", start, start+length, i * 4))?;
            }
            Ok(())
        })
    }
}

pub struct SH {}

impl Test for SH {
    fn name(&self) -> &str { "MI Repeat: SH" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_repeat(2, |start, length, value, mi_regs, buf| {
            unsafe {
                let ptr = buf.as_ptr().cast::<u16>().add(start as usize / 2);
                asm!("
                    .set noat
                    .set noreorder
                    LD {tmp}, 0({value})
                    SW {len}, 0({mi})
                    SH {tmp}, 0({ptr})
                ", tmp = out(reg) _, len = in(reg) 0x100 | (length - 1), mi = in(reg) mi_regs, ptr = in(reg) ptr, value = in(reg) &value)
            }
            let unalignstart = start & 3;
            let value = (value as u32) << (16 - (unalignstart * 8));
            let end = start + length.saturating_sub(start&7);
            let unalignend = length & 3;
            for i in 0..buf.count() as u32 * 2 {
                let n = unsafe { buf.as_ptr().cast::<u32>().add(i as usize).read_volatile() };
                let check = if start != end && i >= start/4 && i < (end+3)/4 {
                    let mut check = value;
                    if unalignstart != 0 && i == start/4 {
                        let shift = unalignstart*8;
                        check = (check & (!0 >> shift)) | (!0 << (32-shift));
                    }
                    if unalignend != 0 && i == end/4 {
                        let shift = unalignend*8;
                        check = (check & (!0 << (32-shift))) | (!0 >> shift);
                    }
                    check
                } else {
                    !0
                };
                soft_assert_eq2(n, check, || format!("Repeat for {}..{} at offset {}", start, start+length, i * 4))?;
            }
            Ok(())
        })
    }
}

pub struct SW {}

impl Test for SW {
    fn name(&self) -> &str { "MI Repeat: SW" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_repeat(4, |start, length, value, mi_regs, buf| {
            unsafe {
                let ptr = buf.as_ptr().cast::<u32>().add(start as usize / 4);
                asm!("
                    .set noat
                    .set noreorder
                    LD {tmp}, 0({value})
                    SW {len}, 0({mi})
                    SW {tmp}, 0({ptr})
                ", tmp = out(reg) _, len = in(reg) 0x100 | (length - 1), mi = in(reg) mi_regs, ptr = in(reg) ptr, value = in(reg) &value)
            }
            let value = value as u32;
            let end = start + length.saturating_sub(start&7);
            let unalign = length & 3;
            for i in 0..buf.count() as u32 * 2 {
                let n = unsafe { buf.as_ptr().cast::<u32>().add(i as usize).read_volatile() };
                let check = if i >= start/4 && i < (end+3)/4 {
                    let mut check = value;
                    if unalign != 0 && i == end/4 {
                        let shift = unalign*8;
                        check = (check & (!0 << (32-shift))) | (!0 >> shift);
                    }
                    check
                } else {
                    !0
                };
                soft_assert_eq2(n, check, || format!("Repeat for {}..{} at offset {}", start, start+length, i * 4))?;
            }
            Ok(())
        })
    }
}

pub struct SD {}

impl Test for SD {
    fn name(&self) -> &str { "MI Repeat: SD" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_repeat(8, |start, length, value, mi_regs, buf| {
            unsafe {
                let ptr = buf.as_ptr().add(start as usize / 8);
                asm!("
                    .set noat
                    .set noreorder
                    LD {tmp}, 0({value})
                    SW {len}, 0({mi})
                    SD {tmp}, 0({ptr})
                ", tmp = out(reg) _, len = in(reg) 0x100 | (length - 1), mi = in(reg) mi_regs, ptr = in(reg) ptr, value = in(reg) &value)
            }
            let unalign = length & 7;
            let end = start + length;
            for i in 0..buf.count() as u32 {
                let n = buf.read(i as usize);

                let check = if i >= start/8 && i < (end+7)/8 {
                    let mut check = value;
                    if unalign != 0 && i == end/8 {
                        let shift = unalign*8;
                        check = (check & (!0 << (64-shift))) | (!0 >> shift);
                    }
                    check
                } else {
                    !0
                };
                soft_assert_eq2(n, check, || format!("Repeat for {}..{} at offset {}", start, start+length, i * 8))?;
            }
            Ok(())
        })
    }
}

pub struct Wrap2KiB {}

impl Test for Wrap2KiB {
    fn name(&self) -> &str { "MI Repeat: Wrap2KiB" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut buf = UncachedHeapMemory::<u64>::new_with_align(512, 2048);
        for i in 0..buf.count() {
            buf.write(i, !0);
        }
        let mi_regs = 0xA430_0000u32 as i32;
        let start = 255;
        unsafe {
            let ptr = buf.as_ptr().add(start);
            asm!("
                .set noat
                .set noreorder
                SW {len}, 0({mi})
                SD $0, 0({ptr})
            ", len = in(reg) 0x100 | (128 - 1), mi = in(reg) mi_regs, ptr = in(reg) ptr)
        }
        for i in 0..buf.count() {
            let n = buf.read(i);
            let check = if (i >= start && i < 256) || (i < ((start + 16) & 0xFF)) {
                0
            } else {
                !0
            };
            soft_assert_eq2(n, check, || format!("Repeat at offset {}", i*8))?;
        }

        Ok(())
    }
}
