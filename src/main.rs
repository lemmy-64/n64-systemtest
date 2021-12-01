#![no_std]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(const_fn_trait_bound)]
#![feature(global_asm)]
#![feature(type_name_of_val)]
#![feature(naked_functions)]
#![no_main]

extern crate alloc;

use spinning_top::Spinlock;

use crate::graphics::vi::Video;
use crate::memory_map::MemoryMap;

mod allocator;
mod assembler;
mod cop0;
mod exception_handler;
mod graphics;
mod isviewer;
mod panic;
mod print;
mod tests;
mod memory_map;
mod enum_str;

global_asm!(include_str!("boot.s"));

static VIDEO: Spinlock<Video> = Spinlock::new(graphics::vi::Video::new());

#[no_mangle]
unsafe extern "C" fn rust_entrypoint() -> ! {
    crate::isviewer::text_out("EntryPoint 0\n");
    MemoryMap::init();
    crate::isviewer::text_out("Memory Map initialized\n");
    allocator::init_allocator();
    crate::isviewer::text_out("Allocator initialized\n");
    println!("Total memory: 0x{:x}", MemoryMap::memory_size());
    main();

    loop {}
}

fn main() {
    // Different consoles/flashcarts have somewhat different status values (e.g. soft reset might be on)
    unsafe { crate::cop0::set_status(0x240000E0); }

    exception_handler::install_exception_handlers();
    let video_init = VIDEO.lock();
    video_init.init();
    video_init.alloc_framebuffer();
    drop(video_init);
    tests::run();

    loop {
        // TODO: Wait for vsync
        let v = VIDEO.lock();
        graphics::framebuffer_console::INSTANCE.lock().render(v.framebuffers().backbuffer().lock().as_mut().unwrap());
        v.swap_buffers();
    }
}
