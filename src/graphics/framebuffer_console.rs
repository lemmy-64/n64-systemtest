use alloc::string::{String, ToString};

use spinning_top::Spinlock;

use crate::graphics::color::Color;
use crate::graphics::cursor::Cursor;
use crate::graphics::font::Font;
use crate::graphics::image::Image;
use crate::graphics::system_font::FONT_GENEVA_9;
use crate::graphics::vi::PixelType;

static INSTANCE: Spinlock<FramebufferConsole> = Spinlock::new(FramebufferConsole::new());

pub struct FramebufferConsole {
    contents: String,
    is_full: bool,
}

impl<'a> FramebufferConsole {
    /// An arbitrary upper limit on the total string size to avoid OOM. As we don't allow scrolling,
    /// there's no point in supporting more.
    const MAX_CHARS: usize = 10_000;

    const fn new() -> Self {
        Self { contents: String::new(), is_full: false }
    }

    pub fn instance() -> &'static Spinlock<FramebufferConsole> { &INSTANCE }

    /// Prepends the given string, even if the console is considered full
    pub fn prepend(&mut self, str: &str) {
        self.contents = str.to_string() + &self.contents;
    }

    pub fn append(&mut self, str: &str) {
        if !self.is_full {
            self.contents += str;
            self.is_full = self.contents.len() > Self::MAX_CHARS;
        }
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