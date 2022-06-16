use crate::graphics::color::Color;
use crate::graphics::image::Image;

#[derive(Debug)]
pub struct Font<'a> {
    pub height: u16,
    pub baseline: u16,
    bitstream: &'a [u8],
    x_offsets: &'a [u8],
    bytes_per_row: u16,
    max_char: u8,
}

const FIRST_CHAR_IN_FONT: u8 = 32;

impl Font<'_> {
    // Decode and sanify-check font. If we return Some(), then the font is okay
    // and we guarantee that drawing won't panic.
    pub fn from_data(data: &[u8]) -> Option<Font> {
        let baseline = data[0] as u16;
        let bytes_per_row = data[1] as u16 | (data[2] as u16) << 8;
        let height = data[3] as u16;
        let o_x_offsets = data[4] as usize | (data[5] as usize) << 8;
        let o_bitstream = data[6] as usize | (data[7] as usize) << 8;

        if baseline >= height ||            // is baseline within characters?
            o_bitstream < o_x_offsets + 4 || // does x_offsets array contain at least two (16 bit) entries?
            data.len() <= o_bitstream {     // is bitstream non-empty?
            return None;
        }

        let x_offsets = &data[o_x_offsets..o_bitstream];
        let bitstream = &data[o_bitstream..];
        let max_char = ((x_offsets.len() >> 1) + FIRST_CHAR_IN_FONT as usize - 2) as u8;
        let font = Font { height, baseline, bitstream, x_offsets, bytes_per_row, max_char };
        Some(font)
    }

    fn x_offset_width(&self, c: char) -> (u16, u16) {
        let co = ((c as u16 - FIRST_CHAR_IN_FONT as u16) << 1) as usize;
        let x0 = self.x_offsets[co + 0] as u16 | (self.x_offsets[co + 1] as u16) << 8;
        let x1 = self.x_offsets[co + 2] as u16 | (self.x_offsets[co + 3] as u16) << 8;
        (x0, x1 - x0)
    }

    fn is_printable(&self, c: char) -> bool {
        let c = c as u8;
        c >= FIRST_CHAR_IN_FONT && c <= self.max_char
    }

    pub fn draw_char<TColor: Copy + Color>(&self, image: &mut Image<TColor>, cx: u16, cy: u16, color: TColor, c: char) -> Option<u16> {
        if self.is_printable(c) {
            let (x0, width) = self.x_offset_width(c);
            if ((cx + width) as u32) > image.padded_width() {
                return None
            }
            let mut yo = 0;
            if ((cy + self.height) as u32) < image.height() {
                for y in 0..self.height {
                    let mut xx = x0;
                    let yy = (cy - self.height + self.baseline + y) as u32;
                    for x in 0..width {
                        if let Some(byte) = self.bitstream.get((yo + (xx >> 3)) as usize) {
                            if byte << (xx & 7) & 0x80 != 0 {
                                image.set_pixel((cx + x) as u32, yy, color);
                            }
                        }
                        xx += 1;
                    }
                    yo += self.bytes_per_row;
                }
            }
            Some(width)
        } else {
            Some(0)
        }
    }
}
