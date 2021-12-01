use alloc::string::String;

use spinning_top::Spinlock;

use crate::graphics::color::Color;
use crate::graphics::cursor::Cursor;
use crate::graphics::font::Font;
use crate::graphics::image::Image;
use crate::graphics::system_font::FONT_GENEVA_9;
use crate::graphics::vi::PixelType;

pub static INSTANCE: Spinlock<FramebufferConsole> = Spinlock::new(FramebufferConsole::new());

pub struct FramebufferConsole {
    contents: String,
}

impl<'a> FramebufferConsole {
    const fn new() -> Self {
        Self { contents: String::new() }
    }

    pub fn append(&mut self, str: &str) {
        self.contents += str;
    }

    pub fn render(&self, buffer: &mut Image<PixelType>) {
        buffer.clear_with_color(PixelType::WHITE);

        let font = Font::from_data(&FONT_GENEVA_9).unwrap();
        let mut cursor = Cursor::new_with_font(&font, PixelType::BLACK);
        cursor.x = 16;
        cursor.y = 16;
        for line in self.contents.lines() {
            cursor.x = 16;
            cursor.draw_text(buffer, line);
            cursor.draw_text(buffer, "\n");
        }
    }
}