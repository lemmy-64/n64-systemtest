pub mod compares;
pub mod full_vs_half_mode;
pub mod randomized;

use alloc::boxed::Box;
use alloc::{format, vec};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use core::mem::transmute;
use arbitrary_int::{u2, u5};
use crate::assembler::{Assembler, FR, GPR};
use crate::cop0::{Cause, CauseException, preset_cause_to_copindex2, set_status, Status};
use crate::cop1::{cfc1, ctc1, fcsr, FCSR, FCSRFlags, FCSRRoundingMode, FConst, set_fcsr};
use crate::exception_handler::expect_exception;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq2, soft_assert_f32_bits, soft_assert_f64_bits};

// Lessons learned:
// - CFC1 for 0 returns a constant. CTC1 to 0 is ignored.
// - CFC1 for 1 returns random garbage that seems to contain the original value of the register as well
//  s no test for those
// - Untested, but we're assuming that 2..=30 behave the same way
// - CFC1 for 31 returns the FCSR. It has a write-mask. If an interrupt is set AND enabled at the same time,
//   it is fired right away

// COP1 exceptions:
// - FPU exception can simply be set via software, by CTC1'ing the bits
// - All FPU operations with the exception of MOV.S and MOV.D clear the cause bits
// - When an FPU exception fires, the NEXT instruction defines what the value of Cause.copindex is
//   (if it's for example a MFC1, it will be 1. For NOP, it will be 0).
// - Underflow happens if flush-denorm is set. If flush-denorm is false OR if either underflow or inexact exceptions are actually enabled,
//   UnimplementedOperationException is fired instead
// - Signalling NANs don't work at all. If either one of the inputs has this value, UnimplementedOperationException is fired
// - Quiet NANs are supported, but they are signalling. Also all NANs (including negative ones) are treated equally

// TODO:
// - Nested exception (this will probably show that cause.unimplemented is also cleared?)
// - Tests for 32 and 64 bit mode separately. Read 64 bit registers after writing them in 64 bit mode

/// Value of the target reg before an operation is executed
const TARGET_REG_DEFAULT_F32: f32 = 12345678f32;
const TARGET_REG_DEFAULT_F64: f64 = 12345678f64;
const TARGET_REG_DEFAULT_I32: i32 = 0x12345678i32;
const TARGET_REG_DEFAULT_I64: i64 = 0x12345678i64;

/// This is the NAN value that the COP1 produces
const COP1_RESULT_NAN_32: f32 = FConst::SIGNALLING_NAN_END_32;
const COP1_RESULT_NAN_64: f64 = FConst::SIGNALLING_NAN_END_64;

// Some shortcuts so avoid the need for generic descriptions below
const fn expected_result<F: Copy>(flags: FCSRFlags, result: F) -> Result<(FCSRFlags, F), ()> { Ok((flags, result)) }
const fn expected_unimplemented_f32() -> Result<(FCSRFlags, f32), ()> { Err(()) }
const fn expected_unimplemented_f64() -> Result<(FCSRFlags, f64), ()> { Err(()) }
const fn expected_unimplemented_i32() -> Result<(FCSRFlags, i32), ()> { Err(()) }
const fn expected_unimplemented_i64() -> Result<(FCSRFlags, i64), ()> { Err(()) }

pub struct CFC1CTC1_0;

impl Test for CFC1CTC1_0 {
    fn name(&self) -> &str { "CFC1 / CTC1 (index 0)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Read initial value
        soft_assert_eq(0xA00, cfc1::<0>(), "CFC1 0")?;

        // Try to overwrite
        ctc1::<0>(0x123);

        // Ensure it wasn't modified
        soft_assert_eq(0xA00, cfc1::<0>(), "CFC1 0")?;
        Ok(())
    }
}

pub struct CFC1CTC1_31;

impl Test for CFC1CTC1_31 {
    fn name(&self) -> &str { "CFC1 / CTC1 FCSR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Read initial value: This is set by the test runner
        soft_assert_eq(FCSR::DEFAULT, fcsr(), "CFC1 31")?;

        // Overwrite and readback
        // When setting the exception bits, don't set a flag and its enable flag at the same time as that actually fires
        // the exception
        for v in [0, 0b1111111_11_11111_0_11111_00000_11111_11, 0b1111111_11_11111_0_00000_11111_00000_11, 0] {
            const MASK: u32 = 0b0000000_11_00000_1_11111_11111_11111_11;
            let value = FCSR::new_with_raw_value(v);
            set_fcsr(value);
            soft_assert_eq(v & MASK, fcsr().raw_value(), format!("CFC1 31 (after writing {:x}). You might need to apply mask 0x{:x}", v, MASK).as_str())?;
        }

        Ok(())
    }
}

pub struct FireExceptionViaCTC1;

impl Test for FireExceptionViaCTC1 {
    fn name(&self) -> &str { "Fire exception through CTC1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        preset_cause_to_copindex2()?;

        let exception_context = expect_exception(CauseException::FPE, 1, || {
            let fcsr = FCSR::new().with_enable_overflow(true).with_cause_overflow(true);
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    CTC1 $2, $31
                    NOP
                ", in("$2") fcsr.raw_value(), options(nostack))
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, Assembler::make_ctc1(GPR::V0, u5::new(31)), "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, Cause::new().with_exception(CauseException::FPE), "Cause")?;
        soft_assert_eq(exception_context.status, Status::DEFAULT.with_exl(true).raw_value(), "Status")?;
        soft_assert_eq(exception_context.fcsr, FCSR::new().with_enable_overflow(true).with_cause_overflow(true), "FCSR")?;
        Ok(())
    }
}

pub struct FireUnimplementedExceptionViaCTC1;

impl Test for FireUnimplementedExceptionViaCTC1 {
    fn name(&self) -> &str { "Fire unimplemented exception through CTC1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        preset_cause_to_copindex2()?;

        let exception_context = expect_exception(CauseException::FPE, 1, || {
            let fcsr = FCSR::new().with_cause_unimplemented_operation(true);
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    CTC1 $2, $31
                    NOP
                ", in("$2") fcsr.raw_value(), options(nostack))
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, Assembler::make_ctc1(GPR::V0, u5::new(31)), "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, Cause::new().with_exception(CauseException::FPE), "Cause")?;
        soft_assert_eq(exception_context.status, Status::DEFAULT.with_exl(true).raw_value(), "Status")?;
        soft_assert_eq(exception_context.fcsr, FCSR::new().with_cause_unimplemented_operation(true), "FCSR")?;
        Ok(())
    }
}

/// This test really requires pipeline emulation, so it is considered TooWeird
pub struct FireExceptionViaCTC1FollowedByMFC1;

impl Test for FireExceptionViaCTC1FollowedByMFC1 {
    fn name(&self) -> &str { "Fire exception through CTC1 (followed by MFC1)" }

    fn level(&self) -> Level { Level::Cycle }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        preset_cause_to_copindex2()?;

        let exception_context = expect_exception(CauseException::FPE, 1, || {
            let fcsr = FCSR::new().with_enable_overflow(true).with_cause_overflow(true);
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    CTC1 $2, $31
                    MFC1 $0, $0
                ", in("$2") fcsr.raw_value(), options(nostack))
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, Assembler::make_ctc1(GPR::V0, u5::new(31)), "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, Cause::new().with_exception(CauseException::FPE).with_coprocessor_error(u2::new(1)), "Cause")?;
        soft_assert_eq(exception_context.status, Status::DEFAULT.with_exl(true).raw_value(), "Status")?;
        soft_assert_eq(exception_context.fcsr, FCSR::new().with_enable_overflow(true).with_cause_overflow(true), "FCSR")?;
        Ok(())
    }
}

/// This test really requires pipeline emulation, so it is considered TooWeird
pub struct FireExceptionViaCTC1FollowedByMFC2;

impl Test for FireExceptionViaCTC1FollowedByMFC2 {
    fn name(&self) -> &str { "Fire exception through CTC1 (followed by MFC2)" }

    fn level(&self) -> Level { Level::Cycle }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        preset_cause_to_copindex2()?;
        unsafe { set_status(Status::DEFAULT.with_cop2usable(true)); }

        let exception_context = expect_exception(CauseException::FPE, 1, || {
            let fcsr = FCSR::new().with_enable_overflow(true).with_cause_overflow(true);
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    CTC1 $2, $31
                    MFC2 $0, $0
                ", in("$2") fcsr.raw_value(), options(nostack))
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, Assembler::make_ctc1(GPR::V0, u5::new(31)), "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, Cause::new().with_exception(CauseException::FPE).with_coprocessor_error(u2::new(2)), "Cause")?;
        soft_assert_eq(exception_context.status, Status::DEFAULT.with_cop2usable(true).with_exl(true).raw_value(), "Status")?;
        soft_assert_eq(exception_context.fcsr, FCSR::new().with_enable_overflow(true).with_cause_overflow(true), "FCSR")?;
        Ok(())
    }
}

pub struct FireExceptionViaCTC1Delay;

impl Test for FireExceptionViaCTC1Delay {
    fn name(&self) -> &str { "Fire exception through CTC1 (delay)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let exception_context = expect_exception(CauseException::FPE, 2, || {
            let fcsr = FCSR::new().with_enable_underflow(true).with_cause_underflow(true);
            unsafe {
                asm!("
                    .set noat
                    .set noreorder
                    BEQL $0, $0, 1f
                    CTC1 $2, $31
                    MFC1 $0, $0 // this is skipped over - we want to see that cause.cop-index will be 0
1:
                    NOP
                ", in("$2") fcsr.raw_value(), options(nostack))
            }

            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32).add(1) }, Assembler::make_ctc1(GPR::V0, u5::new(31)), "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, Cause::new().with_exception(CauseException::FPE).with_branch_delay(true), "Cause")?;
        soft_assert_eq(exception_context.status, Status::DEFAULT.with_exl(true).raw_value(), "Status")?;
        soft_assert_eq(exception_context.fcsr, FCSR::new().with_enable_underflow(true).with_cause_underflow(true), "FCSR")?;
        Ok(())
    }
}

fn asm_block_f32<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: f32, value2: f32) -> f32 {
    let result: f32;
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") value1,
        in("$f2") value2,
        inout("$f4") TARGET_REG_DEFAULT_F32 => result,
        options(nostack, nomem))
    }

    result
}

fn asm_block_f32tof64<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: f32, value2: f32) -> f64 {
    let result: f64;
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") value1,
        in("$f2") value2,
        inout("$f4") TARGET_REG_DEFAULT_F64 => result,
        options(nostack, nomem))
    }

    result
}

fn asm_block_f32toi32<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: f32, value2: f32) -> i32 {
    let mut result: f32 = unsafe { transmute(TARGET_REG_DEFAULT_I32) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") value1,
        in("$f2") value2,
        inout("$f4") result,
        options(nostack, nomem))
    }

    unsafe { transmute(result) }
}

fn asm_block_f32toi64<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: f32, value2: f32) -> i64 {
    let mut result: f64 = unsafe { transmute(TARGET_REG_DEFAULT_I64) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") value1,
        in("$f2") value2,
        inout("$f4") result,
        options(nostack, nomem))
    }

    unsafe { transmute(result) }
}

fn asm_block_f64<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: f64, value2: f64) -> f64 {
    let result: f64;
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") value1,
        in("$f2") value2,
        inout("$f4") TARGET_REG_DEFAULT_F64 => result,
        options(nostack, nomem))
    }

    result
}

fn asm_block_f64tof32<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: f64, value2: f64) -> f32 {
    let result: f32;
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") value1,
        in("$f2") value2,
        inout("$f4") TARGET_REG_DEFAULT_F32 => result,
        options(nostack, nomem))
    }

    result
}

fn asm_block_f64toi32<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: f64, value2: f64) -> i32 {
    let mut result: f32 = unsafe { transmute(TARGET_REG_DEFAULT_I32) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") value1,
        in("$f2") value2,
        inout("$f4") result,
        options(nostack, nomem))
    }

    unsafe { transmute(result) }
}

fn asm_block_f64toi64<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: f64, value2: f64) -> i64 {
    let mut result: f64 = unsafe { transmute(TARGET_REG_DEFAULT_I64) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") value1,
        in("$f2") value2,
        inout("$f4") result,
        options(nostack, nomem))
    }

    unsafe { transmute(result) }
}

fn asm_block_i32tof32<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: i32, value2: i32) -> f32 {
    let result: f32;
    let f1: f32 = unsafe { transmute(value1) };
    let f2: f32 = unsafe { transmute(value2) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") f1,
        in("$f2") f2,
        inout("$f4") TARGET_REG_DEFAULT_F32 => result,
        options(nostack, nomem))
    }

    result
}

fn asm_block_i32toi32<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: i32, value2: i32) -> i32 {
    let mut result: f32 = unsafe { transmute(TARGET_REG_DEFAULT_I32) };
    let f1: f32 = unsafe { transmute(value1) };
    let f2: f32 = unsafe { transmute(value2) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") f1,
        in("$f2") f2,
        inout("$f4") result,
        options(nostack, nomem))
    }

    unsafe { transmute(result) }
}

fn asm_block_i32toi64<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: i32, value2: i32) -> i64 {
    let mut result: f64 = unsafe { transmute(TARGET_REG_DEFAULT_I64) };
    let f1: f32 = unsafe { transmute(value1) };
    let f2: f32 = unsafe { transmute(value2) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") f1,
        in("$f2") f2,
        inout("$f4") result,
        options(nostack, nomem))
    }

    unsafe { transmute(result) }
}

