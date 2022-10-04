use core::fmt::{Debug, Formatter};
use arbitrary_int::u5;
use bitbybit::bitfield;

pub trait Color {
    const WHITE: Self;
    const BLACK: Self;

    const RED: Self;
    const GREEN: Self;
    const BLUE: Self;

    fn with_alpha(&self, field_value: u8) -> Self;
}

/// Widens a 5 bit color channel into an 8 bit.
/// The top 3 bits of value5 must be 0 (this isn't enforced by this private function, but e.g.
/// RGBA1555 does enforce this)
fn widen_5_to_8(value5: u5) -> u8 {
    // What to put into the lowest bits?
    // - 0: Colors will be nicely smoothed out, but we can never reach 0xFF (e.g. full white)
    // - Repeat lowest bit: We can reach white, but colors are unevenly spaced
    // - Dither: Some hardware does that, but it wouldn't be deterministic
    // - Repeat highest 3 bits: Seems to have nice properties: Colors are smooth and we can reach
    //                          full black and full white
    (value5.value() << 3) | (value5.value() >> 2)
}

#[bitfield(u16)]
#[derive(PartialEq, Eq)]
pub struct RGBA5551 {
    #[bits(11..=15, rw)]
    red: u5,

    #[bits(6..=10, rw)]
    green: u5,

    #[bits(1..=5, rw)]
    blue: u5,

    #[bit(0, rw)]
    alpha: bool,
}

impl RGBA5551 {
    pub const fn new(red: u5, green: u5, blue: u5, alpha: bool) -> Self {
        Self::new_with_raw_value(0)
            .with_red(red)
            .with_green(green)
            .with_blue(blue)
            .with_alpha(alpha)
    }

    pub const fn from_argb8888(value: ARGB8888) -> Self {
        Self::new(
            u5::extract_u8(value.red(), 3),
            u5::extract_u8(value.green(), 3),
            u5::extract_u8(value.blue(), 3),
            value.alpha() > 127,
        )
    }
}

impl Color for RGBA5551 {
    const WHITE: Self = Self::from_argb8888(ARGB8888::WHITE);
    const BLACK: Self = Self::from_argb8888(ARGB8888::BLACK);

    const RED: Self = Self::from_argb8888(ARGB8888::RED);
    const GREEN: Self = Self::from_argb8888(ARGB8888::GREEN);
    const BLUE: Self = Self::from_argb8888(ARGB8888::BLUE);

    fn with_alpha(&self, field_value: u8) -> Self {
        RGBA5551::with_alpha(self, field_value > 127)
    }
}

impl Debug for RGBA5551 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Simply pass-through to raw_value for the most compact representation
        self.raw_value.fmt(f)
    }
}

impl From<ARGB8888> for RGBA5551 {
    fn from(value: ARGB8888) -> Self {
        Self::from_argb8888(value)
    }
}

impl From<RGBA5551> for ARGB8888 {
    fn from(value: RGBA5551) -> Self {
        ARGB8888::new(
            widen_5_to_8(value.red()),
            widen_5_to_8(value.green()),
            widen_5_to_8(value.blue()),
            if value.alpha() { 0xFF } else { 0x00 })
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub struct ARGB8888 {
    #[bits(24..=31, rw)]
    alpha: u8,

    #[bits(16..=23, rw)]
    red: u8,

    #[bits(8..=15, rw)]
    green: u8,

    #[bits(0..=7, rw)]
    blue: u8,
}

impl ARGB8888 {
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self::new_with_raw_value(0)
            .with_red(red)
            .with_green(green)
            .with_blue(blue)
            .with_alpha(alpha)
    }
}

impl Debug for ARGB8888 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Simply pass-through to raw_value for the most compact representation
        self.raw_value.fmt(f)
    }
}

impl Color for ARGB8888 {
    const WHITE: Self = Self::new(255, 255, 255, 0);
    const BLACK: Self = Self::new(0, 0, 0, 0);

    const RED: Self = Self::new(255, 0, 0, 0);
    const GREEN: Self = Self::new(0, 255, 0, 0);
    const BLUE: Self = Self::new(0, 0, 255, 0);

    fn with_alpha(&self, field_value: u8) -> Self {
        ARGB8888::with_alpha(self, field_value)
    }
}
