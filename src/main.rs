//! Testsuite for a wide variety of N64 features and behaviors.
//! 
//! All tests included in this suite are found in the [`tests`] module.

#![no_std]
#![feature(alloc_error_handler)]
#![feature(asm_const)]
#![feature(asm_experimental_arch)]
#![feature(asm_sym)]
#![feature(naked_functions)]
#![feature(type_name_of_val)]
#![feature(step_trait)]
#![feature(const_option)]
#![feature(const_result_drop)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(rustdoc::private_intra_doc_links)]
#![no_main]
#![deny(unused_must_use)]

extern crate alloc;

use crate::cop1::FCSR;
use spinning_top::Spinlock;
use crate::cop1::set_fcsr;
use crate::graphics::framebuffer_console::FramebufferConsole;

use crate::graphics::vi::Video;
use crate::memory_map::MemoryMap;
use crate::rsp::spmem::SPMEM;

mod allocator;
mod assembler;
mod cop0;
mod cop1;
mod exception_handler;
mod graphics;
mod isviewer;
mod math;
mod memory_map;
mod mi;
mod panic;
mod pi;
mod print;
mod rdp;
mod rsp;
mod tests;
mod uncached_memory;

static VIDEO: Spinlock<Video> = Spinlock::new(Video::new());

#[no_mangle]
unsafe extern "C" fn entrypoint() -> ! {
    // IPL3 (the bootloader) write the memory size to DMEM. We can read it from there
    let memory_size = SPMEM::read(0) as usize;
    let elf_header_offset = ((SPMEM::read(12) >> 16) << 8) as usize;
    MemoryMap::init(memory_size, elf_header_offset);

    // fcsr isn't reset on boot. Use a good default for the main loop - some tests will change and
    // restore this
    set_fcsr(FCSR::new().with_flush_denorm_to_zero(true).with_enable_invalid_operation(true));

    mi::clear_interrupt_mask();
    allocator::init_allocator();
    main();

    loop {}
}

fn main() {
    exception_handler::install_exception_handlers();
    let video_init = VIDEO.lock();
    video_init.init();
    video_init.alloc_framebuffer();
    drop(video_init);
    tests::run();

    let v = VIDEO.lock();
    FramebufferConsole::instance().lock().render(v.framebuffers().backbuffer().lock().as_mut().unwrap());
    v.swap_buffers();
}
