pub mod color;
pub mod cursor;
pub mod font;
pub mod framebuffer_console;
pub mod framebuffer_images;
pub mod image;
pub mod system_font;
pub mod vi;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Offset2D {
    pub x: u32,
    pub y: u32,
}
