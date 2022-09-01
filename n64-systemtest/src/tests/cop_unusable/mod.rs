use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use arbitrary_int::{u2, u5};
use crate::assembler::{Assembler, FR, GPR};
use crate::cop0;
use crate::cop0::{badvaddr, Cause, CauseException, context_64, set_context_64, set_xcontext_64, Status, xcontext_64};
use crate::cop1::{FCSR, set_fcsr};
use crate::exception_handler::expect_exception;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Various tests for COP unusable (and related). Some findings:
// - COP 0..=3 can be enabled and disabled independently
// - COP3 does not exist - any instruction fires RI without the cop-index set
// - Illegal COP1 instructions (while enabled) fire FloatingPointException
// - Illegal COP2 instructions (while enabled) fire RI, with cop-index = 2
// - Some COP2 instructions (MFC2, LWC2 etc) exist, but they don't do much. Tests below are very
//   incomplete
// - COP0 unusable probably exists, but it isn't tested yet as we're running in kernel mode where
//   it can't fire

fn test_masking(value: Status) -> Result<(), String> {
    unsafe { cop0::set_status(value); }
    soft_assert_eq(value, cop0::status(), "Flag should be settable")?;
    unsafe { cop0::set_status(Status::new()); }
    soft_assert_eq(Status::new(), cop0::status(), "Flag should be clearable")?;
    Ok(())
}

pub struct COP3Usable {}

impl Test for COP3Usable {
    fn name(&self) -> &str { "COP3Usable (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_masking(Status::new().with_cop3usable(true))
    }
}

pub struct COP2Usable {}

impl Test for COP2Usable {
    fn name(&self) -> &str { "COP2Usable (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_masking(Status::new().with_cop2usable(true))
    }
}

pub struct COP1Usable {}

impl Test for COP1Usable {
    fn name(&self) -> &str { "COP1Usable (masking)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_masking(Status::new().with_cop1usable(true))
    }
}

pub struct COP0Usable {}

impl Test for COP0Usable {
    fn name(&self) -> &str { "COP0Usable (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_masking(Status::new().with_cop0usable(true))
    }
}

fn test_instruction_causes_exception<const INSTRUCTION: u32>(
    cop_index_usable: u2, usable: Status, exception_usable: CauseException, fcsr_usable: FCSR,
    cop_index_unusable: u2, unusable: Status, exception_unusable: CauseException, fcsr_unusable: FCSR,) -> Result<(), String> {
    for (desc, cop_index, status, exception, fcsr) in [("usable", cop_index_usable, usable, exception_usable, fcsr_usable), ("unusable", cop_index_unusable, unusable, exception_unusable, fcsr_unusable)] {
        let exception_context = expect_exception(exception, 1, || {
            unsafe { cop0::set_status(status); }
            unsafe {
                asm!("
                LUI $2, 0x8000
                .WORD {INSTRUCTION}
            ", INSTRUCTION = const INSTRUCTION, out("$2") _)
            };
            unsafe { cop0::set_status(Status::DEFAULT); }
            Ok(())
        })?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, INSTRUCTION, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, Cause::new().with_exception(exception).with_coprocessor_error(cop_index).raw_value(), "Cause")?;
        soft_assert_eq(exception_context.status, status.with_exl(true).raw_value(), "Status")?;
        soft_assert_eq(exception_context.fcsr, fcsr, format!("FCSR {}", desc).as_str())?;

        // Same test, this time within delay slot
        let exception_context = expect_exception(exception, 2, || {
            unsafe { cop0::set_status(status); }
            unsafe {
                asm!("
                .set noat
                .set noreorder
                LUI $2, 0x8000
                BEQ $0, $0, 2f
                .WORD {INSTRUCTION}
                2:
            ", INSTRUCTION = const INSTRUCTION, out("$2") _)
            };
            unsafe { cop0::set_status(Status::DEFAULT); }
            Ok(())
        })?;
        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector (delay)")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC (delay)")?;
        soft_assert_eq(unsafe { *((exception_context.exceptpc + 4) as *const u32) }, INSTRUCTION, "ExceptPC points to wrong instruction (delay)")?;
        soft_assert_eq(exception_context.cause, Cause::new().with_branch_delay(true).with_coprocessor_error(cop_index).with_exception(exception).raw_value(), "Cause (delay)")?;
        soft_assert_eq(exception_context.status, status.with_exl(true).raw_value(), "Status (delay)")?;
        soft_assert_eq(exception_context.fcsr, fcsr, "FCSR (delay)")?;
    }
    Ok(())
}

