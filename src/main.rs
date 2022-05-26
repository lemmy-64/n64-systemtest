#![no_std]
#![feature(alloc_error_handler)]
#![feature(asm_const)]
#![feature(asm_experimental_arch)]
#![feature(asm_sym)]
#![feature(naked_functions)]
#![feature(type_name_of_val)]
#![feature(step_trait)]
#![deny(unsafe_op_in_unsafe_fn)]
#![no_main]

extern crate alloc;

use core::arch::global_asm;

use spinning_top::Spinlock;

use crate::graphics::vi::Video;
use crate::memory_map::MemoryMap;

mod allocator;
mod assembler;
mod cop0;
mod enum_str;
mod exception_handler;
mod graphics;
mod isviewer;
mod math;
mod memory_map;
mod mi;
mod panic;
mod pi;
mod print;
mod rsp;
mod tests;

global_asm!(include_str!("boot.s"));

static VIDEO: Spinlock<Video> = Spinlock::new(graphics::vi::Video::new());

#[no_mangle]
unsafe extern "C" fn rust_entrypoint() -> ! {
    MemoryMap::init();
    allocator::init_allocator();
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("n64-systemtest Version {}", VERSION);    main();

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
    graphics::framebuffer_console::INSTANCE.lock().render(v.framebuffers().backbuffer().lock().as_mut().unwrap());
    v.swap_buffers();
}
