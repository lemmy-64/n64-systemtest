use core::fmt::{Debug, Formatter};

/// A signed fixed point that. Terminology:
/// DIGITS_BEFORE: The number of binary digits before the period
/// DIGITS_AFTER: The number of binary digits after the period
/// For example, before=5 and after=2 allows for values 0, 0.25, 0.5, 0.7, 1, .. 31, 31.25, 31.50, 31.75
/// raw_value: Sign extended value. For example -2 is 0xFFFFFFF8, even for a 10_2 value
/// masked_value: This value will have the upper bits at zero. For example, -2 for a 10_2 will be 0xFF8
#[derive(Copy, Clone)]
pub struct SignedFixedPoint<const DIGITS_BEFORE: usize, const DIGITS_AFTER: usize> {
    raw_value: i32,
}

impl<const DIGITS_BEFORE: usize, const DIGITS_AFTER: usize> SignedFixedPoint<DIGITS_BEFORE, DIGITS_AFTER> {
    const DIGITS: usize = DIGITS_BEFORE + DIGITS_AFTER;
    const MASK: u32 = if Self::DIGITS == 32 { 0xFFFF_FFFF } else { (1u32 << Self::DIGITS) - 1 };

    pub const fn raw_value(&self) -> i32 { self.raw_value }

    pub const fn masked_value(&self) -> u32 { (self.raw_value as u32) & Self::MASK }

    pub fn new_with_raw_value(value: i32) -> Self {
        let extra_digits: usize = 32 - Self::DIGITS;
        let sign_extended = ((value as i32) << extra_digits) >> extra_digits;
        assert!(value == sign_extended);
        Self { raw_value: sign_extended }
    }

    pub const fn new_with_masked_value(value: u32) -> Self {
        assert!(value <= Self::MASK);
        let extra_digits: usize = 32 - Self::DIGITS;
        let sign_extended = ((value as i32) << extra_digits) >> extra_digits;
        Self { raw_value: sign_extended }
    }

    pub fn from_i32(value: i32) -> Self {
        let extra_digits: usize = 32 - DIGITS_BEFORE;
        assert!(((value << extra_digits) >> extra_digits) as i32 == value);
        let shifted = value << DIGITS_AFTER;
        Self { raw_value: shifted }
    }

    pub fn as_f32(&self) -> f32 {
        ((self.raw_value as i32) as f32) / ((1 << DIGITS_AFTER) as f32)
    }
}

#[derive(Copy, Clone)]
pub struct UnsignedFixedPoint<const DIGITS_BEFORE: usize, const DIGITS_AFTER: usize> {
    raw_value: u32,
}

impl<const DIGITS_BEFORE: usize, const DIGITS_AFTER: usize> UnsignedFixedPoint<DIGITS_BEFORE, DIGITS_AFTER> {
    const DIGITS: usize = DIGITS_BEFORE + DIGITS_AFTER;
    const MASK: u32 = if Self::DIGITS == 32 { 0xFFFF_FFFF } else { (1u32 << Self::DIGITS) - 1 };

    pub const fn raw_value(&self) -> u32 { self.raw_value }

    pub const fn masked_value(&self) -> u32 { self.raw_value }

    pub const fn new_with_masked_value(value: u32) -> Self {
        assert!(value <= Self::MASK);
        Self { raw_value: value }
    }

    pub const fn from_u32(value: u32) -> Self {
        assert!((value >> DIGITS_BEFORE) == 0);
        Self { raw_value: ((value << DIGITS_AFTER) as u32) }
    }

    pub const fn from_usize(value: usize) -> Self {
        assert!((value >> DIGITS_BEFORE) == 0);
        Self { raw_value: ((value << DIGITS_AFTER) as u32) }
    }

    pub fn as_f32(&self) -> f32 {
        (self.raw_value as f32) / ((1 << DIGITS_AFTER) as f32)
    }
}

impl<const DIGITS_BEFORE: usize, const DIGITS_AFTER: usize> Debug for SignedFixedPoint<DIGITS_BEFORE, DIGITS_AFTER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.as_f32(), f)
    }
}

impl<const DIGITS_BEFORE: usize, const DIGITS_AFTER: usize> Debug for UnsignedFixedPoint<DIGITS_BEFORE, DIGITS_AFTER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.as_f32(), f)
    }
}

pub type U10_2 = UnsignedFixedPoint<10, 2>;
pub type I12_2 = SignedFixedPoint<12, 2>;
pub type I16_16 = SignedFixedPoint<16, 16>;
