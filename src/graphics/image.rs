use alloc::boxed::Box;

use crate::graphics::color::Color;

pub struct Image<T: Copy + Color> {
    width: u32,
    height: u32,
    padding_right: u32,
    pixels: Box<[T]>,
}

impl<T: Copy + Color> Image<T> {
    pub fn new(width: u32, height: u32, padding_right: u32, pixels: Box<[T]>) -> Self {
        Self {
            width,
            height,
            padding_right,
            pixels
        }
    }

    pub fn clear_with_color(&mut self, color: T) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, color);
            }
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: T) {
        //crate::println!("Setting pixel x={} y={} into array start={:x} length={} to color=0x{:x}", x, y, &self.pixels[0] as *const T as usize, self.pixels.len(), color.as_r8g8b8a8().raw_value());
        self.pixels[(y * self.width + x) as usize] = color;
    }

    pub fn padded_width(&self) -> u32 { self.width - self.padding_right }

    pub fn height(&self) -> u32 { self.height }

    pub fn pixels_mut(&mut self) -> &mut Box<[T]> { &mut self.pixels }
}