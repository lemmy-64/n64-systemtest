use core::alloc::Layout;

use linked_list_allocator::LockedHeap;

use crate::memory_map::MemoryMap;
use crate::println;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub fn init_allocator() {
    extern "C" {
        static __bss_end: u8;
    }

    let heap_start = unsafe { &__bss_end as *const u8 as usize };
    let heap_end = 0x8000_0000 + MemoryMap::HEAP_END;

    unsafe {
        let mut guard = ALLOCATOR.lock();
        guard.init(heap_start, heap_end - heap_start);
    }

    // Don't print before the allocator is setup as framebuffer_console needs the allocator
    println!("Heap range: {:x} to {:x}", heap_start, heap_end);
    crate::isviewer::text_out("Allocator D\n");
}

