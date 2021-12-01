pub trait Color {
    const WHITE: Self;
    const BLACK: Self;

    const RED: Self;
    const GREEN: Self;
    const BLUE: Self;
}

/// Widens a 5 bit color channel into an 8 bit.
/// The top 3 bits of value5 must be 0 (this isn't enforced by this private function, but e.g.
/// RGBA1555 does enforce this)
fn widen_5_to_8(value5: u8) -> u8 {
    // What to put into the lowest bits?
    // - 0: Colors will be nicely smoothed out, but we can never reach 0xFF (e.g. full white)
    // - Repeat lowest bit: We can reach white, but colors are unevenly spaced
    // - Dither: Some hardware does that, but it wouldn't be deterministic
    // - Repeat highest 3 bits: Seems to have nice properties: Colors are smooth and we can reach
    //                          full black and full white
    (value5 << 3) | (value5 >> 2)
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct RGBA1555 {
    raw_value: u16,
}

impl RGBA1555 {
    pub const fn new_with_raw_value(value: u16) -> RGBA1555 { RGBA1555 { raw_value: value } }

    pub const fn alpha(&self) -> bool { (self.raw_value & (1u16 << 0usize)) != 0 }
    pub const fn with_alpha(&self, field_value: bool) -> Self {
        Self {
            raw_value: if field_value { self.raw_value | (1u16 << 0usize) } else { self.raw_value & !(1u16 << 0usize) }
        }
    }

    pub const fn blue(&self) -> u8 { ((self.raw_value >> 1usize) & ((1u16 << 5usize) - 1u16)) as u8 }
    pub const fn with_blue(&self, field_value: u8) -> Self {
        Self {
            raw_value: (self.raw_value & !(((1u16 << 5usize) - 1u16) << 1usize)) |
                ((field_value as u16) << 1usize)
        }
    }

    pub const fn green(&self) -> u8 { ((self.raw_value >> 6usize) & ((1u16 << 5usize) - 1u16)) as u8 }
    pub const fn with_green(&self, field_value: u8) -> Self {
        Self {
            raw_value: (self.raw_value & !(((1u16 << 5usize) - 1u16) << 6usize)) |
                ((field_value as u16) << 6usize)
        }
    }

    pub const fn red(&self) -> u8 { ((self.raw_value >> 11usize) & ((1u16 << 5usize) - 1u16)) as u8 }
    pub const fn with_red(&self, field_value: u8) -> Self {
        Self {
            raw_value: (self.raw_value & !(((1u16 << 5usize) - 1u16) << 11usize)) |
                ((field_value as u16) << 11usize)
        }
    }
}

impl RGBA1555 {
    pub const fn new(red: u8, green: u8, blue: u8, alpha: bool) -> Self {
        assert!(red < 32);
        assert!(green < 32);
        assert!(blue < 32);
        Self::new_with_raw_value(0)
            .with_red(red)
            .with_green(green)
            .with_blue(blue)
            .with_alpha(alpha)
    }

    pub const fn from_argb8888(value: ARGB8888) -> Self {
        Self::new(
            value.red() >> 3,
            value.green() >> 3,
            value.blue() >> 3,
            value.alpha() > 127)
    }
}

impl Color for RGBA1555 {
    const WHITE: Self = Self::from_argb8888(ARGB8888::WHITE);
    const BLACK: Self = Self::from_argb8888(ARGB8888::BLACK);

    const RED: Self = Self::from_argb8888(ARGB8888::RED);
    const GREEN: Self = Self::from_argb8888(ARGB8888::GREEN);
    const BLUE: Self = Self::from_argb8888(ARGB8888::BLUE);
}

impl From<ARGB8888> for RGBA1555 {
    fn from(value: ARGB8888) -> Self {
        Self::from_argb8888(value)
    }
}

impl From<RGBA1555> for ARGB8888 {
    fn from(value: RGBA1555) -> Self {
        ARGB8888::new(
            widen_5_to_8(value.red()),
            widen_5_to_8(value.green()),
            widen_5_to_8(value.blue()),
            if value.alpha() { 0xFF } else { 0x00 })
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ARGB8888 {
    raw_value: u32,
}

impl ARGB8888
{
    pub const fn new_with_raw_value(value: u32) -> ARGB8888 { ARGB8888 { raw_value: value } }

    pub const fn blue(&self) -> u8 { ((self.raw_value >> 0usize) & ((1u32 << 8usize) - 1u32)) as u8 }
    pub const fn with_blue(&self, field_value: u8) -> Self {
        Self {
            raw_value:
            (self.raw_value & !(((1u32 << 8usize) - 1u32) << 0usize)) |
                ((field_value as u32) << 0usize)
        }
    }

    pub const fn green(&self) -> u8 { ((self.raw_value >> 8usize) & ((1u32 << 8usize) - 1u32)) as u8 }
    pub const fn with_green(&self, field_value: u8) -> Self {
        Self {
            raw_value:
            (self.raw_value & !(((1u32 << 8usize) - 1u32) << 8usize)) |
                ((field_value as u32) << 8usize)
        }
    }

    pub const fn red(&self) -> u8 { ((self.raw_value >> 16usize) & ((1u32 << 8usize) - 1u32)) as u8 }
    pub const fn with_red(&self, field_value: u8) -> Self
    {
        Self
        {
            raw_value:
            (self.raw_value & !(((1u32 << 8usize) - 1u32) << 16usize)) |
                ((field_value as u32) << 16usize)
        }
    }

    pub const fn alpha(&self) -> u8 { ((self.raw_value >> 24usize) & ((1u32 << 8usize) - 1u32)) as u8 }
    pub const fn with_alpha(&self, field_value: u8) -> Self {
        Self {
            raw_value:
            (self.raw_value & !(((1u32 << 8usize) - 1u32) << 24usize)) |
                ((field_value as u32) << 24usize)
        }
    }
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

impl Color for ARGB8888 {
    const WHITE: Self = Self::new(255, 255, 255, 0);
    const BLACK: Self = Self::new(0, 0, 0, 0);

    const RED: Self = Self::new(255, 0, 0, 0);
    const GREEN: Self = Self::new(0, 255, 0, 0);
    const BLUE: Self = Self::new(0, 0, 255, 0);
}
