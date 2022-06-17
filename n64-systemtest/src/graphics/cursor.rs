use crate::graphics::color::Color;
use crate::graphics::image::Image;

pub struct Cursor<'a, TColor: Copy + Color> {
    pub font: &'a crate::graphics::font::Font<'a>,
    pub x: u16,
    pub y: u16,
    pub color: TColor,
}

impl<'a, TColor: Copy + Color> Cursor<'a, TColor> {
    pub const fn new_with_font(font: &'a crate::graphics::font::Font<'a>, color: TColor) -> Cursor<'a, TColor> {
        Cursor {
            font,
            x: 0,
            y: 0,
            color,
        }
    }

    pub fn draw_text(&mut self, image: &mut Image<TColor>, s: &str) {
        let cx = self.x;
        for c in s.chars() {
            if c == '\n' {
                self.x = cx;
                self.y += self.font.height;
            } else {
                let maybe_width = self.font.draw_char(image, self.x, self.y, self.color, c);
                let maybe_width_2 = if maybe_width.is_none() {
                    // Reached end of screen. Try again, one line shifted down
                    self.x = cx;
                    self.y += self.font.height;
                    self.font.draw_char(image, self.x, self.y, self.color, c)
                } else {
                    maybe_width
                };
                if maybe_width_2.is_none() {
                    // There's no room to draw anything - give up
                    break;
                }
                self.x += maybe_width_2.unwrap();
            }
        }
    }

    pub fn draw_hex_u32(&mut self, image: &mut Image<TColor>, n: u32) {
        for i in 0..8 {
            let shift = 28 - (i * 4);
            let digit = (n >> shift) & 0xF;
            self.x += self.font.draw_char(image, self.x, self.y, self.color, char::from_digit(digit, 16).unwrap()).unwrap_or(0);
        }
    }

    pub fn draw_hex_u64(&mut self, image: &mut Image<TColor>, n: u64) {
        for i in 0..16 {
            let shift = 60 - (i * 4);
            let digit = ((n >> shift) & 0xF) as u32;
            self.x += self.font.draw_char(image, self.x, self.y, self.color, char::from_digit(digit, 16).unwrap()).unwrap_or(0);
        }
    }
}

