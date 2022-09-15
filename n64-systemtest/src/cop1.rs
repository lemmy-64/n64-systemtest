use alloc::format;
use alloc::string::String;
use core::arch::asm;
use core::fmt::{Debug, Formatter};
use core::mem::transmute;
use bitbybit::{bitenum, bitfield};
use crate::cop1::FCSRRoundingMode::{NegativeInfinity, PositiveInfinity};

pub struct FConst { }

impl FConst {
    // Signalling NAN range (taken from https://www.doc.ic.ac.uk/~eedwards/compsys/float/nan.html).
    pub const SIGNALLING_NAN_START_32: f32 = unsafe { transmute(0x7F800001u32) };
    pub const SIGNALLING_NAN_END_32: f32 = unsafe { transmute(0x7FBFFFFFu32) };
    pub const SIGNALLING_NAN_NEGATIVE_START_32: f32 = unsafe { transmute(0xFF800001u32) };
    pub const SIGNALLING_NAN_NEGATIVE_END_32: f32 = unsafe { transmute(0xFFBFFFFFu32) };

    pub const SIGNALLING_NAN_START_64: f64 = unsafe { transmute(0x7FF0000000000001u64) };
    pub const SIGNALLING_NAN_END_64: f64 = unsafe { transmute(0x7FF7FFFFFFFFFFFFu64) };
    pub const SIGNALLING_NAN_NEGATIVE_START_64: f64 = unsafe { transmute(0xFFF0000000000001u64) };
    pub const SIGNALLING_NAN_NEGATIVE_END_64: f64 = unsafe { transmute(0xFFF7FFFFFFFFFFFFu64) };

    // Quiet NAN range. The COP1 doesn't seem to support those at all
    pub const QUIET_NAN_START_32: f32 = unsafe { transmute(0x7FC00000u32) };
    pub const QUIET_NAN_END_32: f32 = unsafe { transmute(0x7FFFFFFFu32) };
    pub const QUIET_NAN_NEGATIVE_START_32: f32 = unsafe { transmute(0xFFC00000u32) };
    pub const QUIET_NAN_NEGATIVE_END_32: f32 = unsafe { transmute(0xFFFFFFFFu32) };

    pub const QUIET_NAN_START_64: f64 = unsafe { transmute(0x7FF8000000000000u64) };
    pub const QUIET_NAN_END_64: f64 = unsafe { transmute(0x7FFFFFFFFFFFFFFFu64) };
    pub const QUIET_NAN_NEGATIVE_START_64: f64 = unsafe { transmute(0xFFF8000000000000u64) };
    pub const QUIET_NAN_NEGATIVE_END_64: f64 = unsafe { transmute(0xFFFFFFFFFFFFFFFFu64) };

    pub const SUBNORMAL_MIN_POSITIVE_32: f32 = unsafe { transmute::<u32, f32>(0x00000001) };
    pub const SUBNORMAL_MAX_POSITIVE_32: f32 = unsafe { transmute::<u32, f32>(0x007fffff) };
    pub const SUBNORMAL_MIN_NEGATIVE_32: f32 = unsafe { transmute::<u32, f32>(0x80000001) };
    pub const SUBNORMAL_MAX_NEGATIVE_32: f32 = unsafe { transmute::<u32, f32>(0x807fffff) };

    pub const SUBNORMAL_EXAMPLE_32: f32 = unsafe { transmute::<u32, f32>(0x00400000) };
    pub const SUBNORMAL_EXAMPLE_64: f64 = unsafe { transmute::<u64, f64>(0x0008000000000000) };
}

#[bitenum(u2, exhaustive: true)]
#[derive(PartialEq, Eq, Debug)]
pub enum FCSRRoundingMode {
    Nearest = 0,
    Zero = 1,
    PositiveInfinity = 2,
    NegativeInfinity = 3,
}

impl FCSRRoundingMode {
    pub const ALL: [FCSRRoundingMode; 4] = [FCSRRoundingMode::Nearest, FCSRRoundingMode::Zero, PositiveInfinity, NegativeInfinity];
}

#[bitfield(u8, default: 0)]
#[derive(PartialEq, Eq)]
pub struct FCSRFlags {
    #[bit(4, rw)]
    invalid_operation : bool,
    #[bit(3, rw)]
    division_by_zero : bool,
    #[bit(2, rw)]
    overflow : bool,
    #[bit(1, rw)]
    underflow : bool,
    #[bit(0, rw)]
    inexact_operation : bool,
}