fn asm_block_i32tof64<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: i32, value2: i32) -> f64 {
    let result: f64;
    let f1: f32 = unsafe { transmute(value1) };
    let f2: f32 = unsafe { transmute(value2) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") f1,
        in("$f2") f2,
        inout("$f4") TARGET_REG_DEFAULT_F64 => result,
        options(nostack, nomem))
    }

    result
}

fn asm_block_i64tof32<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: i64, value2: i64) -> f32 {
    let result: f32;
    let f1: f64 = unsafe { transmute(value1) };
    let f2: f64 = unsafe { transmute(value2) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") f1,
        in("$f2") f2,
        inout("$f4") TARGET_REG_DEFAULT_F32 => result,
        options(nostack, nomem))
    }

    result
}

fn asm_block_i64tof64<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: i64, value2: i64) -> f64 {
    let result: f64;
    let f1: f64 = unsafe { transmute(value1) };
    let f2: f64 = unsafe { transmute(value2) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") f1,
        in("$f2") f2,
        inout("$f4") TARGET_REG_DEFAULT_F64 => result,
        options(nostack, nomem))
    }

    result
}

fn asm_block_i64toi32<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: i64, value2: i64) -> i32 {
    let mut result: f32 = unsafe { transmute(TARGET_REG_DEFAULT_I32) };
    let f1: f64 = unsafe { transmute(value1) };
    let f2: f64 = unsafe { transmute(value2) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") f1,
        in("$f2") f2,
        inout("$f4") result,
        options(nostack, nomem))
    }

    unsafe { transmute(result) }
}

fn asm_block_i64toi64<const BRANCH_INSTRUCTION: u32, const INSTRUCTION: u32>(value1: i64, value2: i64) -> i64 {
    let mut result: f64 = unsafe { transmute(TARGET_REG_DEFAULT_I64) };
    let f1: f64 = unsafe { transmute(value1) };
    let f2: f64 = unsafe { transmute(value2) };
    unsafe {
        asm!("
            .set noat
            .set noreorder
            .word {BRANCH_INSTRUCTION}
            .word {INSTRUCTION}
            nop
        ",
        BRANCH_INSTRUCTION = const BRANCH_INSTRUCTION,
        INSTRUCTION = const INSTRUCTION,
        in("$f0") f1,
        in("$f2") f2,
        inout("$f4") result,
        options(nostack, nomem))
    }

    unsafe { transmute(result) }
}

/// Tests the given FP instruction in both regular and delay position and ensures that the result was seen and the right set of exceptions is being fired
fn test_floating_point<FIn: Copy, FOut: Copy, FAsmBlock: Fn(FIn, FIn) -> FOut, FAsmBlockDelay: Fn(FIn, FIn) -> FOut, FAssertEqual: Fn(FOut, FOut, &str) -> Result<(), String>>(
    context: &str,
    f: FAsmBlock,
    f_delay: FAsmBlockDelay,
    zero: FOut,
    default: FOut,
    instruction: u32,
    assert_f_equal: FAssertEqual,
    flush_denorm_to_zero: bool,
    rounding_mode: FCSRRoundingMode,
    expected_clear_cause_bits: bool,
    value1: FIn,
    value2: FIn,
    expected: Result<(FCSRFlags, FOut), ()>) -> Result<(), String> {
    // Run the operation without any exceptions enabled
    if let Ok((expected_flags, expected_result)) = expected {
        // Run once with all exceptions enabled that we don't care about and once with all disabled
        for enabled in [ FCSRFlags::NONE, expected_flags.invert() ] {
            // Run once with all cause bits off and once with all those set that aren't enabled as exceptions
            for causes_set_before in [FCSRFlags::NONE, enabled.invert()] {
                set_fcsr(FCSR::new().with_rounding_mode(rounding_mode).with_flush_denorm_to_zero(flush_denorm_to_zero).with_enables(enabled).with_maskable_causes(causes_set_before));
                let regular_result = f(value1, value2);
                let regular_result_fcsr = fcsr();
                let expected_cause_bits = if expected_clear_cause_bits { expected_flags } else { expected_flags | causes_set_before };
                // Copy cause bits into flag bits for no-exception case
                let expected_fcsr_no_exception = FCSR::new()
                    .with_flush_denorm_to_zero(flush_denorm_to_zero)
                    .with_rounding_mode(rounding_mode)
                    .with_flags(expected_flags)
                    .with_enables(enabled)
                    .with_maskable_causes(expected_cause_bits);
                assert_f_equal(regular_result, expected_result, format!("Result after {}", context).as_str())?;
                soft_assert_eq2(regular_result_fcsr, expected_fcsr_no_exception, || format!("FCSR after {} with exceptions disabled", context))?;
            }
        }

        if expected_flags == FCSRFlags::NONE {
            // No exceptions expected. We are done
            return Ok(())
        }
    }

    // Exception test. Start with figuring out which combinations of enabled-flags to run
    let enables_and_expected_causes = if let Ok((expected_flags, _)) = expected {
        // Underflow is special: While the error can be silently signalled, it can not fire as an exception
        if expected_flags == FCSRFlags::new().with_underflow(true).with_inexact_operation(true) {
            vec! {
                (FCSRFlags::ALL, FCSR::new().with_cause_unimplemented_operation(true)),
                (FCSRFlags::new().with_underflow(true).with_inexact_operation(true), FCSR::new().with_cause_unimplemented_operation(true)),
                (FCSRFlags::new().with_underflow(true), FCSR::new().with_cause_unimplemented_operation(true)),
                (FCSRFlags::new().with_inexact_operation(true), FCSR::new().with_cause_unimplemented_operation(true))
            }
        } else {
            // Try once with only the required exception and once with all enabled. The result should be the same
            vec! {
                (FCSRFlags::ALL, FCSR::new().with_maskable_causes(expected_flags)),
                (expected_flags, FCSR::new().with_maskable_causes(expected_flags)),
            }
        }
    } else {
        // Unimplemented can't be masked, so no matter what we mask we expect the same
        vec! {
            (FCSRFlags::ALL, FCSR::new().with_cause_unimplemented_operation(true)),
            (FCSRFlags::NONE, FCSR::new().with_cause_unimplemented_operation(true)),
        }
    };

    // If we're expecting any exceptions, set things up now
    preset_cause_to_copindex2()?;

    for (enable, expected_cause) in enables_and_expected_causes {
        for is_delay in [false, true] {
            let mut expection_result = zero;
            let exception_context = expect_exception(CauseException::FPE, if is_delay { 2 } else { 1 }, || {
                // Enable all FPU exceptions
                set_fcsr(FCSR::new().with_rounding_mode(rounding_mode).with_flush_denorm_to_zero(flush_denorm_to_zero).with_enables(enable));

                expection_result = if is_delay {
                    f_delay(value1, value2)
                } else {
                    f(value1, value2)
                };

                set_fcsr(FCSR::new());
                Ok(())
            })?;

            soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
            soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
            soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32).add(if is_delay { 1 } else { 0 }) }, instruction, "ExceptPC points to wrong instruction")?;
            soft_assert_eq(exception_context.cause, Cause::new().with_coprocessor_error(u2::new(0)).with_exception(CauseException::FPE).with_branch_delay(is_delay), "Cause")?;
            soft_assert_eq(exception_context.status, 0x24000002, "Status")?;
            let expected_fcsr_with_all_enabled = expected_cause.with_rounding_mode(rounding_mode).with_flush_denorm_to_zero(flush_denorm_to_zero).with_enables(enable);
            soft_assert_eq(exception_context.fcsr, expected_fcsr_with_all_enabled, "FCSR after operation with exceptions enabled")?;
            assert_f_equal(expection_result, default, "Result after operation (with exception)")?;
        }
    }

    Ok(())
}

fn test_floating_point_f32_which_preserves_cause_bits<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: f32, value2: f32, expected: Result<(FCSRFlags, f32), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_f32::<0, INSTRUCTION>,
        asm_block_f32::<BEQ_INSTRUCTION, INSTRUCTION>,
        0f32,
        TARGET_REG_DEFAULT_F32,
        INSTRUCTION,
        |f1, f2, s| soft_assert_f32_bits(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, false, value1, value2, expected)
}