fn test_instruction_causes_unusable<const INSTRUCTION: u32>(cop_index: u2, usable: Status, unusable: Status) -> Result<(), String> {
    // Context, XContext, BadVAddr aren't affected by the exception. Remember their values so that
    // we can verify they aren't changed
    unsafe {
        set_context_64(0x01234567_ABCDEFFE);
        set_xcontext_64(0x01234567_ABCDEFFE);
    }
    let xcontext_before = xcontext_64();
    let context_before = context_64();
    let badvaddr_before = badvaddr();

    // Try calling instruction while the COP is enabled
    let mut temp: u64 = 0x01234567;
    unsafe { cop0::set_status(usable); }
    unsafe {
        asm!("
        .WORD {INSTRUCTION}
    ", INSTRUCTION = const INSTRUCTION, inout("$2") &mut temp => _)
    };

    // Try calling instruction while the COP is disabled
    let exception_context = expect_exception(CauseException::CpU, 1, || {
        // Set unusable within this block as Rust's function/closure stuff might also cause cop1 code
        unsafe { cop0::set_status(unusable); }
        unsafe {
            asm!("
                .WORD {INSTRUCTION}
            ", INSTRUCTION = const INSTRUCTION, inout("$2") &mut temp => _)
        };
        unsafe { cop0::set_status(Status::DEFAULT); }
        Ok(())
    })?;

    soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
    soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
    soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, INSTRUCTION, "ExceptPC points to wrong instruction")?;
    soft_assert_eq(exception_context.cause, Cause::new().with_exception(CauseException::CpU).with_coprocessor_error(cop_index).raw_value(), "Cause")?;
    soft_assert_eq(exception_context.status, unusable.with_exl(true).raw_value(), "Status")?;

    // Ensure the following weren't changed
    soft_assert_eq(exception_context.xcontext, xcontext_before, "XContext")?;
    soft_assert_eq(exception_context.context, context_before, "Context")?;
    soft_assert_eq(exception_context.badvaddr, badvaddr_before, "BadVAddr")?;

    // Call while it's illegal to call it, but in a delay slot
    let exception_context = expect_exception(CauseException::CpU, 2, || {
        // Set unusable within this block as Rust's function/closure stuff might also cause cop1 code
        unsafe { cop0::set_status(unusable); }
        unsafe {
            asm!("
                .set noat
                .set noreorder
                BEQ $0, $0, 2f
                .WORD {INSTRUCTION}
                2:
                ", INSTRUCTION = const INSTRUCTION, inout("$2") &mut temp => _)
        };
        unsafe { cop0::set_status(Status::DEFAULT); }
        Ok(())
    })?;

    soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
    soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
    soft_assert_eq(unsafe { *((exception_context.exceptpc + 4) as *const u32) }, INSTRUCTION, "ExceptPC points to wrong instruction")?;
    soft_assert_eq(exception_context.cause, Cause::new().with_exception(CauseException::CpU).with_coprocessor_error(cop_index).with_branch_delay(true).raw_value(), "Cause")?;
    soft_assert_eq(exception_context.status, unusable.with_exl(true).raw_value(), "Status")?;

    // Ensure the following weren't changed
    soft_assert_eq(exception_context.xcontext, xcontext_before, "XContext")?;
    soft_assert_eq(exception_context.context, context_before, "Context")?;
    soft_assert_eq(exception_context.badvaddr, badvaddr_before, "BadVAddr")?;

    Ok(())
}

fn test_cop1_instruction_causes_unusable<const INSTRUCTION: u32>() -> Result<(), String> {
    test_instruction_causes_unusable::<INSTRUCTION>(u2::new(1), Status::DEFAULT.with_cop1usable(true), Status::DEFAULT.with_cop1usable(false))
}

fn test_cop1_instruction_causes_fpe<const INSTRUCTION: u32>() -> Result<(), String> {
    // Set a bunch of unrelated FCSR flags to ensure they all get cleared
    for base_fcsr in [
        FCSR::new().with_cause_invalid_operation(true).with_cause_inexact_operation(true).with_cause_underflow(true).with_cause_overflow(true).with_cause_division_by_zero(true),
        FCSR::new().with_condition(true).with_flush_denorm_to_zero(true).with_invalid_operation(true).with_inexact_operation(true).with_underflow(true).with_overflow(true).with_division_by_zero(true)
    ] {
        set_fcsr(base_fcsr);
        // When an exception is fired, all cause flags are cleared and only the one that is being fired remains
        let expected_fcsr = base_fcsr.with_cause_invalid_operation(false).with_cause_inexact_operation(false).with_cause_underflow(false).with_cause_overflow(false).with_cause_division_by_zero(false).with_cause_unimplemented_operation(true);
        test_instruction_causes_exception::<INSTRUCTION>(
            u2::new(0), Status::DEFAULT.with_cop1usable(true), CauseException::FPE, expected_fcsr,
            u2::new(1), Status::DEFAULT.with_cop1usable(false), CauseException::CpU, expected_fcsr)?
    }
    Ok(())
}

fn test_cop2_instruction_causes_unusable<const INSTRUCTION: u32>() -> Result<(), String> {
    test_instruction_causes_unusable::<INSTRUCTION>(u2::new(2), Status::DEFAULT.with_cop2usable(true), Status::DEFAULT.with_cop2usable(false))
}

fn test_cop2_instruction_causes_ri<const INSTRUCTION: u32>() -> Result<(), String> {
    test_instruction_causes_exception::<INSTRUCTION>(
        u2::new(2),Status::DEFAULT.with_cop2usable(true), CauseException::RI, FCSR::DEFAULT,
        u2::new(2),Status::DEFAULT.with_cop2usable(false), CauseException::CpU, FCSR::DEFAULT)
}

fn test_cop3_instruction_causes_ri<const INSTRUCTION: u32>() -> Result<(), String> {
    // COP3 doesn't exist, so the cop index isn't set
    test_instruction_causes_exception::<INSTRUCTION>(
        u2::new(0),Status::DEFAULT.with_cop3usable(true), CauseException::RI, FCSR::DEFAULT,
        u2::new(0),Status::DEFAULT.with_cop3usable(false), CauseException::RI, FCSR::DEFAULT)
}

pub struct COP1UsableLWC1 {}

impl Test for COP1UsableLWC1 {
    fn name(&self) -> &str { "COP1Usable (LWC1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_lwc1(GPR::V0, 0, GPR::V0);
        test_cop1_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP1UsableLDC1 {}

impl Test for COP1UsableLDC1 {
    fn name(&self) -> &str { "COP1Usable (LDC1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_ldc1(GPR::V0, 0, GPR::V0);
        test_cop1_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP1UsableSWC1 {}

impl Test for COP1UsableSWC1 {
    fn name(&self) -> &str { "COP1Usable (SWC1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_swc1(GPR::V0, 0, GPR::V0);
        test_cop1_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP1UsableSDC1 {}

impl Test for COP1UsableSDC1 {
    fn name(&self) -> &str { "COP1Usable (SDC1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sdc1(GPR::V0, 0, GPR::V0);
        test_cop1_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP1UsableMFC1 {}

impl Test for COP1UsableMFC1 {
    fn name(&self) -> &str { "COP1Usable (MFC1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mfc1(GPR::V0, FR::F1);
        test_cop1_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP1UsableMTC1 {}

impl Test for COP1UsableMTC1 {
    fn name(&self) -> &str { "COP1Usable (MTC1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mtc1(GPR::V0, FR::F1);
        test_cop1_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP1UsableDMFC1 {}

impl Test for COP1UsableDMFC1 {
    fn name(&self) -> &str { "COP1Usable (DMFC1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_dmfc1(GPR::V0, FR::F1);
        test_cop1_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP1UsableDMTC1 {}

impl Test for COP1UsableDMTC1 {
    fn name(&self) -> &str { "COP1Usable (DMTC1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_dmtc1(GPR::V0, FR::F1);
        test_cop1_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP1UsableCFC1 {}

impl Test for COP1UsableCFC1 {
    fn name(&self) -> &str { "COP1Usable (CFC1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_cfc1(GPR::V0, u5::new(0));
        test_cop1_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP1UsableCTC1 {}

impl Test for COP1UsableCTC1 {
    fn name(&self) -> &str { "COP1Usable (CTC1)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_ctc1(GPR::V0, u5::new(0));
        test_cop1_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP1UsableDCFC1 {}

impl Test for COP1UsableDCFC1 {
    fn name(&self) -> &str { "COP1Usable (DCFC1)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_dcfc1(GPR::V0, u5::new(0));
        test_cop1_instruction_causes_fpe::<INSTRUCTION>()
    }
}

pub struct COP1UsableDCTC1 {}

impl Test for COP1UsableDCTC1 {
    fn name(&self) -> &str { "COP1Usable (DCTC1)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_dctc1(GPR::V0, u5::new(0));
        test_cop1_instruction_causes_fpe::<INSTRUCTION>()
    }
}

pub struct COP2UsableMFC2 {}

impl Test for COP2UsableMFC2 {
    fn name(&self) -> &str { "COP2Usable (MFC2)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mfc2(GPR::V0, u5::new(6));
        test_cop2_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP2UsableMTC2 {}

impl Test for COP2UsableMTC2 {
    fn name(&self) -> &str { "COP2Usable (MTC2)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mtc2(GPR::V0, u5::new(6));
        test_cop2_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP2UsableDMFC2 {}

impl Test for COP2UsableDMFC2 {
    fn name(&self) -> &str { "COP2Usable (DMFC2)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_dmfc2(GPR::V0, u5::new(6));
        test_cop2_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP2UsableDMTC2 {}

impl Test for COP2UsableDMTC2 {
    fn name(&self) -> &str { "COP2Usable (DMTC2)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_dmtc2(GPR::V0, u5::new(6));
        test_cop2_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP2UsableCFC2 {}

impl Test for COP2UsableCFC2 {
    fn name(&self) -> &str { "COP2Usable (CFC2)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_cfc2(GPR::V0, u5::new(6));
        test_cop2_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP2UsableCTC2 {}

impl Test for COP2UsableCTC2 {
    fn name(&self) -> &str { "COP2Usable (CTC2)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_ctc2(GPR::V0, u5::new(6));
        test_cop2_instruction_causes_unusable::<INSTRUCTION>()
    }
}

pub struct COP2UsableDCFC2 {}

impl Test for COP2UsableDCFC2 {
    fn name(&self) -> &str { "COP2Usable (DCFC2)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_dcfc2(GPR::V0, u5::new(6));
        test_cop2_instruction_causes_ri::<INSTRUCTION>()
    }
}

pub struct COP2UsableDCTC2 {}

impl Test for COP2UsableDCTC2 {
    fn name(&self) -> &str { "COP2Usable (DCTC2)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_dctc2(GPR::V0, u5::new(6));
        test_cop2_instruction_causes_ri::<INSTRUCTION>()
    }
}

pub struct COP3UsableMFC3 {}

impl Test for COP3UsableMFC3 {
    fn name(&self) -> &str { "COP3Usable (MFC3)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mfc3(GPR::V0, u5::new(6));
        test_cop3_instruction_causes_ri::<INSTRUCTION>()
    }
}

/// Tests what MFC2 actually does
pub struct COP2MFCBehavior {}

impl Test for COP2MFCBehavior {
    fn name(&self) -> &str { "MFC2/MTC2/DMFC2/DMTC2" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::DEFAULT.with_cop2usable(true)); }
        let value = 0x01234567_89ABCDEFu64;
        let mut result = [0u64; 8];
        unsafe {
            asm!("
                LD $2, 0($2)

                LUI $4, 0x123
                LUI $5, 0x456
                LUI $6, 0x789
                LUI $7, 0xABC
                LUI $8, 0xDEF
                LUI $9, 0
                LUI $10, 0
                LUI $11, 0

                MTC2 $4, $5
                MTC2 $5, $6
                MTC2 $7, $31

                LUI $4, 0
                LUI $5, 0
                LUI $6, 0
                LUI $7, 0
                LUI $8, 0

                LUI $4, 0x123
                LUI $5, 0x456
                LUI $6, 0x789
                LUI $7, 0xABC
                LUI $8, 0xDEF
                LUI $9, 0
                LUI $10, 0
                LUI $11, 0

                MTC2 $2, $5
                MFC2 $4, $6
                DMFC2 $5, $5
                MFC2 $6, $6
                DMFC2 $7, $31

                DMTC2 $2, $5
                MFC2 $8, $5
                DMFC2 $9, $5
                MFC2 $10, $31
                DMFC2 $11, $30

                SD $4, 0($3)
                SD $5, 8($3)
                SD $6, 16($3)
                SD $7, 24($3)
                SD $8, 32($3)
                SD $9, 40($3)
                SD $10, 48($3)
                SD $11, 56($3)
            ", inout("$2") &value => _, in("$3") &mut result[0], out("$4") _, out("$5") _, out("$6") _, out("$7") _, out("$8") _, out("$9") _, out("$10") _, out("$11") _)
        };

        soft_assert_eq(result[0], 0xFFFFFFFF_89ABCDEF, "MFC2 after MTC2")?;
        soft_assert_eq(result[1], 0x01234567_89ABCDEF, "DMFC2 after MTC2")?;
        soft_assert_eq(result[2], 0xFFFFFFFF_89ABCDEF, "MFC2 after MTC2 (different reg)")?;
        soft_assert_eq(result[3], 0x01234567_89ABCDEF, "DMFC2 after MTC2 (different reg)")?;

        soft_assert_eq(result[4], 0xFFFFFFFF_89ABCDEF, "MFC2 after DMTC2")?;
        soft_assert_eq(result[5], 0x01234567_89ABCDEF, "DMFC2 after DMTC2")?;
        soft_assert_eq(result[6], 0xFFFFFFFF_89ABCDEF, "MFC2 after DMTC2 (different reg)")?;
        soft_assert_eq(result[7], 0x01234567_89ABCDEF, "DMFC2 after DMTC2 (different reg)")?;

        Ok(())
    }
}

/// Tests what LWC2 actually does
/// This test is considered TooWeird because it is incomplete and because it is quite unclear
/// how things actually work:
/// - Doing SWC2 after LWC2 with the same register saves that value
/// - If the register is different, it will save the last load into that register,
///   but that can't quite be the case as it's unlikely that there are actually 32 registers (which
///   DMFC2 can't read)
pub struct COP2LWC2Behavior {}

impl Test for COP2LWC2Behavior {
    fn name(&self) -> &str { "LWC2/LDC2/SWC2/SDC2" }

    fn level(&self) -> Level { Level::TooWeird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::DEFAULT.with_cop2usable(true)); }
        let value = 0xFEDCBA98_76543210u64;
        let mut result = [0u64; 8];
        unsafe {
            asm!("
                LWC2 $9, 0($2)
                DMFC2 $4, $9
                SD $4, 0($3)

                LWC2 $9, 4($2)
                DMFC2 $4, $9
                SD $4, 8($3)

                // Note that saving $10 here will give weird result (see comment above)
                SDC2 $9, 16($3)
                SWC2 $9, 28($3)

                LDC2 $9, 0($2)
                DMFC2 $4, $9
                SD $4, 32($3)
                SDC2 $9, 40($3)
                SWC2 $9, 52($3)

                // Using a different register in SDC2 uses
            ", inout("$2") &value => _, in("$3") &mut result[0], out("$4") _, out("$5") _, out("$6") _, out("$7") _, out("$8") _, out("$9") _, out("$10") _, out("$11") _)
        };

        soft_assert_eq(result[0], 0xFEDCBA98_76543210, "DMFC2 after LWC2")?;
        soft_assert_eq(result[1], 0xFEDCBA98_76543210, "DMFC2 after LWC2 (offset +4)")?;
        soft_assert_eq(result[2], 0xFEDCBA98_76543210, "SDC2 after LWC2")?;
        soft_assert_eq(result[3], 0x00000000_76543210, "SWC2 after LWC2")?;

        soft_assert_eq(result[4], 0xFEDCBA98_76543210, "DMFC2 after LDC2")?;
        soft_assert_eq(result[5], 0xFEDCBA98_76543210, "SDC2 after LDC2")?;
        soft_assert_eq(result[6], 0x00000000_76543210, "SWC2 after LDC2")?;

        Ok(())
    }
}

