use alloc::format;
use core::fmt::{Debug, Formatter};
use core::mem::transmute;
use arbitrary_int::{u11, u23, u52};
use bitbybit::bitfield;

#[bitfield(u32)]
struct F32Bits {
    #[bit(31, r)]
    is_negative: bool,

    #[bits(23..=30, r)]
    exponent: u8,

    #[bits(0..=22, r)]
    mantissa: u23,
}

#[bitfield(u64)]
struct F64Bits {
    #[bit(63, r)]
    is_negative: bool,

    #[bits(52..=62, r)]
    exponent: u11,

    #[bits(0..=51, r)]
    mantissa: u52,
}


/// Wrapper for f32 which prints more nicely. This has two benefits:
/// - Special types like sNAN don't print all kinds of exceptions
/// - Broken COP1 implementations are less likely to affect the printing
pub struct SoftF32(F32Bits);

impl SoftF32 {
    pub fn new(f: f32) -> Self {
        Self(F32Bits::new_with_raw_value(unsafe { transmute(f) }))
    }

    pub fn value(&self) -> f32 {
        unsafe { transmute(self.0) }
    }
}

impl Debug for SoftF32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.0.exponent() {
            u8::MIN => if self.0.mantissa().value() == 0 {
                if self.0.is_negative() {
                    f.write_str("-0.0")
                } else {
                    f.write_str("0.0")
                }
            } else {
                f.write_str(format!("denorm (0x{:x})", self.0.raw_value()).as_str())
            },
            u8::MAX =>
                if self.0.mantissa().value() == 0 {
                    if self.0.is_negative() {
                        f.write_str("-inf")
                    } else {
                        f.write_str("inf")
                    }
                } else {
                    if self.0.mantissa().value() >> 22 != 0 {
                        f.write_str(format!("qNAN (0x{:x})", self.0.raw_value()).as_str())
                    } else {
                        f.write_str(format!("sNAN (0x{:x})", self.0.raw_value()).as_str())
                    }
                }
            _ => {
                // This is a regular float. Fallback to the built-in formatting (and hope the COP1 is
                // working well enough to handle it.
                Debug::fmt(&self.value(), f)
            }
        }
    }
}

/// Wrapper for f64 which prints more nicely. This has two benefits:
/// - Special types like sNAN don't print all kinds of exceptions
/// - Broken COP1 implementations are less likely to affect the printing
pub struct SoftF64(F64Bits);

impl SoftF64 {
    pub fn new(f: f64) -> Self {
        Self(F64Bits::new_with_raw_value(unsafe { transmute(f) }))
    }

    pub fn value(&self) -> f64 {
        unsafe { transmute(self.0) }
    }
}

impl Debug for SoftF64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.0.exponent() {
            u11::MIN => if self.0.mantissa().value() == 0 {
                if self.0.is_negative() {
                    f.write_str("-0.0")
                } else {
                    f.write_str("0.0")
                }
            } else {
                f.write_str(format!("denorm (0x{:x})", self.0.raw_value()).as_str())
            },
            u11::MAX =>
                if self.0.mantissa().value() == 0 {
                    if self.0.is_negative() {
                        f.write_str("-inf")
                    } else {
                        f.write_str("inf")
                    }
                } else {
                    if self.0.mantissa().value() >> 51 != 0 {
                        f.write_str(format!("qNAN (0x{:x})", self.0.raw_value()).as_str())
                    } else {
                        f.write_str(format!("sNAN (0x{:x})", self.0.raw_value()).as_str())
                    }
                }
            _ => {
                // This is a regular float. Fallback to the built-in formatting (and hope the COP1 is
                // working well enough to handle it.
                Debug::fmt(&self.value(), f)
            }
        }
    }
}