use alloc::boxed::Box;
use alloc::vec;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use core::cmp::Ordering;
use arbitrary_int::u2;
use crate::assembler::{Assembler, Cop1Condition, FPUFloatInstruction, FR};
use crate::cop0::{Cause, CauseException, preset_cause_to_copindex2};
use crate::cop1::{FConst, FCSR, fcsr, FCSRFlags, FCSRRoundingMode, set_fcsr};
use crate::exception_handler::expect_exception;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

/// Tests the given FP instruction in both regular and delay position and ensures that the result was seen and the right set of exceptions is being fired
fn test_compare<FIn: Copy>(
    f: fn(FIn, FIn),
    rounding_mode: FCSRRoundingMode,
    value1: FIn,
    value2: FIn,
    instruction: u32,
    expected: Result<(FCSRFlags, bool), ()>) -> Result<(), String> {
    // Run the operation without any exceptions enabled
    if let Ok((expected_flags, expected_result)) = expected {
        // Run once with all exceptions enabled that we don't care about and once with all disabled
        for enabled in [ FCSRFlags::NONE, expected_flags.invert() ] {
            for condition_before in [false, true] {
                set_fcsr(FCSR::new().with_condition(condition_before).with_rounding_mode(rounding_mode).with_enables(enabled));
                f(value1, value2);
                let result_fcsr = fcsr();
                // Copy cause bits into flag bits for no-exception case
                let expected_fcsr = FCSR::new()
                    .with_condition(expected_result)
                    .with_rounding_mode(rounding_mode)
                    .with_flags(expected_flags)
                    .with_enables(enabled)
                    .with_maskable_causes(expected_flags);
                soft_assert_eq(result_fcsr, expected_fcsr, "FCSR after operation with exceptions disabled")?;
            }
        }

        if expected_flags == FCSRFlags::NONE {
            // No exceptions expected. We are done
            return Ok(())
        }
    }

    // If we're expecting any exceptions, set things up now
    preset_cause_to_copindex2()?;

    // Exception test. Start with figuring out which combinations of enabled-flags to run
    let (enabled_flags, expected_cause) = if let Ok((expected_flags, _)) = expected {
        (expected_flags, FCSR::new().with_maskable_causes(expected_flags))
    } else {
        (FCSRFlags::NONE, FCSR::new().with_cause_unimplemented_operation(true))
    };

    for condition_before in [false, true] {
        let exception_context = expect_exception(CauseException::FPE, 1, || {
            set_fcsr(FCSR::new().with_condition(condition_before).with_rounding_mode(rounding_mode).with_enables(enabled_flags));

            f(value1, value2);

            set_fcsr(FCSR::new());
            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, instruction, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, Cause::new().with_coprocessor_error(u2::new(0)).with_exception(CauseException::FPE), "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;
        let expected_fcsr = expected_cause.with_condition(condition_before).with_rounding_mode(rounding_mode).with_enables(enabled_flags);
        soft_assert_eq(exception_context.fcsr, expected_fcsr, "FCSR after operation with exceptions enabled")?;
    }

    Ok(())
}

fn test_compare_f32<const INSTRUCTION: u32>(rounding_mode: FCSRRoundingMode, value1: f32, value2: f32, expected: Result<(FCSRFlags, bool), ()>) -> Result<(), String> {
    fn asm_block<const INSTRUCTION: u32>(value1: f32, value2: f32) {
        unsafe {
            asm!("
                .set noat
                .set noreorder
                .word {INSTRUCTION}
                nop
                nop
            ",
            INSTRUCTION = const INSTRUCTION,
            in("$f0") value1,
            in("$f2") value2,
            options(nostack, nomem))
        }
    }

    test_compare(
        asm_block::<INSTRUCTION>,
        rounding_mode,
        value1,
        value2,
        INSTRUCTION,
        expected)
}

fn test_compare_f64<const INSTRUCTION: u32>(rounding_mode: FCSRRoundingMode, value1: f64, value2: f64, expected: Result<(FCSRFlags, bool), ()>) -> Result<(), String> {
    fn asm_block<const INSTRUCTION: u32>(value1: f64, value2: f64) {
        unsafe {
            asm!("
                .set noat
                .set noreorder
                .word {INSTRUCTION}
                nop
                nop
            ",
            INSTRUCTION = const INSTRUCTION,
            in("$f0") value1,
            in("$f2") value2,
            options(nostack, nomem))
        }
    }

    test_compare(
        asm_block::<INSTRUCTION>,
        rounding_mode,
        value1,
        value2,
        INSTRUCTION,
        expected)
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum FPUSpecialNumber {
    Nope,
    QuietNAN,
    SignallingNAN,
    BothNAN,
    Subnormal,
    QuietNANAndSubnormal,
}

fn test_values() -> Vec<Box<dyn Any>> {
    vec! {
        // Singles
        Box::new((1f32, 2f32, Ordering::Less, FPUSpecialNumber::Nope)),
        Box::new((1f32, -1f32, Ordering::Greater, FPUSpecialNumber::Nope)),
        Box::new((1f32, f32::INFINITY, Ordering::Less, FPUSpecialNumber::Nope)),
        Box::new((1f32, f32::NEG_INFINITY, Ordering::Greater, FPUSpecialNumber::Nope)),
        Box::new((f32::INFINITY, f32::NEG_INFINITY, Ordering::Greater, FPUSpecialNumber::Nope)),

        Box::new((0f32, 0f32, Ordering::Equal, FPUSpecialNumber::Nope)),
        Box::new((0f32, -0f32, Ordering::Equal, FPUSpecialNumber::Nope)),
        Box::new((-0f32, 0f32, Ordering::Equal, FPUSpecialNumber::Nope)),
        Box::new((f32::INFINITY, f32::INFINITY, Ordering::Equal, FPUSpecialNumber::Nope)),
        Box::new((f32::MIN_POSITIVE, f32::MIN_POSITIVE, Ordering::Equal, FPUSpecialNumber::Nope)),
        Box::new((f32::NEG_INFINITY, f32::NEG_INFINITY, Ordering::Equal, FPUSpecialNumber::Nope)),

        Box::new((FConst::QUIET_NAN_START_32, 0f32, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((FConst::QUIET_NAN_END_32, 0f32, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((FConst::QUIET_NAN_NEGATIVE_START_32, 0f32, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((FConst::QUIET_NAN_NEGATIVE_END_32, 0f32, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((1f32, FConst::QUIET_NAN_START_32, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((1f32, FConst::QUIET_NAN_END_32, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((1f32, FConst::QUIET_NAN_NEGATIVE_START_32, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((1f32, FConst::QUIET_NAN_NEGATIVE_END_32, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((FConst::QUIET_NAN_START_32, FConst::QUIET_NAN_START_32, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((FConst::QUIET_NAN_START_32, FConst::QUIET_NAN_END_32, Ordering::Equal, FPUSpecialNumber::QuietNAN)),

        Box::new((FConst::SIGNALLING_NAN_START_32, 0f32, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((FConst::SIGNALLING_NAN_END_32, 0f32, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((FConst::SIGNALLING_NAN_NEGATIVE_START_32, 0f32, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((FConst::SIGNALLING_NAN_NEGATIVE_END_32, 0f32, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((1f32, FConst::SIGNALLING_NAN_START_32, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((1f32, FConst::SIGNALLING_NAN_END_32, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((1f32, FConst::SIGNALLING_NAN_NEGATIVE_START_32, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((1f32, FConst::SIGNALLING_NAN_NEGATIVE_END_32, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((FConst::SIGNALLING_NAN_START_32, FConst::SIGNALLING_NAN_START_32, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((FConst::SIGNALLING_NAN_START_32, FConst::SIGNALLING_NAN_END_32, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),

        Box::new((FConst::SIGNALLING_NAN_START_32, FConst::QUIET_NAN_START_32, Ordering::Equal, FPUSpecialNumber::BothNAN)),
        Box::new((FConst::QUIET_NAN_START_32, FConst::SIGNALLING_NAN_START_32, Ordering::Equal, FPUSpecialNumber::BothNAN)),

        Box::new((FConst::SUBNORMAL_MIN_POSITIVE_32, 0f32, Ordering::Greater, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MIN_NEGATIVE_32, 0f32, Ordering::Less, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MIN_POSITIVE_32, FConst::SUBNORMAL_MAX_POSITIVE_32, Ordering::Less, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MAX_POSITIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, Ordering::Greater, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MIN_NEGATIVE_32, FConst::SUBNORMAL_MAX_NEGATIVE_32, Ordering::Greater, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MAX_NEGATIVE_32, FConst::SUBNORMAL_MIN_NEGATIVE_32, Ordering::Less, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MIN_NEGATIVE_32, FConst::SUBNORMAL_MIN_POSITIVE_32, Ordering::Less, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MAX_POSITIVE_32, FConst::SUBNORMAL_MAX_POSITIVE_32, Ordering::Equal, FPUSpecialNumber::Subnormal)),

        Box::new((FConst::QUIET_NAN_START_32, FConst::SUBNORMAL_MAX_POSITIVE_32, Ordering::Equal, FPUSpecialNumber::QuietNANAndSubnormal)),

        // Doubles
        Box::new((1f64, 2f64, Ordering::Less, FPUSpecialNumber::Nope)),
        Box::new((1f64, -1f64, Ordering::Greater, FPUSpecialNumber::Nope)),
        Box::new((1f64, f64::INFINITY, Ordering::Less, FPUSpecialNumber::Nope)),
        Box::new((1f64, f64::NEG_INFINITY, Ordering::Greater, FPUSpecialNumber::Nope)),
        Box::new((f64::INFINITY, f64::NEG_INFINITY, Ordering::Greater, FPUSpecialNumber::Nope)),

        Box::new((0f64, 0f64, Ordering::Equal, FPUSpecialNumber::Nope)),
        Box::new((0f64, -0f64, Ordering::Equal, FPUSpecialNumber::Nope)),
        Box::new((-0f64, 0f64, Ordering::Equal, FPUSpecialNumber::Nope)),
        Box::new((f64::INFINITY, f64::INFINITY, Ordering::Equal, FPUSpecialNumber::Nope)),
        Box::new((f64::MIN_POSITIVE, f64::MIN_POSITIVE, Ordering::Equal, FPUSpecialNumber::Nope)),
        Box::new((f64::NEG_INFINITY, f64::NEG_INFINITY, Ordering::Equal, FPUSpecialNumber::Nope)),

        Box::new((FConst::QUIET_NAN_START_64, 0f64, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((FConst::QUIET_NAN_END_64, 0f64, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((FConst::QUIET_NAN_NEGATIVE_START_64, 0f64, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((FConst::QUIET_NAN_NEGATIVE_END_64, 0f64, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((1f64, FConst::QUIET_NAN_START_64, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((1f64, FConst::QUIET_NAN_END_64, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((1f64, FConst::QUIET_NAN_NEGATIVE_START_64, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((1f64, FConst::QUIET_NAN_NEGATIVE_END_64, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((FConst::QUIET_NAN_START_64, FConst::QUIET_NAN_START_64, Ordering::Equal, FPUSpecialNumber::QuietNAN)),
        Box::new((FConst::QUIET_NAN_START_64, FConst::QUIET_NAN_END_64, Ordering::Equal, FPUSpecialNumber::QuietNAN)),

        Box::new((FConst::SIGNALLING_NAN_START_64, 0f64, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((FConst::SIGNALLING_NAN_END_64, 0f64, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((FConst::SIGNALLING_NAN_NEGATIVE_START_64, 0f64, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((FConst::SIGNALLING_NAN_NEGATIVE_END_64, 0f64, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((1f64, FConst::SIGNALLING_NAN_START_64, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((1f64, FConst::SIGNALLING_NAN_END_64, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((1f64, FConst::SIGNALLING_NAN_NEGATIVE_START_64, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((1f64, FConst::SIGNALLING_NAN_NEGATIVE_END_64, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((FConst::SIGNALLING_NAN_START_64, FConst::SIGNALLING_NAN_START_64, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),
        Box::new((FConst::SIGNALLING_NAN_START_64, FConst::SIGNALLING_NAN_END_64, Ordering::Equal, FPUSpecialNumber::SignallingNAN)),

        Box::new((FConst::SIGNALLING_NAN_START_64, FConst::QUIET_NAN_START_64, Ordering::Equal, FPUSpecialNumber::BothNAN)),
        Box::new((FConst::QUIET_NAN_START_64, FConst::SIGNALLING_NAN_START_64, Ordering::Equal, FPUSpecialNumber::BothNAN)),

        Box::new((FConst::SUBNORMAL_MIN_POSITIVE_64, 0f64, Ordering::Greater, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MIN_NEGATIVE_64, 0f64, Ordering::Less, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MIN_POSITIVE_64, FConst::SUBNORMAL_MAX_POSITIVE_64, Ordering::Less, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MAX_POSITIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, Ordering::Greater, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MIN_NEGATIVE_64, FConst::SUBNORMAL_MAX_NEGATIVE_64, Ordering::Greater, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MAX_NEGATIVE_64, FConst::SUBNORMAL_MIN_NEGATIVE_64, Ordering::Less, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MIN_NEGATIVE_64, FConst::SUBNORMAL_MIN_POSITIVE_64, Ordering::Less, FPUSpecialNumber::Subnormal)),
        Box::new((FConst::SUBNORMAL_MAX_POSITIVE_64, FConst::SUBNORMAL_MAX_POSITIVE_64, Ordering::Equal, FPUSpecialNumber::Subnormal)),

        Box::new((FConst::QUIET_NAN_START_64, FConst::SUBNORMAL_MAX_POSITIVE_64, Ordering::Equal, FPUSpecialNumber::QuietNANAndSubnormal)),
    }
}

fn test_impl<FMatch: Fn(Ordering, FPUSpecialNumber) -> Result<(FCSRFlags, bool), ()>, const INSTRUCTION_S: u32, const INSTRUCTION_D: u32>(value: &Box<dyn Any>, matcher: FMatch) -> Result<(), String> {
    match (*value).downcast_ref::<(f32, f32, Ordering, FPUSpecialNumber)>() {
        Some((value1, value2, ordering, special)) => {
            for rounding_mode in FCSRRoundingMode::ALL {
                test_compare_f32::<INSTRUCTION_S>(rounding_mode, *value1, *value2, matcher(*ordering, *special))?;
            }
            return Ok(())
        }
        _ => {}
    }
    match (*value).downcast_ref::<(f64, f64, Ordering, FPUSpecialNumber)>() {
        Some((value1, value2, ordering, special)) => {
            for rounding_mode in FCSRRoundingMode::ALL {
                test_compare_f64::<INSTRUCTION_D>(rounding_mode, *value1, *value2, matcher(*ordering, *special))?;
            }
            return Ok(())
        }
        _ => {}
    }

    return Err("Unhandled match pattern".to_string())
}

#[allow(non_camel_case_types)]
pub struct C_F;

impl Test for C_F {
    fn name(&self) -> &str { "COP1 C.F" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::F, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |_order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, false)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::NONE, false)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::NONE, false)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_UN;

impl Test for C_UN {
    fn name(&self) -> &str { "COP1 C.UN" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::UN, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |_order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, false)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new(), true)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), false)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_EQ;

impl Test for C_EQ {
    fn name(&self) -> &str { "COP1 C.EQ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::EQ, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Equal)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new(), false)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Equal)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_UEQ;

impl Test for C_UEQ {
    fn name(&self) -> &str { "COP1 C.UEQ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::UEQ, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Equal)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new(), true)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Equal)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_OLT;

impl Test for C_OLT {
    fn name(&self) -> &str { "COP1 C.OLT" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::OLT, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Less)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new(), false)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Less)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_ULT;

impl Test for C_ULT {
    fn name(&self) -> &str { "COP1 C.ULT" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::ULT, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Less)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new(), true)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Less)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_OLE;

impl Test for C_OLE {
    fn name(&self) -> &str { "COP1 C.OLE" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::OLE, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Less || order == Ordering::Equal)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new(), false)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Less || order == Ordering::Equal)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_ULE;

impl Test for C_ULE {
    fn name(&self) -> &str { "COP1 C.ULE" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::ULE, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Less || order == Ordering::Equal)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new(), true)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Less || order == Ordering::Equal)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_SF;

impl Test for C_SF {
    fn name(&self) -> &str { "COP1 C.SF" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::SF, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |_order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, false)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::NONE, false)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_NGLE;

impl Test for C_NGLE {
    fn name(&self) -> &str { "COP1 C.NGLE" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::NGLE, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |_order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, false)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), false)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_SEQ;

impl Test for C_SEQ {
    fn name(&self) -> &str { "COP1 C.SEQ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::SEQ, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Equal)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Equal)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_NGL;

impl Test for C_NGL {
    fn name(&self) -> &str { "COP1 C.NGL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::NGL, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Equal)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Equal)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_LT;

impl Test for C_LT {
    fn name(&self) -> &str { "COP1 C.LT" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::LT, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Less)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Less)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_NGE;

impl Test for C_NGE {
    fn name(&self) -> &str { "COP1 C.NGE" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::NGE, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Less)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Less)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_LE;

impl Test for C_LE {
    fn name(&self) -> &str { "COP1 C.LE" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::LE, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Less || order == Ordering::Equal)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Less || order == Ordering::Equal)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), false)),
        })
    }
}

#[allow(non_camel_case_types)]
pub struct C_NGT;

impl Test for C_NGT {
    fn name(&self) -> &str { "COP1 C.NGT" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { test_values() }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: FPUFloatInstruction = Assembler::make_c_cond(Cop1Condition::NGT, FR::F0, FR::F2);
        const INSTRUCTION_S: u32 = INSTRUCTION.s();
        const INSTRUCTION_D: u32 = INSTRUCTION.d();

        test_impl::<_, INSTRUCTION_S, INSTRUCTION_D>(value, |order, special| match special {
            FPUSpecialNumber::Nope => Ok((FCSRFlags::NONE, order == Ordering::Less || order == Ordering::Equal)),
            FPUSpecialNumber::QuietNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::SignallingNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::Subnormal => Ok((FCSRFlags::new(), order == Ordering::Less || order == Ordering::Equal)),
            FPUSpecialNumber::BothNAN => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
            FPUSpecialNumber::QuietNANAndSubnormal => Ok((FCSRFlags::new().with_invalid_operation(true), true)),
        })
    }
}