fn test_floating_point_f32<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: f32, value2: f32, expected: Result<(FCSRFlags, f32), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_f32::<0, INSTRUCTION>,
        asm_block_f32::<BEQ_INSTRUCTION, INSTRUCTION>,
        0f32,
        TARGET_REG_DEFAULT_F32,
        INSTRUCTION,
        |f1, f2, s| soft_assert_f32_bits(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_f32tof64<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: f32, value2: f32, expected: Result<(FCSRFlags, f64), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_f32tof64::<0, INSTRUCTION>,
        asm_block_f32tof64::<BEQ_INSTRUCTION, INSTRUCTION>,
        0f64,
        TARGET_REG_DEFAULT_F64,
        INSTRUCTION,
        |f1, f2, s| soft_assert_f64_bits(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_f32toi32<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: f32, value2: f32, expected: Result<(FCSRFlags, i32), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_f32toi32::<0, INSTRUCTION>,
        asm_block_f32toi32::<BEQ_INSTRUCTION, INSTRUCTION>,
        0i32,
        TARGET_REG_DEFAULT_I32,
        INSTRUCTION,
        |f1, f2, s| soft_assert_eq2(f1, f2, || s.to_string()),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_f32toi64<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: f32, value2: f32, expected: Result<(FCSRFlags, i64), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_f32toi64::<0, INSTRUCTION>,
        asm_block_f32toi64::<BEQ_INSTRUCTION, INSTRUCTION>,
        0i64,
        TARGET_REG_DEFAULT_I64,
        INSTRUCTION,
        |f1, f2, s| soft_assert_eq2(f1, f2, || s.to_string()),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_f64_which_preserves_cause_bits<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: f64, value2: f64, expected: Result<(FCSRFlags, f64), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_f64::<0, INSTRUCTION>,
        asm_block_f64::<BEQ_INSTRUCTION, INSTRUCTION>,
        0f64,
        TARGET_REG_DEFAULT_F64,
        INSTRUCTION,
        |f1, f2, s| soft_assert_f64_bits(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, false, value1, value2, expected)
}

fn test_floating_point_f64<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: f64, value2: f64, expected: Result<(FCSRFlags, f64), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_f64::<0, INSTRUCTION>,
        asm_block_f64::<BEQ_INSTRUCTION, INSTRUCTION>,
        0f64,
        TARGET_REG_DEFAULT_F64,
        INSTRUCTION,
        |f1, f2, s| soft_assert_f64_bits(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_f64tof32<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: f64, value2: f64, expected: Result<(FCSRFlags, f32), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_f64tof32::<0, INSTRUCTION>,
        asm_block_f64tof32::<BEQ_INSTRUCTION, INSTRUCTION>,
        0f32,
        TARGET_REG_DEFAULT_F32,
        INSTRUCTION,
        |f1, f2, s| soft_assert_f32_bits(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_f64toi32<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: f64, value2: f64, expected: Result<(FCSRFlags, i32), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_f64toi32::<0, INSTRUCTION>,
        asm_block_f64toi32::<BEQ_INSTRUCTION, INSTRUCTION>,
        0i32,
        TARGET_REG_DEFAULT_I32,
        INSTRUCTION,
        |f1, f2, s| soft_assert_eq(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_f64toi64<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: f64, value2: f64, expected: Result<(FCSRFlags, i64), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_f64toi64::<0, INSTRUCTION>,
        asm_block_f64toi64::<BEQ_INSTRUCTION, INSTRUCTION>,
        0i64,
        TARGET_REG_DEFAULT_I64,
        INSTRUCTION,
        |f1, f2, s| soft_assert_eq(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_i32tof32<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: i32, value2: i32, expected: Result<(FCSRFlags, f32), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_i32tof32::<0, INSTRUCTION>,
        asm_block_i32tof32::<BEQ_INSTRUCTION, INSTRUCTION>,
        0f32,
        TARGET_REG_DEFAULT_F32,
        INSTRUCTION,
        |f1, f2, s| soft_assert_f32_bits(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_i32toi32<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: i32, value2: i32, expected: Result<(FCSRFlags, i32), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_i32toi32::<0, INSTRUCTION>,
        asm_block_i32toi32::<BEQ_INSTRUCTION, INSTRUCTION>,
        0i32,
        TARGET_REG_DEFAULT_I32,
        INSTRUCTION,
        |f1, f2, s| soft_assert_eq(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_i32toi64<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: i32, value2: i32, expected: Result<(FCSRFlags, i64), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_i32toi64::<0, INSTRUCTION>,
        asm_block_i32toi64::<BEQ_INSTRUCTION, INSTRUCTION>,
        0i64,
        TARGET_REG_DEFAULT_I64,
        INSTRUCTION,
        |f1, f2, s| soft_assert_eq(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_i32tof64<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: i32, value2: i32, expected: Result<(FCSRFlags, f64), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_i32tof64::<0, INSTRUCTION>,
        asm_block_i32tof64::<BEQ_INSTRUCTION, INSTRUCTION>,
        0f64,
        TARGET_REG_DEFAULT_F64,
        INSTRUCTION,
        |f1, f2, s| soft_assert_f64_bits(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_i64tof32<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: i64, value2: i64, expected: Result<(FCSRFlags, f32), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_i64tof32::<0, INSTRUCTION>,
        asm_block_i64tof32::<BEQ_INSTRUCTION, INSTRUCTION>,
        0f32,
        TARGET_REG_DEFAULT_F32,
        INSTRUCTION,
        |f1, f2, s| soft_assert_f32_bits(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_i64tof64<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: i64, value2: i64, expected: Result<(FCSRFlags, f64), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_i64tof64::<0, INSTRUCTION>,
        asm_block_i64tof64::<BEQ_INSTRUCTION, INSTRUCTION>,
        0f64,
        TARGET_REG_DEFAULT_F64,
        INSTRUCTION,
        |f1, f2, s| soft_assert_f64_bits(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_i64toi32<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: i64, value2: i64, expected: Result<(FCSRFlags, i32), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_i64toi32::<0, INSTRUCTION>,
        asm_block_i64toi32::<BEQ_INSTRUCTION, INSTRUCTION>,
        0i32,
        TARGET_REG_DEFAULT_I32,
        INSTRUCTION,
        |f1, f2, s| soft_assert_eq(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

fn test_floating_point_i64toi64<const INSTRUCTION: u32>(context: &str, flush_denorm_to_zero: bool, rounding_mode: FCSRRoundingMode, value1: i64, value2: i64, expected: Result<(FCSRFlags, i64), ()>) -> Result<(), String> {
    const BEQ_INSTRUCTION: u32 = Assembler::make_beq(GPR::R0, GPR::R0, 1);

    test_floating_point(
        context,
        asm_block_i64toi64::<0, INSTRUCTION>,
        asm_block_i64toi64::<BEQ_INSTRUCTION, INSTRUCTION>,
        0i64,
        TARGET_REG_DEFAULT_I64,
        INSTRUCTION,
        |f1, f2, s| soft_assert_eq(f1, f2, s),
        flush_denorm_to_zero, rounding_mode, true, value1, value2, expected)
}

pub struct DivS;

impl Test for DivS {
    fn name(&self) -> &str { "COP1: DIV.S" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, 2f32, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, -1f32, expected_result(FCSRFlags::new(), f32::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, -1f32, expected_result(FCSRFlags::new(), -f32::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, -1f32, expected_result(FCSRFlags::new(), f32::MIN))),

            Box::new((false, FCSRRoundingMode::Nearest, 1f32, 10f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 0.1f32))),
            Box::new((false, FCSRRoundingMode::Zero, 1f32, 10f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 0.099999994f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 1f32, 10f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 0.099999994f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 1f32, 10f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 0.1f32))),

            Box::new((false, FCSRRoundingMode::Nearest, -1f32, 10f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -0.1f32))),
            Box::new((false, FCSRRoundingMode::Zero, -1f32, 10f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -0.099999994f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -1f32, 10f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -0.1f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -1f32, 10f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -0.099999994f32))),

            // f32::MIN / 2 will cause unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Zero, f32::MIN_POSITIVE, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f32::MIN_POSITIVE, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f32::MIN_POSITIVE, 2f32, expected_unimplemented_f32())),

            // f32::MIN / 2 with flush-denorm causes underflow/inexact
            Box::new((true, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, 2f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),
            Box::new((true, FCSRRoundingMode::Zero, f32::MIN_POSITIVE, 2f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, f32::MIN_POSITIVE, 2f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, f32::MIN_POSITIVE, 2f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), f32::MIN_POSITIVE))),

            Box::new((true, FCSRRoundingMode::Nearest, -f32::MIN_POSITIVE, 2f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::Zero, -f32::MIN_POSITIVE, 2f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, -f32::MIN_POSITIVE, 2f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -f32::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, -f32::MIN_POSITIVE, 2f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),

            // f32::MIN / 1.0000001 with flush-denorm causes underflow/inexact
            Box::new((true, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, 1.0000001f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),
            Box::new((true, FCSRRoundingMode::Zero, f32::MIN_POSITIVE, 1.0000001f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, f32::MIN_POSITIVE, 1.0000001f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, f32::MIN_POSITIVE, 1.0000001f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), f32::MIN_POSITIVE))),

            Box::new((true, FCSRRoundingMode::Nearest, -f32::MIN_POSITIVE, 1.0000001f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::Zero, -f32::MIN_POSITIVE, 1.0000001f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, -f32::MIN_POSITIVE, 1.0000001f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -f32::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, -f32::MIN_POSITIVE, 1.0000001f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),

            // Any subnormal input causes unimplemented
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, 1f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, 1f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, 1f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, 1f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),

            // 0/0 gives an invalid operation and produces a specific nan result
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, 0f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f32, 0f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // 2/0 fires division by zero or produces an infinite result
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, 0f32, expected_result(FCSRFlags::new().with_division_by_zero(true), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, -2f32, 0f32, expected_result(FCSRFlags::new().with_division_by_zero(true), f32::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, -0f32, expected_result(FCSRFlags::new().with_division_by_zero(true), f32::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, -2f32, -0f32, expected_result(FCSRFlags::new().with_division_by_zero(true), f32::INFINITY))),

            // MAX / 0.5 is overflow (and inexact for some reason). Same for negative
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, 0.5f32, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, 0.5f32, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f32::NEG_INFINITY))),

            // Various calculations involving infinity
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, 0.5f32, expected_result(FCSRFlags::new(), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, -0.5f32, expected_result(FCSRFlags::new(), f32::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, -2f32, expected_result(FCSRFlags::new(), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, 1f32, f32::INFINITY, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, f32::INFINITY, expected_result(FCSRFlags::new(), -0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, f32::INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, f32::NEG_INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, f32::INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, f32::NEG_INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, FConst::QUIET_NAN_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // 2/NAN produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_NEGATIVE_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_NEGATIVE_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_END_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_NEGATIVE_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_f32())),

            // NAN/2 produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_32, 2f32, expected_unimplemented_f32())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_div(FR::F4, FR::F0, FR::F2).s();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, f32, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, value2, expected)) => {
                test_floating_point_f32::<INSTRUCTION>("DIV.S", *flush_denorm_to_zero, *rounding_mode, *value1, *value2, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct DivD;

impl Test for DivD {
    fn name(&self) -> &str { "COP1: DIV.D" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f64, 2f64, expected_result(FCSRFlags::new(), 0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, -1f64, expected_result(FCSRFlags::new(), f64::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, -1f64, expected_result(FCSRFlags::new(), -f64::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, -1f64, expected_result(FCSRFlags::new(), f64::MIN))),

            Box::new((false, FCSRRoundingMode::Nearest, 1f64, 10f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 0.1f64))),
            Box::new((false, FCSRRoundingMode::Zero, 1f64, 10f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 0.09999999999999999f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 1f64, 10f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 0.09999999999999999f64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 1f64, 10f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 0.1f64))),

            Box::new((false, FCSRRoundingMode::Nearest, -1f64, 10f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -0.1f64))),
            Box::new((false, FCSRRoundingMode::Zero, -1f64, 10f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -0.09999999999999999f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -1f64, 10f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -0.1f64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -1f64, 10f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -0.09999999999999999f64))),

            // f64::MIN / 2 will cause unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Zero, f64::MIN_POSITIVE, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f64::MIN_POSITIVE, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f64::MIN_POSITIVE, 2f64, expected_unimplemented_f64())),

            // f64::MIN / 2 with flush-denorm causes underflow/inexact
            Box::new((true, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, 1.01f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f64))),
            Box::new((true, FCSRRoundingMode::Zero, f64::MIN_POSITIVE, 1.01f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f64))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, f64::MIN_POSITIVE, 1.01f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f64))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, f64::MIN_POSITIVE, 1.01f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), f64::MIN_POSITIVE))),

            Box::new((true, FCSRRoundingMode::Nearest, -f64::MIN_POSITIVE, 1.01f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f64))),
            Box::new((true, FCSRRoundingMode::Zero, -f64::MIN_POSITIVE, 1.01f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f64))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, -f64::MIN_POSITIVE, 1.01f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -f64::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, -f64::MIN_POSITIVE, 1.01f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f64))),

            // f64::MIN / 1.0000001 with flush-denorm causes underflow/inexact
            Box::new((true, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, 1.0000001f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f64))),
            Box::new((true, FCSRRoundingMode::Zero, f64::MIN_POSITIVE, 1.0000001f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f64))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, f64::MIN_POSITIVE, 1.0000001f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f64))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, f64::MIN_POSITIVE, 1.0000001f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), f64::MIN_POSITIVE))),

            Box::new((true, FCSRRoundingMode::Nearest, -f64::MIN_POSITIVE, 1.0000001f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f64))),
            Box::new((true, FCSRRoundingMode::Zero, -f64::MIN_POSITIVE, 1.0000001f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f64))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, -f64::MIN_POSITIVE, 1.0000001f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -f64::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, -f64::MIN_POSITIVE, 1.0000001f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f64))),

            // Any subnormal input causes unimplemented
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, 1f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, 1f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, 1f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, 1f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),

            // 0/0 gives an invalid operation and produces a specific nan result
            Box::new((false, FCSRRoundingMode::Nearest, 0f64, 0f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f64, 0f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // 2/0 fires division by zero or produces an infinite result
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, 0f64, expected_result(FCSRFlags::new().with_division_by_zero(true), f64::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, -2f64, 0f64, expected_result(FCSRFlags::new().with_division_by_zero(true), f64::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, -0f64, expected_result(FCSRFlags::new().with_division_by_zero(true), f64::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, -2f64, -0f64, expected_result(FCSRFlags::new().with_division_by_zero(true), f64::INFINITY))),

            // MAX / 0.5 is overflow (and inexact for some reason). Same for negative
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, 0.5f64, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f64::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, 0.5f64, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f64::NEG_INFINITY))),

            // Various calculations involving infinity
            Box::new((false, FCSRRoundingMode::Nearest, f64::INFINITY, 0.5f64, expected_result(FCSRFlags::new(), f64::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::INFINITY, -0.5f64, expected_result(FCSRFlags::new(), f64::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::NEG_INFINITY, -2f64, expected_result(FCSRFlags::new(), f64::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, 1f64, f64::INFINITY, expected_result(FCSRFlags::new(), 0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, f64::INFINITY, expected_result(FCSRFlags::new(), -0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::INFINITY, f64::INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::INFINITY, f64::NEG_INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::NEG_INFINITY, f64::INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::NEG_INFINITY, f64::NEG_INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::INFINITY, FConst::QUIET_NAN_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // 2/NAN produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_NEGATIVE_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_NEGATIVE_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_END_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_NEGATIVE_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_NEGATIVE_END_64, expected_unimplemented_f64())),

            // NAN/2 produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_64, 2f64, expected_unimplemented_f64())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_div(FR::F4, FR::F0, FR::F2).d();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, f64, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, value2, expected)) => {
                test_floating_point_f64::<INSTRUCTION>("DIV.D", *flush_denorm_to_zero, *rounding_mode, *value1, *value2, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct MulS;

impl Test for MulS {
    fn name(&self) -> &str { "COP1: MUL.S" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // Elements: (flush_denorm_to_zero, rounding mode, first number, second number, expected-cause-bits, expected result
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, 2f32, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, 0f32, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f32, 0f32, expected_result(FCSRFlags::new(), -0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -2f32, 0f32, expected_result(FCSRFlags::new(), -0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, f32::MAX, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f32, f32::MAX, expected_result(FCSRFlags::new(), -0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, -1f32, expected_result(FCSRFlags::new(), f32::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, -1f32, expected_result(FCSRFlags::new(), -f32::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, -1f32, expected_result(FCSRFlags::new(), f32::MIN))),

            // Inexact
            Box::new((false, FCSRRoundingMode::Nearest, 0.123456789f32, 10.123456789f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1.2498095f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 0.123456789f32, 10.123456789f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1.2498095f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 0.123456789f32, 10.123456789f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1.2498096f32))),
            Box::new((false, FCSRRoundingMode::Zero, 0.123456789f32, 10.123456789f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1.2498095f32))),

            Box::new((false, FCSRRoundingMode::Nearest, 0.123456789f32, -10.123456789f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1.2498095f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 0.123456789f32, -10.123456789f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1.2498096f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 0.123456789f32, -10.123456789f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1.2498095f32))),
            Box::new((false, FCSRRoundingMode::Zero, 0.123456789f32, -10.123456789f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1.2498095f32))),

            Box::new((false, FCSRRoundingMode::Nearest, 0.456789123f32, 10.123457095f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4.624285f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 0.456789123f32, 10.123457095f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4.6242847f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 0.456789123f32, 10.123457095f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4.624285f32))),
            Box::new((false, FCSRRoundingMode::Zero, 0.456789123f32, 10.123457095f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4.6242847f32))),


            // Inexact with overflow
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, f32::MAX, expected_result(FCSRFlags::new().with_inexact_operation(true).with_overflow(true), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, f32::MAX, expected_result(FCSRFlags::new().with_inexact_operation(true).with_overflow(true), f32::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, f32::MIN, expected_result(FCSRFlags::new().with_inexact_operation(true).with_overflow(true), f32::INFINITY))),

            // Infinity * 0 is NAN. Infinity times anything else is Infinity (while preserving the sign)
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, 1f32, expected_result(FCSRFlags::new(), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, 2f32, expected_result(FCSRFlags::new(), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, 0f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, -1f32, expected_result(FCSRFlags::new(), f32::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, -2f32, expected_result(FCSRFlags::new(), f32::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, -0f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, 1f32, expected_result(FCSRFlags::new(), f32::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, 2f32, expected_result(FCSRFlags::new(), f32::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, 0f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, -1f32, expected_result(FCSRFlags::new(), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, -2f32, expected_result(FCSRFlags::new(), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, -0f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, f32::INFINITY, expected_result(FCSRFlags::new(), f32::NEG_INFINITY))),

            // Underflow with flush-denorm off causes unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, 0.5f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f32::MIN_POSITIVE, 0.5f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f32::MIN_POSITIVE, 0.5f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Zero, f32::MIN_POSITIVE, 0.5f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Zero, f32::MIN_POSITIVE, f32::MIN_POSITIVE, expected_unimplemented_f32())),

            // Underflow with flush-denorm on causes underflow
            Box::new((true, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, 0.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f32))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, f32::MIN_POSITIVE, 0.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), f32::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, f32::MIN_POSITIVE, 0.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f32))),
            Box::new((true, FCSRRoundingMode::Zero, f32::MIN_POSITIVE, 0.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f32))),

            Box::new((true, FCSRRoundingMode::Nearest, -f32::MIN_POSITIVE, 0.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), -0f32))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, -f32::MIN_POSITIVE, 0.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), -0f32))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, -f32::MIN_POSITIVE, 0.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), -f32::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::Zero, -f32::MIN_POSITIVE, 0.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), -0f32))),

            Box::new((true, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f32))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, f32::MIN_POSITIVE, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), f32::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, f32::MIN_POSITIVE, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f32))),
            Box::new((true, FCSRRoundingMode::Zero, f32::MIN_POSITIVE, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f32))),

            // Any subnormal input causes unimplemented
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, 1f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, f32::MAX, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, f32::INFINITY, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, f32::INFINITY, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, f32::INFINITY, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),

            // Experiments with extreme values
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, f32::MIN_POSITIVE, expected_result(FCSRFlags::new(), 3.9999998f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f32::MAX, f32::MIN_POSITIVE, expected_result(FCSRFlags::new(), 3.9999998f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f32::MAX, f32::MIN_POSITIVE, expected_result(FCSRFlags::new(), 3.9999998f32))),
            Box::new((false, FCSRRoundingMode::Zero, f32::MAX, f32::MIN_POSITIVE, expected_result(FCSRFlags::new(), 3.9999998f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f32, 0f32, expected_result(FCSRFlags::new(), -0f32))),

            // 2*NAN produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_NEGATIVE_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_NEGATIVE_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_END_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_NEGATIVE_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_f32())),

            // NAN*2 produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_32, f32::INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_32, 2f32, expected_unimplemented_f32())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mul(FR::F4, FR::F0, FR::F2).s();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, f32, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, value2, expected)) => {
                test_floating_point_f32::<INSTRUCTION>("MUL.S", *flush_denorm_to_zero, *rounding_mode, *value1, *value2, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct MulD;

impl Test for MulD {
    fn name(&self) -> &str { "COP1: MUL.D" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f64, 2f64, expected_result(FCSRFlags::new(), 0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, -1f64, expected_result(FCSRFlags::new(), f64::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, -1f64, expected_result(FCSRFlags::new(), -f64::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, -1f64, expected_result(FCSRFlags::new(), f64::MIN))),

            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, f64::MAX, expected_result(FCSRFlags::new().with_inexact_operation(true).with_overflow(true), f64::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, f64::MAX, expected_result(FCSRFlags::new().with_inexact_operation(true).with_overflow(true), f64::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, f64::MIN, expected_result(FCSRFlags::new().with_inexact_operation(true).with_overflow(true), f64::INFINITY))),

            // Underflow with flush-denorm off causes unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, 0.5f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f64::MIN_POSITIVE, 0.5f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f64::MIN_POSITIVE, 0.5f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Zero, f64::MIN_POSITIVE, 0.5f64, expected_unimplemented_f64())),

            // Underflow with flush-denorm on causes underflow
            Box::new((true, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, 0.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f64))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, f64::MIN_POSITIVE, 0.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), f64::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, f64::MIN_POSITIVE, 0.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f64))),
            Box::new((true, FCSRRoundingMode::Zero, f64::MIN_POSITIVE, 0.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f64))),

            Box::new((true, FCSRRoundingMode::Nearest, -f64::MIN_POSITIVE, 0.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), -0f64))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, -f64::MIN_POSITIVE, 0.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), -0f64))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, -f64::MIN_POSITIVE, 0.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), -f64::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::Zero, -f64::MIN_POSITIVE, 0.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), -0f64))),

            Box::new((true, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f64))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, f64::MIN_POSITIVE, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), f64::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, f64::MIN_POSITIVE, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f64))),
            Box::new((true, FCSRRoundingMode::Zero, f64::MIN_POSITIVE, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true).with_underflow(true), 0f64))),

            // Any subnormal input causes unimplemented
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, 1f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, f64::MAX, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, f64::INFINITY, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),

            // Experiments with extreme values
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, f64::MIN_POSITIVE, expected_result(FCSRFlags::new(), 3.9999999999999996f64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f64::MAX, f64::MIN_POSITIVE, expected_result(FCSRFlags::new(), 3.9999999999999996f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f64::MAX, f64::MIN_POSITIVE, expected_result(FCSRFlags::new(), 3.9999999999999996f64))),
            Box::new((false, FCSRRoundingMode::Zero, f64::MAX, f64::MIN_POSITIVE, expected_result(FCSRFlags::new(), 3.9999999999999996f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f64, 0f64, expected_result(FCSRFlags::new(), -0f64))),

            // 2*NAN produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_NEGATIVE_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_NEGATIVE_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_END_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_NEGATIVE_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_NEGATIVE_END_64, expected_unimplemented_f64())),

            // NAN*2 produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_64, 2f64, expected_unimplemented_f64())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mul(FR::F4, FR::F0, FR::F2).d();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, f64, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, value2, expected)) => {
                test_floating_point_f64::<INSTRUCTION>("MUL.D", *flush_denorm_to_zero, *rounding_mode, *value1, *value2, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct AddS;

impl Test for AddS {
    fn name(&self) -> &str { "COP1: ADD.S" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, 2f32, expected_result(FCSRFlags::new(), 2f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1f32, 5f32, expected_result(FCSRFlags::new(), 6f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, -1f32, expected_result(FCSRFlags::new().with_inexact_operation(true), f32::MIN))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, -1f32, expected_result(FCSRFlags::new().with_inexact_operation(true), f32::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, 1f32, expected_result(FCSRFlags::new().with_inexact_operation(true), f32::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, f32::MAX, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), f32::MAX))),

            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, f32::NEG_INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, f32::INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Tests for rounding mode
            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f32, 0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f32, 0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 1000000000000000f32, 0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000050000000f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 1000000000000000f32, 0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),

            Box::new((false, FCSRRoundingMode::Nearest, -1000000000000000f32, -0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::Zero, -1000000000000000f32, -0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -1000000000000000f32, -0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -1000000000000000f32, -0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000050000000f32))),

            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f32, 33500000f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f32, 33600000f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000050000000f32))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f32, 33500000f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f32, 33600000f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),

            // Overflow
            Box::new((false, FCSRRoundingMode::Nearest, 3e38f32, 8e37f32, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, -3e38f32, -8e37f32, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f32::NEG_INFINITY))),

            // Underflow: If denorm is false => unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, 1.5285104e-37f32, -1.5391543e-37f32, expected_unimplemented_f32())),

            // Underflow: If denorm is true => works if exceptions are off, but unimplemented if they are enabled (wow). Also, the rounding mode matters on underflow
            Box::new((true, FCSRRoundingMode::Nearest, 1.5285104e-37f32, -1.5391543e-37f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::Zero, 1.5285104e-37f32, -1.5391543e-37f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, 1.5285104e-37f32, -1.5391543e-37f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, 1.5285104e-37f32, -1.5391543e-37f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -1.1754944e-38f32))),

            Box::new((true, FCSRRoundingMode::Nearest, -1.5285104e-37f32, 1.5391543e-37f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),
            Box::new((true, FCSRRoundingMode::Zero, -1.5285104e-37f32, 1.5391543e-37f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, -1.5285104e-37f32, 1.5391543e-37f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 1.1754944e-38f32))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, -1.5285104e-37f32, 1.5391543e-37f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),

            // Any subnormal input causes unimplemented
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, 1f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, f32::MAX, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, f32::INFINITY, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),

            // 2+qNAN produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_NEGATIVE_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_NEGATIVE_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_END_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_NEGATIVE_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_f32())),

            // NAN+2 produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_32, 2f32, expected_unimplemented_f32())),

            // Mixing both types of NAN cause unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_32, FConst::QUIET_NAN_START_32, expected_unimplemented_f32())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_add(FR::F4, FR::F0, FR::F2).s();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, f32, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, value2, expected)) => {
                test_floating_point_f32::<INSTRUCTION>("ADD.S", *flush_denorm_to_zero, *rounding_mode, *value1, *value2, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct AddD;

impl Test for AddD {
    fn name(&self) -> &str { "COP1: ADD.D" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f64, 2f64, expected_result(FCSRFlags::new(), 2f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 1f64, 5f64, expected_result(FCSRFlags::new(), 6f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, -1f64, expected_result(FCSRFlags::new().with_inexact_operation(true), f64::MIN))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, -1f64, expected_result(FCSRFlags::new().with_inexact_operation(true), f64::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, 1f64, expected_result(FCSRFlags::new().with_inexact_operation(true), f64::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, f64::MAX, expected_result(FCSRFlags::new(), 0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), f64::MAX))),

            Box::new((false, FCSRRoundingMode::Nearest, f64::INFINITY, f64::NEG_INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::NEG_INFINITY, f64::INFINITY, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Tests for rounding mode
            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f64, 0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f64, 0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 1000000000000000f64, 0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000.1f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 1000000000000000f64, 0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),

            Box::new((false, FCSRRoundingMode::Nearest, -1000000000000000f64, -0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::Zero, -1000000000000000f64, -0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -1000000000000000f64, -0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -1000000000000000f64, -0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000.1f64))),

            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f64, 0.04f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f64, 0.08f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000.1f64))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f64, 0.04f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f64, 0.08f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),

            // Overflow
            Box::new((false, FCSRRoundingMode::Nearest, 1.6e308f64, 8e307f64, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f64::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, -1.6e308f64, -8e307f64, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f64::NEG_INFINITY))),

            // Underflow: If denorm is false => unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, 3.18021e-307f64, -3.1622e-307f64, expected_unimplemented_f64())),

            // Underflow: If denorm is true => works if exceptions are off, but unimplemented if they are enabled (wow). This special case is handled inside of the
            Box::new((true, FCSRRoundingMode::Nearest, 3.18021e-307f64, -3.1622e-307f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f64))),
            Box::new((true, FCSRRoundingMode::Zero, 3.18021e-307f64, -3.1622e-307f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f64))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, 3.18021e-307f64, -3.1622e-307f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f64))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, 3.18021e-307f64, -3.1622e-307f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), f64::MIN_POSITIVE))),

            Box::new((true, FCSRRoundingMode::Nearest, -3.18021e-307f64, 3.1622e-307f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f64))),
            Box::new((true, FCSRRoundingMode::Zero, -3.18021e-307f64, 3.1622e-307f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f64))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, -3.18021e-307f64, 3.1622e-307f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -f64::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, -3.18021e-307f64, 3.1622e-307f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f64))),

            // Any subnormal input causes unimplemented
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, 1f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, f64::MAX, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, f64::INFINITY, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),

            // 2+NAN produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_NEGATIVE_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_NEGATIVE_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_END_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_NEGATIVE_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_NEGATIVE_END_64, expected_unimplemented_f64())),

            // NAN+2 produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_64, 2f64, expected_unimplemented_f64())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_add(FR::F4, FR::F0, FR::F2).d();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, f64, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, value2, expected)) => {
                test_floating_point_f64::<INSTRUCTION>("ADD.D", *flush_denorm_to_zero, *rounding_mode, *value1, *value2, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct SubS;

impl Test for SubS {
    fn name(&self) -> &str { "COP1: SUB.S" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, -2f32, expected_result(FCSRFlags::new(), 2f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1f32, -5f32, expected_result(FCSRFlags::new(), 6f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, 1f32, expected_result(FCSRFlags::new().with_inexact_operation(true), f32::MIN))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, 1f32, expected_result(FCSRFlags::new().with_inexact_operation(true), f32::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, -1f32, expected_result(FCSRFlags::new().with_inexact_operation(true), f32::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, -f32::MAX, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, -f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), f32::MAX))),

            // Tests for rounding mode
            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f32, -0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f32, -0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 1000000000000000f32, -0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000050000000f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 1000000000000000f32, -0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),

            Box::new((false, FCSRRoundingMode::Nearest, -1000000000000000f32, 0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::Zero, -1000000000000000f32, 0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -1000000000000000f32, 0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -1000000000000000f32, 0.00000000000000000005f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000050000000f32))),

            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f32, -33500000f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f32, -33600000f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000050000000f32))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f32, -33500000f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f32, -33600000f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f32))),

            // Overflow
            Box::new((false, FCSRRoundingMode::Nearest, 3e38f32, -8e37f32, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, -3e38f32, 8e37f32, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f32::NEG_INFINITY))),

            // Underflow: If denorm is false => unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, 1.5285104e-37f32, 1.5391543e-37f32, expected_unimplemented_f32())),

            // Underflow: If denorm is true => works if exceptions are off, but unimplemented if they are enabled (wow). This special case is handled inside of the
            Box::new((true, FCSRRoundingMode::Nearest, 1.5285104e-37f32, 1.5391543e-37f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, 1.5285104e-37f32, 1.5391543e-37f32, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -1.1754944e-38f32))),

            // Any subnormal input causes unimplemented
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, 0f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f32, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, 1f32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, f32::MAX, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, f32::INFINITY, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),

            // 2+NAN produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_NEGATIVE_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::QUIET_NAN_NEGATIVE_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_END_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_NEGATIVE_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_f32())),

            // NAN+2 produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_32, 2f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_32, 2f32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_32, 2f32, expected_unimplemented_f32())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sub(FR::F4, FR::F0, FR::F2).s();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, f32, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, value2, expected)) => {
                test_floating_point_f32::<INSTRUCTION>("SUB.S", *flush_denorm_to_zero, *rounding_mode, *value1, *value2, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct SubD;

impl Test for SubD {
    fn name(&self) -> &str { "COP1: SUB.D" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f64, -2f64, expected_result(FCSRFlags::new(), 2f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 1f64, -5f64, expected_result(FCSRFlags::new(), 6f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, 1f64, expected_result(FCSRFlags::new().with_inexact_operation(true), f64::MIN))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, 1f64, expected_result(FCSRFlags::new().with_inexact_operation(true), f64::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, -1f64, expected_result(FCSRFlags::new().with_inexact_operation(true), f64::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, f64::MIN, expected_result(FCSRFlags::new(), 0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), f64::MAX))),

            // Tests for rounding mode
            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f64, -0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f64, -0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 1000000000000000f64, -0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000.1f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 1000000000000000f64, -0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),

            Box::new((false, FCSRRoundingMode::Nearest, -1000000000000000f64, 0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::Zero, -1000000000000000f64, 0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -1000000000000000f64, 0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -1000000000000000f64, 0.00000000000000000005f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -1000000000000000.1f64))),

            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f64, -0.04f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 1000000000000000f64, -0.08f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000.1f64))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f64, -0.04f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),
            Box::new((false, FCSRRoundingMode::Zero, 1000000000000000f64, -0.08f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1000000000000000f64))),

            // Overflow
            Box::new((false, FCSRRoundingMode::Nearest, 1.6e308f64, -8e307f64, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f64::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, -1.6e308f64, 8e307f64, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f64::NEG_INFINITY))),

            // Underflow: If denorm is false => unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, 3.18021e-307f64, 3.1622e-307f64, expected_unimplemented_f64())),

            // Underflow: If denorm is true => works if exceptions are off, but unimplemented if they are enabled (wow). This special case is handled inside of the
            Box::new((true, FCSRRoundingMode::Nearest, 3.18021e-307f64, 3.1622e-307f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f64))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, 3.18021e-307f64, 3.1622e-307f64, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), f64::MIN_POSITIVE))),

            // Any subnormal input causes unimplemented
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, 0f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, 0f64, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, 1f64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, f64::MAX, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, f64::INFINITY, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Zero, FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),

            // 2+NAN produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_NEGATIVE_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::QUIET_NAN_NEGATIVE_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_END_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_NEGATIVE_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, FConst::SIGNALLING_NAN_NEGATIVE_END_64, expected_unimplemented_f64())),

            // NAN+2 produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_64, 2f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_64, 2f64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_64, 2f64, expected_unimplemented_f64())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sub(FR::F4, FR::F0, FR::F2).d();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, f64, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, value2, expected)) => {
                test_floating_point_f64::<INSTRUCTION>("SUB.D", *flush_denorm_to_zero, *rounding_mode, *value1, *value2, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct AbsS;

impl Test for AbsS {
    fn name(&self) -> &str { "COP1: ABS.S" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f32, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1f32, expected_result(FCSRFlags::new(), 1f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -1f32, expected_result(FCSRFlags::new(), 1f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, expected_result(FCSRFlags::new(), f32::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, expected_result(FCSRFlags::new(), f32::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, expected_result(FCSRFlags::new(), f32::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, -f32::MIN_POSITIVE, expected_result(FCSRFlags::new(), f32::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, expected_result(FCSRFlags::new(), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, expected_result(FCSRFlags::new(), f32::INFINITY))),

            // ABS(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_f32())),

            // Subnormal also causes unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f32())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_abs(FR::F4, FR::F0).s();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                test_floating_point_f32::<INSTRUCTION>("ABS.S", *flush_denorm_to_zero, *rounding_mode, *value1, 0f32, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct AbsD;

impl Test for AbsD {
    fn name(&self) -> &str { "COP1: ABS.D" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f64, expected_result(FCSRFlags::new(), 0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f64, expected_result(FCSRFlags::new(), 0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 1f64, expected_result(FCSRFlags::new(), 1f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -1f64, expected_result(FCSRFlags::new(), 1f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, expected_result(FCSRFlags::new(), f64::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, expected_result(FCSRFlags::new(), f64::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, expected_result(FCSRFlags::new(), f64::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new(), f64::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::INFINITY, expected_result(FCSRFlags::new(), f64::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::NEG_INFINITY, expected_result(FCSRFlags::new(), f64::INFINITY))),

            // ABS(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_64, expected_unimplemented_f64())),

            // Subnormal also causes unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_abs(FR::F4, FR::F0).d();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                test_floating_point_f64::<INSTRUCTION>("ABS.D", *flush_denorm_to_zero, *rounding_mode, *value1, 0f64, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct NegS;

impl Test for NegS {
    fn name(&self) -> &str { "COP1: Neg.S" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, expected_result(FCSRFlags::new(), -0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f32, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1f32, expected_result(FCSRFlags::new(), -1f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -1f32, expected_result(FCSRFlags::new(), 1f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, expected_result(FCSRFlags::new(), f32::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, expected_result(FCSRFlags::new(), f32::MIN))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, expected_result(FCSRFlags::new(), -f32::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, -f32::MIN_POSITIVE, expected_result(FCSRFlags::new(), f32::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, expected_result(FCSRFlags::new(), f32::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, expected_result(FCSRFlags::new(), f32::INFINITY))),

            // ABS(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_f32())),

            // Subnormal also causes unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f32())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_neg(FR::F4, FR::F0).s();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                test_floating_point_f32::<INSTRUCTION>("NEG.S", *flush_denorm_to_zero, *rounding_mode, *value1, 0f32, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct NegD;

impl Test for NegD {
    fn name(&self) -> &str { "COP1: NEG.D" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f64, expected_result(FCSRFlags::new(), -0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f64, expected_result(FCSRFlags::new(), 0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 1f64, expected_result(FCSRFlags::new(), -1f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -1f64, expected_result(FCSRFlags::new(), 1f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, expected_result(FCSRFlags::new(), f64::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, expected_result(FCSRFlags::new(), f64::MIN))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, expected_result(FCSRFlags::new(), -f64::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new(), f64::MIN_POSITIVE))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::INFINITY, expected_result(FCSRFlags::new(), f64::NEG_INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::NEG_INFINITY, expected_result(FCSRFlags::new(), f64::INFINITY))),

            // ABS(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_64, expected_unimplemented_f64())),

            // Subnormal also causes unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_neg(FR::F4, FR::F0).d();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                test_floating_point_f64::<INSTRUCTION>("NEG.D", *flush_denorm_to_zero, *rounding_mode, *value1, 0f64, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct MovS;

impl Test for MovS {
    fn name(&self) -> &str { "COP1: MOV.S" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, expected_result(FCSRFlags::new(), f32::MIN))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, expected_result(FCSRFlags::new(), f32::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_32, expected_result(FCSRFlags::new(), FConst::QUIET_NAN_START_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_32, expected_result(FCSRFlags::new(), FConst::SIGNALLING_NAN_START_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_result(FCSRFlags::new(), FConst::SUBNORMAL_MIN_POSITIVE_32))),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_result(FCSRFlags::new(), FConst::SUBNORMAL_MIN_POSITIVE_32))),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mov(FR::F4, FR::F0).s();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                test_floating_point_f32_which_preserves_cause_bits::<INSTRUCTION>("MOV.S", *flush_denorm_to_zero, *rounding_mode, *value1, 0f32, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct MovD;

impl Test for MovD {
    fn name(&self) -> &str { "COP1: MOV.D" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f64, expected_result(FCSRFlags::new(), 0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, expected_result(FCSRFlags::new(), f64::MIN))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, expected_result(FCSRFlags::new(), f64::MAX))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_64, expected_result(FCSRFlags::new(), FConst::QUIET_NAN_START_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_64, expected_result(FCSRFlags::new(), FConst::SIGNALLING_NAN_START_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_result(FCSRFlags::new(), FConst::SUBNORMAL_MIN_POSITIVE_64))),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_result(FCSRFlags::new(), FConst::SUBNORMAL_MIN_POSITIVE_64))),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mov(FR::F4, FR::F0).d();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                test_floating_point_f64_which_preserves_cause_bits::<INSTRUCTION>("MOV.D", *flush_denorm_to_zero, *rounding_mode, *value1, 0f64, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct SqrtS;

impl Test for SqrtS {
    fn name(&self) -> &str { "COP1: SQRT.S" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f32, expected_result(FCSRFlags::new(), 0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f32, expected_result(FCSRFlags::new(), -0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f32, expected_result(FCSRFlags::new().with_inexact_operation(true), core::f32::consts::SQRT_2))),
            Box::new((false, FCSRRoundingMode::Nearest, 4f32, expected_result(FCSRFlags::new(), 2f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -4f32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MAX, expected_result(FCSRFlags::new().with_inexact_operation(true), 1.8446743e19f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f32::MAX, expected_result(FCSRFlags::new().with_inexact_operation(true), 1.8446744e19f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f32::MAX, expected_result(FCSRFlags::new().with_inexact_operation(true), 1.8446743e19f32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, expected_result(FCSRFlags::new(), 1.0842022e-19f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, expected_result(FCSRFlags::new(), f32::INFINITY))),

            // This value shows that inexact can happen even if sqrt*sqrt==original
            Box::new((false, FCSRRoundingMode::Nearest, 0.0000000000000000000000106731965f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 3.2669859e-12f32))),

            // Sqrt(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_f32())),

            // Subnormal also causes unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f32())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sqrt(FR::F4, FR::F0).s();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                test_floating_point_f32::<INSTRUCTION>("SQRT.S", *flush_denorm_to_zero, *rounding_mode, *value1, 0f32, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct SqrtD;

impl Test for SqrtD {
    fn name(&self) -> &str { "COP1: SQRT.D" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, FCSRRoundingMode::Nearest, 0f64, expected_result(FCSRFlags::new(), 0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f64, expected_result(FCSRFlags::new(), -0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 2f64, expected_result(FCSRFlags::new().with_inexact_operation(true), core::f64::consts::SQRT_2))),
            Box::new((false, FCSRRoundingMode::Nearest, 4f64, expected_result(FCSRFlags::new(), 2f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -4f64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, expected_result(FCSRFlags::new().with_inexact_operation(true), 1.3407807929942596e154f64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f64::MAX, expected_result(FCSRFlags::new().with_inexact_operation(true), 1.3407807929942597e154f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f64::MAX, expected_result(FCSRFlags::new().with_inexact_operation(true), 1.3407807929942596e154f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, expected_result(FCSRFlags::new(), 1.4916681462400413e-154f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::INFINITY, expected_result(FCSRFlags::new(), f64::INFINITY))),

            // This value shows that inexact can happen even if sqrt*sqrt==original
            Box::new((false, FCSRRoundingMode::Nearest, 3.890549325378585e109f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 6.23742681350137e54f64))),

            // Sqrt(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_64, expected_unimplemented_f64())),

            // Subnormal also causes unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f64())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sqrt(FR::F4, FR::F0).d();
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                test_floating_point_f64::<INSTRUCTION>("SQRT.D", *flush_denorm_to_zero, *rounding_mode, *value1, 0f64, *expected)?
            }
            _ => return Err("Unhandled match pattern".to_string())
        }
        Ok(())
    }
}

pub struct CvtS;

impl Test for CvtS {
    fn name(&self) -> &str { "COP1: CVT.S.fmt" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // S ==> S (which doesn't exist)
            Box::new((false, FCSRRoundingMode::Nearest, 4f32, expected_unimplemented_f32())),
            //
            // // D => S
            Box::new((false, FCSRRoundingMode::Nearest, 4f64, expected_result(FCSRFlags::new(), 4f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f64, expected_result(FCSRFlags::new(), -0f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 4.123456789123456f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4.123457f32))),
            Box::new((false, FCSRRoundingMode::Zero, 4.123456789123456f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4.1234565f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 4.123456789123456f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4.1234565f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 4.123456789123456f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4.123457f32))),

            Box::new((false, FCSRRoundingMode::Nearest, -4.123456789123456f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -4.123457f32))),
            Box::new((false, FCSRRoundingMode::Zero, -4.123456789123456f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -4.1234565f32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -4.123456789123456f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -4.123457f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -4.123456789123456f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -4.1234565f32))),

            Box::new((false, FCSRRoundingMode::Nearest, f64::INFINITY, expected_result(FCSRFlags::new(), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f64::NEG_INFINITY, expected_result(FCSRFlags::new(), f32::NEG_INFINITY))),

            // Overflow
            Box::new((false, FCSRRoundingMode::Nearest, f64::MAX, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f32::INFINITY))),

            // If we're juuuust above the f32 limit, the rounding mode determines overflow or not
            Box::new((false, FCSRRoundingMode::Nearest, 3.40282348e+38_f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 3.4028235e+38_f32))),
            Box::new((false, FCSRRoundingMode::Zero, 3.40282348e+38_f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 3.4028235e+38_f32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 3.40282348e+38_f64, expected_result(FCSRFlags::new().with_overflow(true).with_inexact_operation(true), f32::INFINITY))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 3.40282348e+38_f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 3.4028235e+38_f32))),

            // Underflow
            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),
            Box::new((true, FCSRRoundingMode::Zero, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), f32::MIN_POSITIVE))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), 0f32))),

            Box::new((false, FCSRRoundingMode::Nearest, -f64::MIN_POSITIVE, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::Zero, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::PositiveInfinity, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -0f32))),
            Box::new((true, FCSRRoundingMode::NegativeInfinity, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_underflow(true).with_inexact_operation(true), -f32::MIN_POSITIVE))),

            // CVT(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_64, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_32))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_64, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_64, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_64, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_64, expected_unimplemented_f32())),

            // Subnormal also causes unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_f32())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_f32())),

            // W => S
            Box::new((false, FCSRRoundingMode::Nearest, 9i32, expected_result(FCSRFlags::new(), 9f32))),
            Box::new((false, FCSRRoundingMode::Zero, 1234567891i32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1234567800f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1234567891i32, expected_result(FCSRFlags::new().with_inexact_operation(true), 1234568000f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -1234567891i32, expected_result(FCSRFlags::new().with_inexact_operation(true), -1234568000f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0x7FFFFFFDi32, expected_result(FCSRFlags::new().with_inexact_operation(true), 2147483600f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0x7FFFFFFEi32, expected_result(FCSRFlags::new().with_inexact_operation(true), 2147483600f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0x7FFFFFFFi32, expected_result(FCSRFlags::new().with_inexact_operation(true), 2147483600f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0x80000002u32 as i32, expected_result(FCSRFlags::new().with_inexact_operation(true), -2147483600f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0x80000001u32 as i32, expected_result(FCSRFlags::new().with_inexact_operation(true), -2147483600f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0x80000000u32 as i32, expected_result(FCSRFlags::new(), -2147483600f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0xFFFFFFFDu32 as i32, expected_result(FCSRFlags::new(), -3f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0xFFFFFFFEu32 as i32, expected_result(FCSRFlags::new(), -2f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 0xFFFFFFFFu32 as i32, expected_result(FCSRFlags::new(), -1f32))),


            // L => S
            Box::new((false, FCSRRoundingMode::Nearest, 9i64, expected_result(FCSRFlags::new(), 9f32))),
            Box::new((false, FCSRRoundingMode::Zero, 1234567891i64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1234567800f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1234567891i64, expected_result(FCSRFlags::new().with_inexact_operation(true), 1234568000f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -1234567891i64, expected_result(FCSRFlags::new().with_inexact_operation(true), -1234568000f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 53, expected_result(FCSRFlags::new(), 9007199000000000f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 54, expected_result(FCSRFlags::new(), 1.8014399e16f32))),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 55, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 56, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 61, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 62, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 63, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, -(1i64 << 55), expected_result(FCSRFlags::new(), -3.6028797e16f32))),
            Box::new((false, FCSRRoundingMode::Nearest, -(1i64 << 55) - 1, expected_unimplemented_f32())),
            Box::new((false, FCSRRoundingMode::Nearest, i64::MAX, expected_unimplemented_f32())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                const INSTRUCTION: u32 = Assembler::make_cvt_s(FR::F4, FR::F0).s();
                test_floating_point_f32::<INSTRUCTION>("CVT.S.S", *flush_denorm_to_zero, *rounding_mode, *value1, 0f32, *expected)?;
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                const INSTRUCTION: u32 = Assembler::make_cvt_s(FR::F4, FR::F0).d();
                test_floating_point_f64tof32::<INSTRUCTION>("CVT.S.D", *flush_denorm_to_zero, *rounding_mode, *value1, 0f64, *expected)?;
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, i32, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                const INSTRUCTION: u32 = Assembler::make_cvt_s(FR::F4, FR::F0).w();
                test_floating_point_i32tof32::<INSTRUCTION>("CVT.S.W", *flush_denorm_to_zero, *rounding_mode, *value1, 0, *expected)?;
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, i64, Result<(FCSRFlags, f32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                const INSTRUCTION: u32 = Assembler::make_cvt_s(FR::F4, FR::F0).l();
                test_floating_point_i64tof32::<INSTRUCTION>("CVT.S.L", *flush_denorm_to_zero, *rounding_mode, *value1, 0, *expected)?;
                return Ok(())
            }
            _ => {}
        }
        Err("Unhandled format".to_string())
    }
}

pub struct CvtD;

impl Test for CvtD {
    fn name(&self) -> &str { "COP1: CVT.D.fmt" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // S ==> D
            Box::new((false, FCSRRoundingMode::Nearest, 4f32, expected_result(FCSRFlags::new(), 4f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -0f32, expected_result(FCSRFlags::new(), -0f64))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, expected_result(FCSRFlags::new(), 1.1754943508222875e-38))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::INFINITY, expected_result(FCSRFlags::new(), f64::INFINITY))),
            Box::new((false, FCSRRoundingMode::Nearest, f32::NEG_INFINITY, expected_result(FCSRFlags::new(), f64::NEG_INFINITY))),

            // CVT(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_START_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::QUIET_NAN_NEGATIVE_END_32, expected_result(FCSRFlags::new().with_invalid_operation(true), COP1_RESULT_NAN_64))),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_START_32, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_END_32, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_START_32, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_f64())),

            // Subnormal also causes unimplemented
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_f64())),
            Box::new((true, FCSRRoundingMode::Nearest, FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_f64())),

            // D => D (which doesn't exist)
            Box::new((false, FCSRRoundingMode::Nearest, 4f64, expected_unimplemented_f64())),

            // W => D
            Box::new((false, FCSRRoundingMode::Nearest, 9i32, expected_result(FCSRFlags::new(), 9f64))),
            Box::new((false, FCSRRoundingMode::Zero, 1234567891i32, expected_result(FCSRFlags::new(), 1234567891f64))),
            Box::new((false, FCSRRoundingMode::Zero, -1234567891i32, expected_result(FCSRFlags::new(), -1234567891f64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -1234567891i32, expected_result(FCSRFlags::new(), -1234567891f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -1234567891i32, expected_result(FCSRFlags::new(), -1234567891f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, i32::MAX, expected_result(FCSRFlags::new(), 2147483647f64))),

            // L => D
            Box::new((false, FCSRRoundingMode::Nearest, 9i64, expected_result(FCSRFlags::new(), 9f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 1234567891i64, expected_result(FCSRFlags::new(), 1234567891f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -1234567891i32, expected_result(FCSRFlags::new(), -1234567891f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 53, expected_result(FCSRFlags::new(), 9007199254740992f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 54, expected_result(FCSRFlags::new(), 1.8014398509481984e16f64))),

            // Large numbers start being weird: We can see inexact here. Too large numbers aren't supported at all
            Box::new((false, FCSRRoundingMode::Nearest, (1i64 << 55) - 4, expected_result(FCSRFlags::new(), 3.6028797018963964e16f64))),
            Box::new((false, FCSRRoundingMode::Nearest, (1i64 << 55) - 3, expected_result(FCSRFlags::new().with_inexact_operation(true), 3.6028797018963964e16f64))),
            Box::new((false, FCSRRoundingMode::Nearest, (1i64 << 55) - 2, expected_result(FCSRFlags::new().with_inexact_operation(true), 3.602879701896397e16f64))),
            Box::new((false, FCSRRoundingMode::Zero, (1i64 << 55) - 2, expected_result(FCSRFlags::new().with_inexact_operation(true), 3.6028797018963964e16f64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, (1i64 << 55) - 2, expected_result(FCSRFlags::new().with_inexact_operation(true), 3.602879701896397e16f64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, (1i64 << 55) - 2, expected_result(FCSRFlags::new().with_inexact_operation(true), 3.6028797018963964e16f64))),
            Box::new((false, FCSRRoundingMode::Nearest, (1i64 << 55) - 1, expected_result(FCSRFlags::new().with_inexact_operation(true), 3.602879701896397e16f64))),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 55, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 56, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 61, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, 1i64 << 62, expected_unimplemented_f64())),

            // Same with negative numbers
            Box::new((false, FCSRRoundingMode::Nearest, -(1i64 << 55) + 2, expected_result(FCSRFlags::new().with_inexact_operation(true), -3.602879701896397e16f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -(1i64 << 55) + 1, expected_result(FCSRFlags::new().with_inexact_operation(true), -3.602879701896397e16f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -(1i64 << 55), expected_result(FCSRFlags::new(), -3.602879701896397e16f64))),
            Box::new((false, FCSRRoundingMode::Nearest, -(1i64 << 55) - 1, expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, -(1i64 << 56), expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, -(1i64 << 57), expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, -(1i64 << 61), expected_unimplemented_f64())),
            Box::new((false, FCSRRoundingMode::Nearest, -(1i64 << 62), expected_unimplemented_f64())),

            Box::new((false, FCSRRoundingMode::Nearest, i64::MAX, expected_unimplemented_f64())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                const INSTRUCTION: u32 = Assembler::make_cvt_d(FR::F4, FR::F0).s();
                test_floating_point_f32tof64::<INSTRUCTION>("CVT.D.S", *flush_denorm_to_zero, *rounding_mode, *value1, 0f32, *expected)?;
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                const INSTRUCTION: u32 = Assembler::make_cvt_d(FR::F4, FR::F0).d();
                test_floating_point_f64::<INSTRUCTION>("CVT.D.D", *flush_denorm_to_zero, *rounding_mode, *value1, 0f64, *expected)?;
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, i32, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                const INSTRUCTION: u32 = Assembler::make_cvt_d(FR::F4, FR::F0).w();
                test_floating_point_i32tof64::<INSTRUCTION>("CVT.D.W", *flush_denorm_to_zero, *rounding_mode, *value1, 0, *expected)?;
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, i64, Result<(FCSRFlags, f64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                const INSTRUCTION: u32 = Assembler::make_cvt_d(FR::F4, FR::F0).l();
                test_floating_point_i64tof64::<INSTRUCTION>("CVT.D.L", *flush_denorm_to_zero, *rounding_mode, *value1, 0, *expected)?;
                return Ok(())
            }
            _ => {}
        }
        Err("Unhandled format".to_string())
    }
}

pub struct ConvertToW;

impl Test for ConvertToW {
    fn name(&self) -> &str { "COP1: CVT.W, ROUND.W, TRUNC.W, CEIL.W" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // S ==> W
            Box::new((4f32, expected_result(FCSRFlags::new(), 4i32))),

            Box::new((false, FCSRRoundingMode::Nearest, 4.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i32))),
            Box::new((false, FCSRRoundingMode::Nearest, 5.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 6i32))),
            Box::new((false, FCSRRoundingMode::Nearest, 6.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 6i32))),
            Box::new((false, FCSRRoundingMode::Nearest, 7.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 8i32))),
            Box::new((false, FCSRRoundingMode::Nearest, 8.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 8i32))),

            Box::new((false, FCSRRoundingMode::Nearest, 4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i32))),
            Box::new((false, FCSRRoundingMode::Zero, 4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 5i32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i32))),

            Box::new((false, FCSRRoundingMode::Nearest, -4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i32))),
            Box::new((false, FCSRRoundingMode::Zero, -4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -5i32))),

            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i32))),
            Box::new((false, FCSRRoundingMode::Zero, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 1i32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i32))),

            Box::new((2140000000f32, expected_result(FCSRFlags::new(), 2140000000i32))),
            Box::new((2147483520f32, expected_result(FCSRFlags::new(), 2147483520i32))),
            Box::new((2147483600f32, expected_unimplemented_i32())),
            Box::new((2150000000f32, expected_unimplemented_i32())),

            Box::new((-2140000000f32, expected_result(FCSRFlags::new(), -2140000000i32))),
            Box::new((-2147483520f32, expected_result(FCSRFlags::new(), -2147483520i32))),
            Box::new((-2147483600f32, expected_result(FCSRFlags::new(), -2147483648i32))),
            Box::new((-2147483904f32, expected_unimplemented_i32())),
            Box::new((-2150000000f32, expected_unimplemented_i32())),

            Box::new((f32::INFINITY, expected_unimplemented_i32())),
            Box::new((f32::NEG_INFINITY, expected_unimplemented_i32())),

            // Quiet NANs aren't supported and cause unimplemented operation
            Box::new((FConst::QUIET_NAN_START_32, expected_unimplemented_i32())),
            Box::new((FConst::QUIET_NAN_END_32, expected_unimplemented_i32())),
            Box::new((FConst::QUIET_NAN_NEGATIVE_START_32, expected_unimplemented_i32())),
            Box::new((FConst::QUIET_NAN_NEGATIVE_END_32, expected_unimplemented_i32())),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((FConst::SIGNALLING_NAN_START_32, expected_unimplemented_i32())),
            Box::new((FConst::SIGNALLING_NAN_END_32, expected_unimplemented_i32())),
            Box::new((FConst::SIGNALLING_NAN_NEGATIVE_START_32, expected_unimplemented_i32())),
            Box::new((FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_i32())),

            // Subnormal also causes unimplemented
            Box::new((FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_i32())),
            Box::new((FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_i32())),
            Box::new((FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_i32())),
            Box::new((FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_i32())),

            // D => W
            Box::new((4f64, expected_result(FCSRFlags::new(), 4i32))),

            Box::new((false, FCSRRoundingMode::Nearest, 4.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i32))),
            Box::new((false, FCSRRoundingMode::Nearest, 5.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 6i32))),
            Box::new((false, FCSRRoundingMode::Nearest, 6.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 6i32))),
            Box::new((false, FCSRRoundingMode::Nearest, 7.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 8i32))),
            Box::new((false, FCSRRoundingMode::Nearest, 8.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 8i32))),

            Box::new((false, FCSRRoundingMode::Nearest, 4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i32))),
            Box::new((false, FCSRRoundingMode::Zero, 4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 5i32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i32))),

            Box::new((false, FCSRRoundingMode::Nearest, -4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i32))),
            Box::new((false, FCSRRoundingMode::Zero, -4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -5i32))),

            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i32))),
            Box::new((false, FCSRRoundingMode::Zero, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 1i32))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i32))),

            Box::new((2147483500f64, expected_result(FCSRFlags::new(), 2147483500i32))),
            Box::new((2147483647f64, expected_result(FCSRFlags::new(), 2147483647i32))),
            Box::new((2147483648f64, expected_unimplemented_i32())),
            Box::new((2147483649f64, expected_unimplemented_i32())),

            Box::new((-2147483500f64, expected_result(FCSRFlags::new(), -2147483500i32))),
            Box::new((-2147483647f64, expected_result(FCSRFlags::new(), -2147483647i32))),
            Box::new((-2147483648f64, expected_result(FCSRFlags::new(), -2147483648i32))),
            Box::new((-2147483649f64, expected_unimplemented_i32())),

            Box::new((false, FCSRRoundingMode::Nearest, 2147483647.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 2147483647i32))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 2147483647.4f64, expected_unimplemented_i32())),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 2147483647.6f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 2147483647i32))),
            Box::new((false, FCSRRoundingMode::Nearest, 2147483647.6f64, expected_unimplemented_i32())),

            Box::new((f64::INFINITY, expected_unimplemented_i32())),
            Box::new((f64::NEG_INFINITY, expected_unimplemented_i32())),

            // CVT(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((FConst::QUIET_NAN_START_64, expected_unimplemented_i32())),
            Box::new((FConst::QUIET_NAN_END_64, expected_unimplemented_i32())),
            Box::new((FConst::QUIET_NAN_NEGATIVE_START_64, expected_unimplemented_i32())),
            Box::new((FConst::QUIET_NAN_NEGATIVE_END_64, expected_unimplemented_i32())),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((FConst::SIGNALLING_NAN_START_64, expected_unimplemented_i32())),
            Box::new((FConst::SIGNALLING_NAN_END_64, expected_unimplemented_i32())),
            Box::new((FConst::SIGNALLING_NAN_NEGATIVE_START_64, expected_unimplemented_i32())),
            Box::new((FConst::SIGNALLING_NAN_NEGATIVE_END_64, expected_unimplemented_i32())),

            // Subnormal also causes unimplemented
            Box::new((FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_i32())),
            Box::new((FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_i32())),
            Box::new((FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_i32())),
            Box::new((FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_i32())),

            // W => W (which doesn't exist)
            Box::new((4i32, expected_unimplemented_i32())),

            // L => W (which doesn't exist)
            Box::new((4i64, expected_unimplemented_i32())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, i32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                // Call CVT
                const INSTRUCTION: u32 = Assembler::make_cvt_w(FR::F4, FR::F0).s();
                test_floating_point_f32toi32::<INSTRUCTION>("CVT.W.S", *flush_denorm_to_zero, *rounding_mode, *value1, 0f32, *expected)?;

                // Call ROUND/TRUNC/CEIL/FLOOR while the wrong rounding mode is set
                const ROUND_INSTRUCTION: u32 = Assembler::make_round_w(FR::F4, FR::F0).s();
                const TRUNC_INSTRUCTION: u32 = Assembler::make_trunc_w(FR::F4, FR::F0).s();
                const FLOOR_INSTRUCTION: u32 = Assembler::make_floor_w(FR::F4, FR::F0).s();
                const CEIL_INSTRUCTION: u32 = Assembler::make_ceil_w(FR::F4, FR::F0).s();

                for dummy_rounding_mode in [FCSRRoundingMode::PositiveInfinity, FCSRRoundingMode::NegativeInfinity] {
                    match *rounding_mode {
                        FCSRRoundingMode::Nearest => test_floating_point_f32toi32::<ROUND_INSTRUCTION>("ROUND.W.S", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f32, *expected)?,
                        FCSRRoundingMode::Zero => test_floating_point_f32toi32::<TRUNC_INSTRUCTION>("TRUNC.W.S", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f32, *expected)?,
                        FCSRRoundingMode::PositiveInfinity => test_floating_point_f32toi32::<CEIL_INSTRUCTION>("CEIL.W.S", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f32, *expected)?,
                        FCSRRoundingMode::NegativeInfinity => test_floating_point_f32toi32::<FLOOR_INSTRUCTION>("FLOOR.W.S", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f32, *expected)?,
                    }
                }
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, i32), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                // Call CVT
                const INSTRUCTION: u32 = Assembler::make_cvt_w(FR::F4, FR::F0).d();
                test_floating_point_f64toi32::<INSTRUCTION>("CVT.W.D", *flush_denorm_to_zero, *rounding_mode, *value1, 0f64, *expected)?;

                // Call ROUND/TRUNC/CEIL/FLOOR while the wrong rounding mode is set
                const ROUND_INSTRUCTION: u32 = Assembler::make_round_w(FR::F4, FR::F0).d();
                const TRUNC_INSTRUCTION: u32 = Assembler::make_trunc_w(FR::F4, FR::F0).d();
                const FLOOR_INSTRUCTION: u32 = Assembler::make_floor_w(FR::F4, FR::F0).d();
                const CEIL_INSTRUCTION: u32 = Assembler::make_ceil_w(FR::F4, FR::F0).d();

                for dummy_rounding_mode in [FCSRRoundingMode::PositiveInfinity, FCSRRoundingMode::NegativeInfinity] {
                    match *rounding_mode {
                        FCSRRoundingMode::Nearest => test_floating_point_f64toi32::<ROUND_INSTRUCTION>("ROUND.W.D", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f64, *expected)?,
                        FCSRRoundingMode::Zero => test_floating_point_f64toi32::<TRUNC_INSTRUCTION>("TRUNC.W.D", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f64, *expected)?,
                        FCSRRoundingMode::PositiveInfinity => test_floating_point_f64toi32::<CEIL_INSTRUCTION>("CEIL.W.D", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f64, *expected)?,
                        FCSRRoundingMode::NegativeInfinity => test_floating_point_f64toi32::<FLOOR_INSTRUCTION>("FLOOR.W.D", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f64, *expected)?,
                    }
                }
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(f32, Result<(FCSRFlags, i32), ()>)>() {
            Some((value1, expected)) => {
                // No rounding mode specified. Make a recursive call with all four possible ones
                for rounding_mode in FCSRRoundingMode::ALL {
                    let data: Box<dyn Any> = Box::new((false, rounding_mode, *value1, *expected));
                    self.run(&data)?;
                }
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(f64, Result<(FCSRFlags, i32), ()>)>() {
            Some((value1, expected)) => {
                // No rounding mode specified. Make a recursive call with all four possible ones
                for rounding_mode in FCSRRoundingMode::ALL {
                    let data: Box<dyn Any> = Box::new((false, rounding_mode, *value1, *expected));
                    self.run(&data)?;
                }
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(i32, Result<(FCSRFlags, i32), ()>)>() {
            Some((value1, expected)) => {
                for rounding_mode in FCSRRoundingMode::ALL {
                    const INSTRUCTION: u32 = Assembler::make_cvt_w(FR::F4, FR::F0).w();
                    test_floating_point_i32toi32::<INSTRUCTION>("CVT.W.W", false, rounding_mode, *value1, 0, *expected)?;

                    const INSTRUCTION2: u32 = Assembler::make_round_w(FR::F4, FR::F0).w();
                    test_floating_point_i32toi32::<INSTRUCTION2>("ROUND.W.W", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION3: u32 = Assembler::make_trunc_w(FR::F4, FR::F0).w();
                    test_floating_point_i32toi32::<INSTRUCTION3>("TRUNC.W.W", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION4: u32 = Assembler::make_ceil_w(FR::F4, FR::F0).w();
                    test_floating_point_i32toi32::<INSTRUCTION4>("CEIL.W.W", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION5: u32 = Assembler::make_floor_w(FR::F4, FR::F0).w();
                    test_floating_point_i32toi32::<INSTRUCTION5>("FLOOR.W.W", false, rounding_mode, *value1, 0, *expected)?;
                }
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(i64, Result<(FCSRFlags, i32), ()>)>() {
            Some((value1, expected)) => {
                for rounding_mode in FCSRRoundingMode::ALL {
                    const INSTRUCTION: u32 = Assembler::make_cvt_w(FR::F4, FR::F0).l();
                    test_floating_point_i64toi32::<INSTRUCTION>("CVT.W.L", false, rounding_mode, *value1, 0, *expected)?;

                    const INSTRUCTION2: u32 = Assembler::make_round_w(FR::F4, FR::F0).l();
                    test_floating_point_i64toi32::<INSTRUCTION2>("ROUND.W.L", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION3: u32 = Assembler::make_trunc_w(FR::F4, FR::F0).l();
                    test_floating_point_i64toi32::<INSTRUCTION3>("TRUNC.W.L", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION4: u32 = Assembler::make_ceil_w(FR::F4, FR::F0).l();
                    test_floating_point_i64toi32::<INSTRUCTION4>("CEIL.W.L", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION5: u32 = Assembler::make_floor_w(FR::F4, FR::F0).l();
                    test_floating_point_i64toi32::<INSTRUCTION5>("FLOOR.W.L", false, rounding_mode, *value1, 0, *expected)?;
                }
                return Ok(())
            }
            _ => {}
        }
        Err("Unhandled format".to_string())
    }
}

pub struct ConvertToL;

impl Test for ConvertToL {
    fn name(&self) -> &str { "COP1: CVT.L, ROUND.L, TRUNC.L, CEIL.L" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            // S ==> L
            Box::new((4f32, expected_result(FCSRFlags::new(), 4i64))),

            Box::new((false, FCSRRoundingMode::Nearest, 4.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i64))),
            Box::new((false, FCSRRoundingMode::Nearest, 5.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 6i64))),
            Box::new((false, FCSRRoundingMode::Nearest, 6.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 6i64))),
            Box::new((false, FCSRRoundingMode::Nearest, 7.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 8i64))),
            Box::new((false, FCSRRoundingMode::Nearest, 8.5f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 8i64))),

            Box::new((false, FCSRRoundingMode::Nearest, 4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i64))),
            Box::new((false, FCSRRoundingMode::Zero, 4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 5i64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i64))),

            Box::new((false, FCSRRoundingMode::Nearest, -4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i64))),
            Box::new((false, FCSRRoundingMode::Zero, -4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -4.4f32, expected_result(FCSRFlags::new().with_inexact_operation(true), -5i64))),

            Box::new((false, FCSRRoundingMode::Nearest, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i64))),
            Box::new((false, FCSRRoundingMode::Zero, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 1i64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f32::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i64))),

            Box::new((9.007198e15f32, expected_result(FCSRFlags::new(), 9007198180999168i64))),
            Box::new((9.00719871787e15f32, expected_result(FCSRFlags::new(), 9007198717870080i64))),
            Box::new((9.00719925474e15f32, expected_unimplemented_i64())),
            Box::new((9.00720032848e15f32, expected_unimplemented_i64())),

            Box::new((-9.007198e15f32, expected_result(FCSRFlags::new(), -9007198180999168i64))),
            Box::new((-9.00719871787e15f32, expected_result(FCSRFlags::new(), -9007198717870080i64))),
            Box::new((-9.00719925474e15f32, expected_unimplemented_i64())),
            Box::new((-9.00720032848e15f32, expected_unimplemented_i64())),

            Box::new((f32::INFINITY, expected_unimplemented_i64())),
            Box::new((f32::NEG_INFINITY, expected_unimplemented_i64())),

            // CVT(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((FConst::QUIET_NAN_START_32, expected_unimplemented_i64())),
            Box::new((FConst::QUIET_NAN_END_32, expected_unimplemented_i64())),
            Box::new((FConst::QUIET_NAN_NEGATIVE_START_32, expected_unimplemented_i64())),
            Box::new((FConst::QUIET_NAN_NEGATIVE_END_32, expected_unimplemented_i64())),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((FConst::SIGNALLING_NAN_START_32, expected_unimplemented_i64())),
            Box::new((FConst::SIGNALLING_NAN_END_32, expected_unimplemented_i64())),
            Box::new((FConst::SIGNALLING_NAN_NEGATIVE_START_32, expected_unimplemented_i64())),
            Box::new((FConst::SIGNALLING_NAN_NEGATIVE_END_32, expected_unimplemented_i64())),

            // Subnormal also causes unimplemented
            Box::new((FConst::SUBNORMAL_MIN_POSITIVE_32, expected_unimplemented_i64())),
            Box::new((FConst::SUBNORMAL_MAX_POSITIVE_32, expected_unimplemented_i64())),
            Box::new((FConst::SUBNORMAL_MIN_NEGATIVE_32, expected_unimplemented_i64())),
            Box::new((FConst::SUBNORMAL_MAX_NEGATIVE_32, expected_unimplemented_i64())),

            // D => L
            Box::new((4f64, expected_result(FCSRFlags::new(), 4i64))),

            Box::new((false, FCSRRoundingMode::Nearest, 4.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i64))),
            Box::new((false, FCSRRoundingMode::Nearest, 5.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 6i64))),
            Box::new((false, FCSRRoundingMode::Nearest, 6.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 6i64))),
            Box::new((false, FCSRRoundingMode::Nearest, 7.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 8i64))),
            Box::new((false, FCSRRoundingMode::Nearest, 8.5f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 8i64))),

            Box::new((false, FCSRRoundingMode::Nearest, 4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i64))),
            Box::new((false, FCSRRoundingMode::Zero, 4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, 4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 5i64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, 4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), 4i64))),

            Box::new((false, FCSRRoundingMode::Nearest, -4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i64))),
            Box::new((false, FCSRRoundingMode::Zero, -4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -4i64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -4.4f64, expected_result(FCSRFlags::new().with_inexact_operation(true), -5i64))),

            Box::new((false, FCSRRoundingMode::Nearest, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i64))),
            Box::new((false, FCSRRoundingMode::Zero, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 1i64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i64))),

            Box::new((false, FCSRRoundingMode::Nearest, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i64))),
            Box::new((false, FCSRRoundingMode::Zero, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i64))),
            Box::new((false, FCSRRoundingMode::PositiveInfinity, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), 0i64))),
            Box::new((false, FCSRRoundingMode::NegativeInfinity, -f64::MIN_POSITIVE, expected_result(FCSRFlags::new().with_inexact_operation(true), -1i64))),

            Box::new((4e15f64, expected_result(FCSRFlags::new(), 4000000000000000i64))),
            Box::new((8e15f64, expected_result(FCSRFlags::new(), 8000000000000000i64))),
            Box::new((9e15f64, expected_result(FCSRFlags::new(), 9000000000000000i64))),

            Box::new((9007199254740991f64, expected_result(FCSRFlags::new(), 9007199254740991i64))),
            Box::new((9007199254740992f64, expected_unimplemented_i64())),

            Box::new((-9007199254740990f64, expected_result(FCSRFlags::new(), -9007199254740990i64))),
            Box::new((-9007199254740991f64, expected_result(FCSRFlags::new(), -9007199254740991i64))),
            Box::new((-9007199254740992f64, expected_unimplemented_i64())),
            Box::new((-9007199254740993f64, expected_unimplemented_i64())),

            Box::new((f64::INFINITY, expected_unimplemented_i64())),
            Box::new((f64::NEG_INFINITY, expected_unimplemented_i64())),

            // CVT(NAN) produces another NAN and invalid operation (which is the opposite of what their name implies)
            Box::new((FConst::QUIET_NAN_START_64, expected_unimplemented_i64())),
            Box::new((FConst::QUIET_NAN_END_64, expected_unimplemented_i64())),
            Box::new((FConst::QUIET_NAN_NEGATIVE_START_64, expected_unimplemented_i64())),
            Box::new((FConst::QUIET_NAN_NEGATIVE_END_64, expected_unimplemented_i64())),

            // Signalling NANs aren't supported and cause unimplemented operation
            Box::new((FConst::SIGNALLING_NAN_START_64, expected_unimplemented_i64())),
            Box::new((FConst::SIGNALLING_NAN_END_64, expected_unimplemented_i64())),
            Box::new((FConst::SIGNALLING_NAN_NEGATIVE_START_64, expected_unimplemented_i64())),
            Box::new((FConst::SIGNALLING_NAN_NEGATIVE_END_64, expected_unimplemented_i64())),

            // Subnormal also causes unimplemented
            Box::new((FConst::SUBNORMAL_MIN_POSITIVE_64, expected_unimplemented_i64())),
            Box::new((FConst::SUBNORMAL_MAX_POSITIVE_64, expected_unimplemented_i64())),
            Box::new((FConst::SUBNORMAL_MIN_NEGATIVE_64, expected_unimplemented_i64())),
            Box::new((FConst::SUBNORMAL_MAX_NEGATIVE_64, expected_unimplemented_i64())),

            // W => L (which doesn't exist)
            Box::new((4i64, expected_unimplemented_i64())),

            // L => L (which doesn't exist)
            Box::new((4i64, expected_unimplemented_i64())),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, i64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                // Call CVT
                const INSTRUCTION: u32 = Assembler::make_cvt_l(FR::F4, FR::F0).s();
                test_floating_point_f32toi64::<INSTRUCTION>("CVT.L.S", *flush_denorm_to_zero, *rounding_mode, *value1, 0f32, *expected)?;

                const ROUND_INSTRUCTION: u32 = Assembler::make_round_l(FR::F4, FR::F0).s();
                const TRUNC_INSTRUCTION: u32 = Assembler::make_trunc_l(FR::F4, FR::F0).s();
                const FLOOR_INSTRUCTION: u32 = Assembler::make_floor_l(FR::F4, FR::F0).s();
                const CEIL_INSTRUCTION: u32 = Assembler::make_ceil_l(FR::F4, FR::F0).s();

                for dummy_rounding_mode in [FCSRRoundingMode::PositiveInfinity, FCSRRoundingMode::NegativeInfinity] {
                    match *rounding_mode {
                        FCSRRoundingMode::Nearest => test_floating_point_f32toi64::<ROUND_INSTRUCTION>("ROUND.L.S", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f32, *expected)?,
                        FCSRRoundingMode::Zero => test_floating_point_f32toi64::<TRUNC_INSTRUCTION>("TRUNC.L.S", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f32, *expected)?,
                        FCSRRoundingMode::PositiveInfinity => test_floating_point_f32toi64::<CEIL_INSTRUCTION>("CEIL.L.S", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f32, *expected)?,
                        FCSRRoundingMode::NegativeInfinity => test_floating_point_f32toi64::<FLOOR_INSTRUCTION>("FLOOR.L.S", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f32, *expected)?,
                    }
                }
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, i64), ()>)>() {
            Some((flush_denorm_to_zero, rounding_mode, value1, expected)) => {
                // Call CVT
                const INSTRUCTION: u32 = Assembler::make_cvt_l(FR::F4, FR::F0).d();
                test_floating_point_f64toi64::<INSTRUCTION>("CVT.L.D", *flush_denorm_to_zero, *rounding_mode, *value1, 0f64, *expected)?;

                // Call ROUND/TRUNC/CEIL/FLOOR while the wrong rounding mode is set
                const ROUND_INSTRUCTION: u32 = Assembler::make_round_l(FR::F4, FR::F0).d();
                const TRUNC_INSTRUCTION: u32 = Assembler::make_trunc_l(FR::F4, FR::F0).d();
                const FLOOR_INSTRUCTION: u32 = Assembler::make_floor_l(FR::F4, FR::F0).d();
                const CEIL_INSTRUCTION: u32 = Assembler::make_ceil_l(FR::F4, FR::F0).d();

                for dummy_rounding_mode in [FCSRRoundingMode::PositiveInfinity, FCSRRoundingMode::NegativeInfinity] {
                    match *rounding_mode {
                        FCSRRoundingMode::Nearest => test_floating_point_f64toi64::<ROUND_INSTRUCTION>("ROUND.L.D", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f64, *expected)?,
                        FCSRRoundingMode::Zero => test_floating_point_f64toi64::<TRUNC_INSTRUCTION>("TRUNC.L.D", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f64, *expected)?,
                        FCSRRoundingMode::PositiveInfinity => test_floating_point_f64toi64::<CEIL_INSTRUCTION>("CEIL.L.D", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f64, *expected)?,
                        FCSRRoundingMode::NegativeInfinity => test_floating_point_f64toi64::<FLOOR_INSTRUCTION>("FLOOR.L.D", *flush_denorm_to_zero, dummy_rounding_mode, *value1, 0f64, *expected)?,
                    }
                }
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(f32, Result<(FCSRFlags, i64), ()>)>() {
            Some((value1, expected)) => {
                // No rounding mode specified. Make a recursive call with all four possible ones
                for rounding_mode in FCSRRoundingMode::ALL {
                    let data: Box<dyn Any> = Box::new((false, rounding_mode, *value1, *expected));
                    self.run(&data)?;
                }
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(f64, Result<(FCSRFlags, i64), ()>)>() {
            Some((value1, expected)) => {
                // No rounding mode specified. Make a recursive call with all four possible ones
                for rounding_mode in FCSRRoundingMode::ALL {
                    let data: Box<dyn Any> = Box::new((false, rounding_mode, *value1, *expected));
                    self.run(&data)?;
                }
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(i32, Result<(FCSRFlags, i64), ()>)>() {
            Some((value1, expected)) => {
                for rounding_mode in FCSRRoundingMode::ALL {
                    const INSTRUCTION: u32 = Assembler::make_cvt_l(FR::F4, FR::F0).w();
                    test_floating_point_i32toi64::<INSTRUCTION>("CVT.L.W", false, rounding_mode, *value1, 0, *expected)?;

                    const INSTRUCTION2: u32 = Assembler::make_round_l(FR::F4, FR::F0).w();
                    test_floating_point_i32toi64::<INSTRUCTION2>("ROUND.L.W", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION3: u32 = Assembler::make_trunc_l(FR::F4, FR::F0).w();
                    test_floating_point_i32toi64::<INSTRUCTION3>("TRUNC.L.W", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION4: u32 = Assembler::make_ceil_l(FR::F4, FR::F0).w();
                    test_floating_point_i32toi64::<INSTRUCTION4>("CEIL.L.W", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION5: u32 = Assembler::make_floor_l(FR::F4, FR::F0).w();
                    test_floating_point_i32toi64::<INSTRUCTION5>("FLOOR.L.W", false, rounding_mode, *value1, 0, *expected)?;
                }
                return Ok(())
            }
            _ => {}
        }
        match (*value).downcast_ref::<(i64, Result<(FCSRFlags, i64), ()>)>() {
            Some((value1, expected)) => {
                for rounding_mode in FCSRRoundingMode::ALL {
                    const INSTRUCTION: u32 = Assembler::make_cvt_l(FR::F4, FR::F0).l();
                    test_floating_point_i64toi64::<INSTRUCTION>("CVT.L.L", false, rounding_mode, *value1, 0, *expected)?;

                    const INSTRUCTION2: u32 = Assembler::make_round_l(FR::F4, FR::F0).l();
                    test_floating_point_i64toi64::<INSTRUCTION2>("ROUND.L.L", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION3: u32 = Assembler::make_trunc_l(FR::F4, FR::F0).l();
                    test_floating_point_i64toi64::<INSTRUCTION3>("TRUNC.L.L", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION4: u32 = Assembler::make_ceil_l(FR::F4, FR::F0).l();
                    test_floating_point_i64toi64::<INSTRUCTION4>("CEIL.L.L", false, rounding_mode, *value1, 0, *expected)?;
                    const INSTRUCTION5: u32 = Assembler::make_floor_l(FR::F4, FR::F0).l();
                    test_floating_point_i64toi64::<INSTRUCTION5>("FLOOR.L.L", false, rounding_mode, *value1, 0, *expected)?;
                }
                return Ok(())
            }
            _ => {}
        }
        Err("Unhandled format".to_string())
    }
}

fn test_fcsr_unchanged<const INSTRUCTION: u32>() -> Result<(), String> {

    let mut temp: u64 = 0x01234567;

    let fs = FCSR::new().with_enable_overflow(true).with_cause_division_by_zero(true).with_condition(true).with_rounding_mode(FCSRRoundingMode::PositiveInfinity);
    set_fcsr(fs);
    unsafe {
        asm!("
        .WORD {INSTRUCTION}
    ", INSTRUCTION = const INSTRUCTION, inout("$2") &mut temp => _)
    };
    soft_assert_eq(fs, fcsr(), "FCSR was modified but should have not")
}

pub struct LWC1PreservingFCSR;

impl Test for LWC1PreservingFCSR {
    fn name(&self) -> &str { "LWC1 preserving FCSR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_lwc1(GPR::V0, 0, GPR::V0);
        test_fcsr_unchanged::<INSTRUCTION>()
    }
}

pub struct SWC1PreservingFCSR;

impl Test for SWC1PreservingFCSR {
    fn name(&self) -> &str { "SWC1 preserving FCSR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_swc1(GPR::V0, 0, GPR::V0);
        test_fcsr_unchanged::<INSTRUCTION>()
    }
}

pub struct LDC1PreservingFCSR;

impl Test for LDC1PreservingFCSR {
    fn name(&self) -> &str { "LDC1 preserving FCSR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_ldc1(GPR::V0, 0, GPR::V0);
        test_fcsr_unchanged::<INSTRUCTION>()
    }
}

pub struct SDC1PreservingFCSR;

impl Test for SDC1PreservingFCSR {
    fn name(&self) -> &str { "SDC1 preserving FCSR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sdc1(GPR::V0, 0, GPR::V0);
        test_fcsr_unchanged::<INSTRUCTION>()
    }
}
