use alloc::boxed::Box;
use core::alloc::Layout;
use core::slice;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::graphics::color::Color;
use crate::graphics::image::Image;
use spinning_top::Spinlock;

pub struct FramebufferImages<T: Copy + Color> {
    buffer1_is_front: AtomicBool,
    buffer1: Spinlock<Option<Image<T>>>,
    buffer2: Spinlock<Option<Image<T>>>,
}

impl<T: Copy + Color> FramebufferImages<T> {
    pub const fn new() -> Self {
        Self {
            buffer1_is_front: AtomicBool::new(true),
            buffer1: Spinlock::new(None),
            buffer2: Spinlock::new(None),
        }
    }

    pub fn alloc_buffers(&self, framebuffer_alignment: usize, width: u32, height: u32) {
        let pixel_size = core::mem::size_of::<T>();
        let pixel_count = (width * height) as usize;
        let framebuffer_layout = Layout::from_size_align(pixel_count * pixel_size, framebuffer_alignment).unwrap();

        let framebuffer1 = unsafe { Box::from_raw(slice::from_raw_parts_mut(alloc::alloc::alloc_zeroed(framebuffer_layout) as *mut T, pixel_count)) };
        let framebuffer2 = unsafe { Box::from_raw(slice::from_raw_parts_mut(alloc::alloc::alloc_zeroed(framebuffer_layout) as *mut T, pixel_count)) };

        self.use_existing_buffers(width, height, framebuffer1, framebuffer2);
    }

    pub fn use_existing_buffers(&self, width: u32, height: u32, framebuffer1: Box<[T]>, framebuffer2: Box<[T]>) {
        *self.buffer1.lock() = Some(Image::new(width, height, 8, framebuffer1));
        *self.buffer2.lock() = Some(Image::new(width, height, 8, framebuffer2));
    }

    pub fn swap_buffers(&self) {
        self.buffer1_is_front.fetch_xor(true, Ordering::AcqRel);
    }

    pub fn buffer1_is_front(&self) -> bool {
        self.buffer1_is_front.load(Ordering::Acquire)
    }

    fn get_buffer(&self, front: bool) -> &Spinlock<Option<Image<T>>> {
        if front ^ self.buffer1_is_front() { &self.buffer2 } else { &self.buffer1 }
    }

    pub fn frontbuffer(&self) -> &Spinlock<Option<Image<T>>> { self.get_buffer(true) }
    pub fn backbuffer(&self) -> &Spinlock<Option<Image<T>>> { self.get_buffer(false) }
}
