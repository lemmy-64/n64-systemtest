use core::arch::asm;
use bitbybit::{bitenum, bitfield};

#[bitenum(u2, exhaustive: true)]
pub enum FCSRRoundingMode {
    Nearest = 0,
    Zero = 1,
    PositiveInfinity = 2,
    NegativeInfinity = 3,
}

#[bitfield(u32, default: 0)]
#[derive(Debug, PartialEq, Eq)]
pub struct FCSR {
    #[bit(24, rw)]
    flush_denorm_to_zero : bool,

    #[bit(23, rw)]
    condition : bool,

    #[bit(17, rw)]
    cause_unimplemented_operation : bool,

    #[bit(16, rw)]
    cause_invalid_operation : bool,
    #[bit(15, rw)]
    cause_division_by_zero : bool,
    #[bit(14, rw)]
    cause_overflow : bool,
    #[bit(13, rw)]
    cause_underflow : bool,
    #[bit(12, rw)]
    cause_inexact_operation : bool,

    #[bit(11, rw)]
    enable_invalid_operation : bool,
    #[bit(10, rw)]
    enable_division_by_zero : bool,
    #[bit(9, rw)]
    enable_overflow : bool,
    #[bit(8, rw)]
    enable_underflow : bool,
    #[bit(7, rw)]
    enable_inexact_operation : bool,

    #[bit(6, rw)]
    invalid_operation : bool,
    #[bit(5, rw)]
    division_by_zero : bool,
    #[bit(4, rw)]
    overflow : bool,
    #[bit(3, rw)]
    underflow : bool,
    #[bit(2, rw)]
    inexact_operation : bool,

    #[bits(0..=1, rw)]
    rounding_mode : FCSRRoundingMode,
}

impl FCSR {
    pub const DEFAULT: Self = Self::new().with_enable_invalid_operation(true).with_flush_denorm_to_zero(true);
}

#[inline(always)]
pub fn cfc1<const INDEX: u8>() -> u32 {
    let result: u32;
    unsafe {
        asm!("
            .set noat
            cfc1 {result}, ${index}
            nop
            nop",
        index = const INDEX,
        result = out(reg) result)
    }
    result
}

#[inline(always)]
pub fn ctc1<const INDEX: u8>(value: u32) {
    unsafe {
        asm!("
            .set noat
            ctc1 {value}, ${index}
            nop
            nop",
        index = const INDEX,
        value = in(reg) value)
    }
}

pub fn set_fcsr(value: FCSR) {
    ctc1::<31>(value.raw_value())
}

pub fn fcsr() -> FCSR {
     FCSR::new_with_raw_value(cfc1::<31>())
}

