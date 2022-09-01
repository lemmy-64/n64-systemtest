use alloc::format;
use core::arch::asm;
use core::ops::{Deref, DerefMut};

use spinning_top::Spinlock;

use crate::cop0::{cause_extract_delay, cause_extract_exception, CauseException};
use crate::cop1::FCSR;
use crate::graphics::color::Color;
use crate::graphics::cursor::Cursor;
use crate::graphics::font::Font;
use crate::graphics::system_font::FONT_GENEVA_9;
use crate::graphics::vi::PixelType;
use crate::VIDEO;

use super::cop0;

static EXCEPTION_SKIP: Spinlock<Option<u64>> = Spinlock::new(None);

/// Once an exception is seen, this is set to the exception context, along with 1. If another
/// exception comes in, the counter is incremented and its Context is lost)
static SEEN_EXCEPTION: Spinlock<Option<(Context, u32)>> = Spinlock::new(None);

// TODO: named labels (like exception_handler_000_start) raise a warning. Our usage should be fine,
// as we only use them to calculate a delta and as we only use it within [naked] functions.
// However, we should rewrite the code so that we don't have to disable the warning named_asm_labels
// Switching to global_asm would be an option here

#[allow(named_asm_labels)]
// This code will be copied to 0x80000000.
#[naked]
extern "C" fn exception_handler_000() {
    unsafe {
        asm!("
            .set noat
            .set noreorder
            exception_handler_000_start:
            la $26, 0x80000000
            j {exception_handler_generic}
            nop // delay slot
            exception_handler_000_size = . - exception_handler_000_start
            .global exception_handler_000_size
   ", exception_handler_generic = sym exception_handler_generic, options(noreturn));
    }
}

#[allow(named_asm_labels)]
// This code will be copied to 0x80000080.
#[naked]
extern "C" fn exception_handler_080() {
    unsafe {
        asm!("
            .set noat
            .set noreorder
            exception_handler_080_start:
            la $26, 0x80000080
            j {exception_handler_generic}
            nop // delay slot
            exception_handler_080_size = . - exception_handler_080_start
            .global exception_handler_080_size
   ", exception_handler_generic = sym exception_handler_generic, options(noreturn));
    }
}

#[allow(named_asm_labels)]
// This code will be copied to 0x80000180.
#[naked]
extern "C" fn exception_handler_180() {
    unsafe {
        asm!("
            .set noat
            .set noreorder
            exception_handler_180_start:
            la $26, 0x80000180
            j {exception_handler_generic}
            nop // delay slot
            exception_handler_180_size = . - exception_handler_180_start
            .global exception_handler_180_size
   ", exception_handler_generic = sym exception_handler_generic, options(noreturn));
    }
}

// The other three exception handlers just here.
#[naked]
extern "C" fn exception_handler_generic() {
    unsafe {
        // Save GPR except: R0 (=0), R29 (=stack pointer)
        asm!("
            .set noat
            .set noreorder
            addi $sp, $sp, -{CONTEXT_SIZE}
            sd $1, 0 ($sp)
            sd $2, 8 ($sp)
            sd $3, 16 ($sp)
            sd $4, 24 ($sp)
            sd $5, 32 ($sp)
            sd $6, 40 ($sp)
            sd $7, 48 ($sp)
            sd $8, 56 ($sp)
            sd $9, 64 ($sp)
            sd $10, 72 ($sp)
            sd $11, 80 ($sp)
            sd $12, 88 ($sp)
            sd $13, 96 ($sp)
            sd $14, 104 ($sp)
            sd $15, 112 ($sp)
            sd $16, 120 ($sp)
            sd $17, 128 ($sp)
            sd $18, 136 ($sp)
            sd $19, 144 ($sp)
            sd $20, 152 ($sp)
            sd $21, 160 ($sp)
            sd $22, 168 ($sp)
            sd $23, 176 ($sp)
            sd $24, 184 ($sp)
            sd $25, 192 ($sp)
            sd $26, 200 ($sp)
            sd $27, 208 ($sp)
            sd $28, 216 ($sp)
            sd $30, 224 ($sp)
            sd $31, 232 ($sp)
            mflo $1
            mfhi $2
            sd $1, 240 ($sp)
            sd $2, 248 ($sp)

            // Exception information
            dmfc0 $1, ${ExceptPCRegisterIndex}
            dmfc0 $2, ${ErrorEPCRegisterIndex}
            dmfc0 $3, ${ContextRegisterIndex}
            dmfc0 $4, ${XContextRegisterIndex}
            dmfc0 $5, ${BadVAddrRegisterIndex}
            dmfc0 $6, ${EntryLo0RegisterIndex}
            dmfc0 $7, ${EntryLo1RegisterIndex}
            dmfc0 $8, ${EntryHiRegisterIndex}
            mfc0 $9, ${CauseRegisterIndex}
            mfc0 $10, ${StatusRegisterIndex}

            // Make sure COP1 is usable so that we can get FCSR
            lui $11, 0x2000
            or $11, $11, $10
            mtc0 $11, ${StatusRegisterIndex}
            nop
            nop
            cfc1 $11, $31          # COP1 FCSR
            sd $1, 256 ($sp)
            sd $2, 264 ($sp)
            sd $3, 272 ($sp)
            sd $4, 280 ($sp)
            sd $5, 288 ($sp)
            sd $6, 296 ($sp)
            sd $7, 304 ($sp)
            sd $8, 312 ($sp)
            sd $0, 320 ($sp) // return_to address
            sw $9, 328 ($sp)
            sw $10, 332 ($sp)
            sw $11, 336 ($sp)

            // Call into compiled code. Pass stackpointer as argument
            la $at, {CompiledFunction}
            jalr $at
            ori $a0, $sp, 0x0  // delay slot

            // Use new stackpointer as returned, then restore from stack
            ori $sp, $v0, 0x0

            ld $2, 320 ($sp)
            lw $10, 332 ($sp)
            // TODO: In some cases, we'll need to return from ErrorEPC. When is that? (Nested exceptions?)
            dmtc0 $2, ${ExceptPCRegisterIndex}
            mtc0 $10, ${StatusRegisterIndex}

            ld $2, 248 ($sp)
            ld $1, 240 ($sp)
            mthi $2
            mtlo $1
            ld $31, 232 ($sp)
            ld $30, 224 ($sp)
            ld $28, 216 ($sp)
            lui $27, 0x0000  // don't restore this, zero it out
            lui $26, 0x0000  // don't restore this, zero it out
            ld $25, 192 ($sp)
            ld $24, 184 ($sp)
            ld $23, 176 ($sp)
            ld $22, 168 ($sp)
            ld $21, 160 ($sp)
            ld $20, 152 ($sp)
            ld $19, 144 ($sp)
            ld $18, 136 ($sp)
            ld $17, 128 ($sp)
            ld $16, 120 ($sp)
            ld $15, 112 ($sp)
            ld $14, 104 ($sp)
            ld $13, 96 ($sp)
            ld $12, 88 ($sp)
            ld $11, 80 ($sp)
            ld $10, 72 ($sp)
            ld $9, 64 ($sp)
            ld $8, 56 ($sp)
            ld $7, 48 ($sp)
            ld $6, 40 ($sp)
            ld $5, 32 ($sp)
            ld $4, 24 ($sp)
            ld $3, 16 ($sp)
            ld $2, 8 ($sp)
            ld $1, 0 ($sp)
            addi $sp, $sp, {CONTEXT_SIZE}
            eret
   ", CompiledFunction = sym exception_handler_compiled,
        ExceptPCRegisterIndex = const cop0::RegisterIndex::ExceptPC as u32,
        ErrorEPCRegisterIndex = const cop0::RegisterIndex::ErrorEPC as u32,
        ContextRegisterIndex = const cop0::RegisterIndex::Context as u32,
        XContextRegisterIndex = const cop0::RegisterIndex::XContext as u32,
        BadVAddrRegisterIndex = const cop0::RegisterIndex::BadVAddr as u32,
        EntryLo0RegisterIndex = const cop0::RegisterIndex::EntryLo0 as u32,
        EntryLo1RegisterIndex = const cop0::RegisterIndex::EntryLo1 as u32,
        EntryHiRegisterIndex = const cop0::RegisterIndex::EntryHi as u32,
        CauseRegisterIndex = const cop0::RegisterIndex::Cause as u32,
        StatusRegisterIndex = const cop0::RegisterIndex::Status as u32,
        CONTEXT_SIZE = const Context::SIZE, options(noreturn));
    }
}

extern "C" fn exception_handler_compiled(stackpointer: usize) -> usize {
    let avoid_bluescreen = true;

    let context = unsafe { &mut *(stackpointer as *mut Context) };

    let mut guard = SEEN_EXCEPTION.lock();
    let skip_guard = EXCEPTION_SKIP.lock();
    if guard.is_none() || avoid_bluescreen {
        // Skip the offending instruction(s) and return
        if skip_guard.is_some() {
            context.return_to = context.exceptpc + skip_guard.unwrap() * 4;
        } else {
            crate::isviewer::text_out("Got unhandled exception. Attempting to continue\n");
            context.return_to = context.exceptpc + (if cause_extract_delay(context.cause) { 8 } else { 4 });
        }

        // Save the exception context
        if guard.is_none() {
            *guard.deref_mut() = Some(((*context), 1));
        } else {
            guard.deref_mut().as_mut().unwrap().1 += 1;
            crate::println!("Multiple exceptions seen. Trying to recover (turn off avoid_bluescreen if this loops endlessly)")
        }

        // Continue running
        return stackpointer;
    }

    crate::isviewer::text_out("Got an exception but already got an exception previously. Showing bluescreen\n");
    show_bluescreen_of_death(context);
}

pub fn drain_seen_exception() -> Option<(Context, u32)> {
    let mut guard = SEEN_EXCEPTION.lock();
    let result = *guard.deref();
    *guard.deref_mut() = None;
    result
}

pub fn expect_exception<F>(code: CauseException, skip_instructions_on_hit: u64, f: F) -> Result<Context, alloc::string::String>
    where F: FnOnce() -> Result<(), &'static str> {
    let guard = SEEN_EXCEPTION.lock();
    if guard.is_some() {
        return Err(format!("Expected exception {:?} but we already previously got {:?}", code, cause_extract_exception(guard.unwrap().0.cause)));
    }
    drop(guard);

    let mut skip_guard = EXCEPTION_SKIP.lock();
    assert!(skip_guard.is_none());
    *skip_guard = Some(skip_instructions_on_hit);
    drop(skip_guard);

    let result = f();

    let mut skip_guard = EXCEPTION_SKIP.lock();
    assert!(skip_guard.is_some());
    *skip_guard = None;
    drop(skip_guard);

    let seen_exception_and_count = drain_seen_exception();
    match result {
        Ok(_) => {
            match seen_exception_and_count {
                None => {
                    Err(format!("Exception expected but none seen"))
                }
                Some((context, count)) => {
                    let actual_exception = cause_extract_exception(context.cause);
                    if count != 1 {
                        Err(format!("Expected exception {:?} but got {} exceptions, the first of which was {:?}", code, count, actual_exception))
                    } else if actual_exception == Ok(code) {
                        Ok(context)
                    } else {
                        Err(format!("Expected exception {:?} but got {:?}", code, actual_exception))
                    }
                }
            }
        }
        Err(result) => Err(format!("{}", result))
    }
}

pub fn install_handler(source: *mut u8, target: *mut u8, size: usize, capacity: usize) {
    assert!(size <= capacity);

    unsafe {
        source.copy_to(target, size);
    };

    // Fill up with NOPs afterwards. Not really necessary, but this looks a lot cleaner in the debugger
    for i in size..capacity {
        let p = (target as usize + i) as *mut u8;
        unsafe {
            *p = 0;
        }
    }
}

pub fn install_exception_handlers() {
    extern "C" {
        static exception_handler_000_size: u8;
        static exception_handler_080_size: u8;
        static exception_handler_180_size: u8;
    }
    let size_000 = unsafe { &exception_handler_000_size as *const u8 as usize };
    let size_080 = unsafe { &exception_handler_080_size as *const u8 as usize };
    let size_180 = unsafe { &exception_handler_180_size as *const u8 as usize };
    install_handler(exception_handler_000 as *mut u8, 0x8000_0000 as *mut u8, size_000, 0x080);
    install_handler(exception_handler_080 as *mut u8, 0x8000_0080 as *mut u8, size_080, 0x100);
    install_handler(exception_handler_180 as *mut u8, 0x8000_0180 as *mut u8, size_180, 0x180);

    // Invalidate the full 8Kbytes in the Data Cache
    invalidate_data_cache(0x8000_0000 as *const u32, 8 * 1024);

    // Invalidate the full 16Kbytes in the Instruction Cache
    invalidate_instruction_cache(0x8000_0000 as *const u32, 16 * 1024);
}

fn invalidate_instruction_cache(start: *const u32, bytes: usize) {
    assert_eq!(start as usize & 31, 0);
    assert_eq!(bytes & 31, 0);
    for i in (0..bytes).step_by(32) {
        unsafe {
            // 0: Invalidate Instruction Cache
            cop0::cache::<0, 0>((start as usize) + i);
        }
    }

}

fn invalidate_data_cache(start: *const u32, bytes: usize) {
    assert_eq!(start as usize & 15, 0);
    assert_eq!(bytes & 15, 0);
    for i in (0..bytes).step_by(16) {
        unsafe {
            // 1: Index_Write_Back_Invalidate
            cop0::cache::<1, 0>((start as usize) + i);
        }
    }

}

/// Attempts to take over video and show various cop0 registers.
fn show_bluescreen_of_death(context: &Context) -> ! {
    let font = &Font::from_data(&FONT_GENEVA_9).unwrap();
    let mut cursor = Cursor::new_with_font(font, PixelType::WHITE);
    // Not sure why this has to be in a loop, but if we only create a single framebuffer image
    // and leave it, there is some screen corruption
    loop {
        let video = VIDEO.lock();
        let mut backbuffer_lock = video.framebuffers().backbuffer().lock();
        let backbuffer = backbuffer_lock.deref_mut().as_mut().unwrap();
        backbuffer.clear_with_color(PixelType::BLUE);
        cursor.x = 32;
        cursor.y = 32;
        cursor.draw_text(backbuffer, "Crash! Exception vector: ");
        cursor.draw_hex_u32(backbuffer, context.k0_exception_vector as u32);
        cursor.x = 32;
        cursor.y += 16;
        cursor.draw_text(backbuffer, "Cause: ");
        cursor.draw_hex_u32(backbuffer, context.cause);
        cursor.draw_text(backbuffer, " (");
        match cause_extract_exception(context.cause) {
            Ok(exc) => { cursor.draw_text(backbuffer, format!("{:?}", exc).as_str()); }
            Err(code) => {
                cursor.draw_text(backbuffer, "0x");
                cursor.draw_hex_u32(backbuffer, code as u32);
            }
        }
        cursor.draw_text(backbuffer, "), Status: ");
        cursor.draw_hex_u32(backbuffer, context.status);
        cursor.x = 32;
        cursor.y += 16;
        cursor.draw_text(backbuffer, "ExceptPC: ");
        cursor.draw_hex_u64(backbuffer, context.exceptpc);
        cursor.draw_text(backbuffer, ", ErrorEPC: ");
        cursor.draw_hex_u64(backbuffer, context.errorepc);
        cursor.x = 32;
        cursor.y += 16;
        cursor.draw_text(backbuffer, "BadVAddr: ");
        cursor.draw_hex_u64(backbuffer, context.badvaddr);
        cursor.draw_text(backbuffer, ", Context: ");
        cursor.draw_hex_u64(backbuffer, context.context);
        cursor.x = 32;
        cursor.y += 16;
        cursor.draw_text(backbuffer, "XContext: ");
        cursor.draw_hex_u64(backbuffer, context.xcontext);
        cursor.x = 32;
        cursor.y += 16;
        cursor.draw_text(backbuffer, "EntryLo0: ");
        cursor.draw_hex_u64(backbuffer, context.entry_lo0);
        cursor.draw_text(backbuffer, ", EntryLo1: ");
        cursor.draw_hex_u64(backbuffer, context.entry_lo1);
        cursor.x = 32;
        cursor.y += 16;
        cursor.draw_text(backbuffer, "EntryHi: ");
        cursor.draw_hex_u64(backbuffer, context.entry_hi);
        drop(backbuffer_lock);
        video.swap_buffers();
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Context {
    pub at: u64,
    pub v0: u64,
    pub v1: u64,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub a3: u64,
    pub t0: u64,
    pub t1: u64,
    pub t2: u64,
    pub t3: u64,
    pub t4: u64,
    pub t5: u64,
    pub t6: u64,
    pub t7: u64,
    pub s0: u64,
    pub s1: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    pub t8: u64,
    pub t9: u64,
    pub k0_exception_vector: u64,
    pub k1: u64,
    pub gp: u64,
    pub s8: u64,
    pub ra: u64,

    pub lo: u64,
    pub hi: u64,

    pub exceptpc: u64,
    pub errorepc: u64,
    pub context: u64,
    pub xcontext: u64,
    pub badvaddr: u64,
    pub entry_lo0: u64,
    pub entry_lo1: u64,
    pub entry_hi: u64,

    pub return_to: u64,

    pub cause: u32,
    pub status: u32,
    pub fcsr: FCSR,
    padding: u32,  // used to pad to 64 bit - feel free to use going forward
}

impl Context {
    pub const SIZE: usize = 344;
}