impl FCSRFlags {
    pub const ALL: FCSRFlags = FCSRFlags::new().with_invalid_operation(true).with_division_by_zero(true).with_overflow(true).with_underflow(true).with_inexact_operation(true);
    pub const NONE: FCSRFlags = FCSRFlags::new();
    pub const fn invert(&self) -> Self {
        FCSRFlags::new()
            .with_invalid_operation(!self.invalid_operation())
            .with_division_by_zero(!self.division_by_zero())
            .with_overflow(!self.overflow())
            .with_underflow(!self.underflow())
            .with_inexact_operation(!self.inexact_operation())
    }
}

impl Debug for FCSRFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut s = String::new();
        if self.invalid_operation() {
            s += "inv-op "
        }
        if self.division_by_zero() {
            s += "div-by-0 "
        }
        if self.overflow() {
            s += "overflow "
        }
        if self.underflow() {
            s += "underflow "
        }
        if self.inexact_operation() {
            s += "inexact "
        }
        f.write_str(s.trim_end())
    }
}

#[bitfield(u32, default: 0)]
#[derive(PartialEq, Eq)]
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

    pub const fn with_maskable_causes(self, value: FCSRFlags) -> Self {
        self.with_cause_invalid_operation(value.invalid_operation())
            .with_cause_division_by_zero(value.division_by_zero())
            .with_cause_overflow(value.overflow())
            .with_cause_underflow(value.underflow())
            .with_cause_inexact_operation(value.inexact_operation())
    }
    pub const fn maskable_causes(self) -> FCSRFlags {
        FCSRFlags::new()
            .with_invalid_operation(self.cause_invalid_operation())
            .with_division_by_zero(self.cause_division_by_zero())
            .with_overflow(self.cause_overflow())
            .with_underflow(self.cause_underflow())
            .with_inexact_operation(self.cause_inexact_operation())
    }

    pub const fn with_enables(self, value: FCSRFlags) -> Self {
        self.with_enable_invalid_operation(value.invalid_operation())
            .with_enable_division_by_zero(value.division_by_zero())
            .with_enable_overflow(value.overflow())
            .with_enable_underflow(value.underflow())
            .with_enable_inexact_operation(value.inexact_operation())
    }
    pub const fn enables(self) -> FCSRFlags {
        FCSRFlags::new()
            .with_invalid_operation(self.enable_invalid_operation())
            .with_division_by_zero(self.enable_division_by_zero())
            .with_overflow(self.enable_overflow())
            .with_underflow(self.enable_underflow())
            .with_inexact_operation(self.enable_inexact_operation())
    }

    pub const fn with_flags(self, value: FCSRFlags) -> Self {
        self.with_invalid_operation(value.invalid_operation())
            .with_division_by_zero(value.division_by_zero())
            .with_overflow(value.overflow())
            .with_underflow(value.underflow())
            .with_inexact_operation(value.inexact_operation())
    }
    pub const fn flags(self) -> FCSRFlags {
        FCSRFlags::new()
            .with_invalid_operation(self.invalid_operation())
            .with_division_by_zero(self.division_by_zero())
            .with_overflow(self.overflow())
            .with_underflow(self.underflow())
            .with_inexact_operation(self.inexact_operation())
    }
}

impl Debug for FCSR {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let cause_tmp = format!("{} {:?}", if self.cause_unimplemented_operation() { "unimpl " } else { "" }, self.maskable_causes());
        let cause = cause_tmp.trim_end();

        f.debug_struct("FCSR")
            .field("condition", &self.condition())
            .field("rounding_mode", &self.rounding_mode())
            .field("flush_denorm_to_zero", &self.flush_denorm_to_zero())
            .field("flags", &self.flags())
            .field("enables", &self.enables())
            .field("causes", &cause)
            .finish()
    }
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
        result = out(reg) result,
        options(nostack, nomem))
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
        value = in(reg) value,
        options(nostack, nomem))
    }
}

pub fn set_fcsr(value: FCSR) {
    ctc1::<31>(value.raw_value())
}

pub fn fcsr() -> FCSR {
     FCSR::new_with_raw_value(cfc1::<31>())
}

