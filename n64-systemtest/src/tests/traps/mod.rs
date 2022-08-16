pub mod delay;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use arbitrary_int::u5;
use crate::assembler::{Assembler, RegimmOpcode, SpecialOpcode};
use crate::cop0::CauseException;
use crate::exception_handler::expect_exception;

use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

#[repr(u16)]
#[derive(Debug)]
pub enum Immediate {
    Zero = 0u16,
    Two = 2u16,
    MinusTwo = 0xFFFE,
}

fn trap<const INSTRUCTION: u32>(v1: u64, v2: u64) {
    unsafe {
        asm!("
            .set noat
            .set noreorder
            dsll32 $2, {gpr_hi_1}, 0
            dsll32 $3, {gpr_hi_2}, 0
            // Zero extend gpr_lo
            dsll32 {tmp1b}, {gpr_lo_1}, 0
            dsll32 {tmp2b}, {gpr_lo_2}, 0
            dsrl32 {tmp1b}, {tmp1b}, 0
            dsrl32 {tmp2b}, {tmp2b}, 0
            or $2, $2, {tmp1b}
            or $3, $3, {tmp2b}
            .word {INSTRUCTION}
        ", gpr_lo_1 = in(reg) (v1 as u32), gpr_hi_1 = in(reg) ((v1 >> 32) as u32),
        gpr_lo_2 = in(reg) (v2 as u32), gpr_hi_2 = in(reg) ((v2 >> 32) as u32),
        INSTRUCTION = const INSTRUCTION,
        tmp1b = out(reg) _, tmp2b = out(reg) _,
        out("$2") _, out("$3") _)
    }
}

fn test_trap<const INSTRUCTION: u32>(v1: u64, v2: u64, expect_trap: bool) -> Result<(), String> {
    if expect_trap {
        let exception_context = expect_exception(CauseException::Tr, 1, || {
            trap::<INSTRUCTION>(v1, v2);
            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, INSTRUCTION, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x34, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;
    } else {
        trap::<INSTRUCTION>(v1, v2);
    }
    Ok(())
}

fn trap_imm<const INSTRUCTION: u32>(v1: u64) {
    unsafe {
        asm!("
            .set noat
            .set noreorder
            dsll32 $2, {gpr_hi_1}, 0
            // Zero extend gpr_lo
            dsll32 {tmp1b}, {gpr_lo_1}, 0
            dsrl32 {tmp1b}, {tmp1b}, 0
            or $2, $2, {tmp1b}
            .word {INSTRUCTION}
        ", gpr_lo_1 = in(reg) (v1 as u32), gpr_hi_1 = in(reg) ((v1 >> 32) as u32),
        INSTRUCTION = const INSTRUCTION,
        tmp1b = out(reg) _,
        out("$2") _)
    }
}

fn test_trap_imm<const INSTRUCTION: u32>(v1: u64, expect_trap: bool) -> Result<(), String> {
    if expect_trap {
        let exception_context = expect_exception(CauseException::Tr, 1, || {
            trap_imm::<INSTRUCTION>(v1);
            Ok(())
        })?;

        soft_assert_eq(exception_context.k0_exception_vector, 0xFFFFFFFF_80000180, "Exception Vector")?;
        soft_assert_eq(exception_context.exceptpc & 0xFFFFFFFF_FF000000, 0xFFFFFFFF_80000000, "ExceptPC")?;
        soft_assert_eq(unsafe { *(exception_context.exceptpc as *const u32) }, INSTRUCTION, "ExceptPC points to wrong instruction")?;
        soft_assert_eq(exception_context.cause, 0x34, "Cause")?;
        soft_assert_eq(exception_context.status, 0x24000002, "Status")?;
    } else {
        trap_imm::<INSTRUCTION>(v1);
    }
    Ok(())
}

fn execute_test_case_2_registers<const INSTRUCTION: u32>(value: &Box<dyn Any>) -> Result<(), String> {
    match value.downcast_ref::<(bool, u64, u64)>() {
        Some((exception_expected, v1, v2)) => {
            test_trap::<INSTRUCTION>((*v1) as u64, (*v2) as u64, *exception_expected)?;

            Ok(())
        }
        _ => {
            Err("(bool, u64, u64) expected".to_string())
        }
    }
}

fn execute_test_case_register_with_immediate<
    const INSTRUCTION_MINUS2: u32,
    const INSTRUCTION_0: u32,
    const INSTRUCTION_2: u32>(value: &Box<dyn Any>) -> Result<(), String> {

    match value.downcast_ref::<(bool, u64, Immediate)>() {
        Some((exception_expected, v1, imm)) => {
            match *imm {
                Immediate::Two => {
                    test_trap_imm::<INSTRUCTION_2>(*v1, *exception_expected)?;
                }
                Immediate::Zero => {
                    test_trap_imm::<INSTRUCTION_0>(*v1, *exception_expected)?;
                }
                Immediate::MinusTwo => {
                    test_trap_imm::<INSTRUCTION_MINUS2>(*v1, *exception_expected)?;
                }
            }

            Ok(())
        }
        _ => {
            Err("(bool, u64, Immediate) expected".to_string())
        }
    }
}

pub struct TLT {}

impl Test for TLT {
    fn name(&self) -> &str { "TLT" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0x0000_0000_0000_0095u64, 0x0000_0000_0000_0096u64)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FFFFu64, 0x0000_0000_0000_0000u64)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FF00u64, 0xFFFF_FFFF_FFFF_FF01u64)),
            Box::new((true, 0x0000_0095_0000_0096u64, 0x0000_0096_0000_0095u64)),
            Box::new((true, 0xFFFF_FFFF_0000_0000u64, 0x0000_0000_0000_0000u64)),
            Box::new((true, 0xFFFF_FF00_0000_0010u64, 0xFFFF_FF01_0000_0009u64)),

            Box::new((false, 0xFFFF_FFFF_0000_0000u64, 0xFFFF_FFFF_0000_0000u64)),
            Box::new((false, 0x0000_0000_FFFF_FFFFu64, 0x0000_0000_FFFF_FFFFu64)),
            Box::new((false, 0xBADDECAF15C0FFEEu64, 0xBADDECAF15C0FFEEu64)),

            Box::new((false, 0x0000_0000_0000_0096u64, 0x0000_0000_0000_0095u64)),
            Box::new((false, 0x0000_0000_0000_0000u64, 0xFFFF_FFFF_FFFF_FFFFu64)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FF01u64, 0xFFFF_FFFF_FFFF_FF00u64)),
            Box::new((false, 0x0000_0096_0000_0095u64, 0x0000_0095_0000_0096u64)),
            Box::new((false, 0x0000_0000_0000_0000u64, 0xFFFF_FFFF_0000_0000u64)),
            Box::new((false, 0xFFFF_FF01_0000_0009u64, 0xFFFF_FF00_0000_0010u64)),

            Box::new((true, 0xFFFF_FFFF_0000_0000u64, 0xFFFF_FFFF_F000_0000u64)),
            Box::new((false, 0xFFFF_FFFF_F000_0000u64, 0xFFFF_FFFF_0000_0000u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_special(SpecialOpcode::TLT, u5::new(0), u5::new(0), u5::new(2), u5::new(3));
        execute_test_case_2_registers::<INSTRUCTION>(value)
    }
}

pub struct TLTU {}

impl Test for TLTU {
    fn name(&self) -> &str { "TLTU" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, 0x0000_0000_0000_0000u64, 0x0000_0000_0000_0000u64)),

            Box::new((true, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_0101u64)),
            Box::new((false, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_0100u64)),
            Box::new((false, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_00FFu64)),

            Box::new((true, 0x0000_0100_0000_0000u64, 0x0000_0101_0000_0000u64)),
            Box::new((false, 0x0000_0100_0000_0000u64, 0x0000_0100_0000_0000u64)),
            Box::new((false, 0x0000_0100_0000_0000u64, 0x0000_00FF_0000_0000u64)),

            Box::new((true, 0x0000_0100_0000_0100u64, 0x0000_0101_0000_00FFu64)),
            Box::new((false, 0x0000_0100_0000_0100u64, 0x0000_0100_0000_0100u64)),
            Box::new((false, 0x0000_0100_0000_0100u64, 0x0000_00FF_0000_0101u64)),

            Box::new((false, 0xFFFF_FFFF_0000_0122u64, 0x0FFF_FFFF_0000_0123u64)),
            Box::new((true, 0xFFFF_FFFF_0000_0000u64, 0xFFFF_FFFF_F000_0000u64)),
            Box::new((false, 0xFFFF_FFFF_F000_0000u64, 0xFFFF_FFFF_0000_0000u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_special(SpecialOpcode::TLTU, u5::new(0), u5::new(0), u5::new(2), u5::new(3));
        execute_test_case_2_registers::<INSTRUCTION>(value)
    }
}

pub struct TGE {}

impl Test for TGE {
    fn name(&self) -> &str { "TGE" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, 0x0000_0000_0000_0095u64, 0x0000_0000_0000_0096u64)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FFFFu64, 0x0000_0000_0000_0000u64)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FF00u64, 0xFFFF_FFFF_FFFF_FF01u64)),
            Box::new((false, 0x0000_0095_0000_0096u64, 0x0000_0096_0000_0095u64)),
            Box::new((false, 0xFFFF_FFFF_0000_0000u64, 0x0000_0000_0000_0000u64)),
            Box::new((false, 0xFFFF_FF00_0000_0010u64, 0xFFFF_FF01_0000_0009u64)),

            Box::new((true, 0xFFFF_FFFF_0000_0000u64, 0xFFFF_FFFF_0000_0000u64)),
            Box::new((true, 0x0000_0000_FFFF_FFFFu64, 0x0000_0000_FFFF_FFFFu64)),
            Box::new((true, 0xBADDECAF15C0FFEEu64, 0xBADDECAF15C0FFEEu64)),

            Box::new((true, 0x0000_0000_0000_0096u64, 0x0000_0000_0000_0095u64)),
            Box::new((true, 0x0000_0000_0000_0000u64, 0xFFFF_FFFF_FFFF_FFFFu64)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FF01u64, 0xFFFF_FFFF_FFFF_FF00u64)),
            Box::new((true, 0x0000_0096_0000_0095u64, 0x0000_0095_0000_0096u64)),
            Box::new((true, 0x0000_0000_0000_0000u64, 0xFFFF_FFFF_0000_0000u64)),
            Box::new((true, 0xFFFF_FF01_0000_0009u64, 0xFFFF_FF00_0000_0010u64)),

            Box::new((false, 0xFFFF_FFFF_0000_0000u64, 0xFFFF_FFFF_F000_0000u64)),
            Box::new((true, 0xFFFF_FFFF_F000_0000u64, 0xFFFF_FFFF_0000_0000u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_special(SpecialOpcode::TGE, u5::new(0), u5::new(0), u5::new(2), u5::new(3));
        execute_test_case_2_registers::<INSTRUCTION>(value)
    }
}

pub struct TGEU {}

impl Test for TGEU {
    fn name(&self) -> &str { "TGEU" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0x0000_0000_0000_0000u64, 0x0000_0000_0000_0000u64)),

            Box::new((false, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_0101u64)),
            Box::new((true, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_0100u64)),
            Box::new((true, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_00FFu64)),

            Box::new((false, 0x0000_0100_0000_0000u64, 0x0000_0101_0000_0000u64)),
            Box::new((true, 0x0000_0100_0000_0000u64, 0x0000_0100_0000_0000u64)),
            Box::new((true, 0x0000_0100_0000_0000u64, 0x0000_00FF_0000_0000u64)),

            Box::new((false, 0x0000_0100_0000_0100u64, 0x0000_0101_0000_00FFu64)),
            Box::new((true, 0x0000_0100_0000_0100u64, 0x0000_0100_0000_0100u64)),
            Box::new((true, 0x0000_0100_0000_0100u64, 0x0000_00FF_0000_0101u64)),

            Box::new((false, 0x0FFF_FFFF_0000_0123u64, 0xFFFF_FFFF_0000_0122u64)),
            Box::new((false, 0xFFFF_FFFF_0000_0000u64, 0xFFFF_FFFF_F000_0000u64)),
            Box::new((true, 0xFFFF_FFFF_F000_0000u64, 0xFFFF_FFFF_0000_0000u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_special(SpecialOpcode::TGEU, u5::new(0), u5::new(0), u5::new(2), u5::new(3));
        execute_test_case_2_registers::<INSTRUCTION>(value)
    }
}

pub struct TEQ {}

impl Test for TEQ {
    fn name(&self) -> &str { "TEQ" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0x0000_0000_0000_0000u64, 0x0000_0000_0000_0000u64)),

            Box::new((false, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_0101u64)),
            Box::new((true, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_0100u64)),
            Box::new((false, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_00FFu64)),

            Box::new((false, 0x0000_0100_0000_0000u64, 0x0000_0101_0000_0000u64)),
            Box::new((true, 0x0000_0100_0000_0000u64, 0x0000_0100_0000_0000u64)),
            Box::new((false, 0x0000_0100_0000_0000u64, 0x0000_00FF_0000_0000u64)),

            Box::new((false, 0x0000_0100_0000_0100u64, 0x0000_0101_0000_00FFu64)),
            Box::new((true, 0x0000_0100_0000_0100u64, 0x0000_0100_0000_0100u64)),
            Box::new((false, 0x0000_0100_0000_0100u64, 0x0000_00FF_0000_0101u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_special(SpecialOpcode::TEQ, u5::new(0), u5::new(0), u5::new(2), u5::new(3));
        execute_test_case_2_registers::<INSTRUCTION>(value)
    }
}

pub struct TNE {}

impl Test for TNE {
    fn name(&self) -> &str { "TNE" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, 0x0000_0000_0000_0000u64, 0x0000_0000_0000_0000u64)),

            Box::new((true, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_0101u64)),
            Box::new((false, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_0100u64)),
            Box::new((true, 0x0000_0000_0000_0100u64, 0x0000_0000_0000_00FFu64)),

            Box::new((true, 0x0000_0100_0000_0000u64, 0x0000_0101_0000_0000u64)),
            Box::new((false, 0x0000_0100_0000_0000u64, 0x0000_0100_0000_0000u64)),
            Box::new((true, 0x0000_0100_0000_0000u64, 0x0000_00FF_0000_0000u64)),

            Box::new((true, 0x0000_0100_0000_0100u64, 0x0000_0101_0000_00FFu64)),
            Box::new((false, 0x0000_0100_0000_0100u64, 0x0000_0100_0000_0100u64)),
            Box::new((true, 0x0000_0100_0000_0100u64, 0x0000_00FF_0000_0101u64)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_special(SpecialOpcode::TNE, u5::new(0), u5::new(0), u5::new(2), u5::new(3));
        execute_test_case_2_registers::<INSTRUCTION>(value)
    }
}

pub struct TEQI {}

impl Test for TEQI {
    fn name(&self) -> &str { "TEQI" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0x0000_0000_0000_0000u64, Immediate::Zero)),
            Box::new((false, 0x0000_0000_0000_0001u64, Immediate::Zero)),
            Box::new((false, 0x0000_0000_0001_0000u64, Immediate::Zero)),
            Box::new((false, 0x0000_0001_0000_0000u64, Immediate::Zero)),

            Box::new((true, 0x0000_0000_0000_0002u64, Immediate::Two)),
            Box::new((false, 0x0000_0000_0001_0002u64, Immediate::Two)),
            Box::new((false, 0x0000_0001_0000_0002u64, Immediate::Two)),

            Box::new((true, 0xFFFF_FFFF_FFFF_FFFEu64, Immediate::MinusTwo)),
            Box::new((false, 0xFFFF_FFF0_FFFF_FFFEu64, Immediate::MinusTwo)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FFF0u64, Immediate::MinusTwo)),
            Box::new((false, 0xFFFF_FFFF_FFF0_FFFEu64, Immediate::MinusTwo)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const I2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TEQI, u5::new(2), 2);
        const I0: u32 = Assembler::make_regimm_trap(RegimmOpcode::TEQI, u5::new(2), 0);
        const INEG2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TEQI, u5::new(2), 0xFFFE);

        execute_test_case_register_with_immediate::<INEG2, I0, I2>(value)
    }
}

pub struct TNEI {}

impl Test for TNEI {
    fn name(&self) -> &str { "TNEI" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, 0x0000_0000_0000_0000u64, Immediate::Zero)),
            Box::new((true, 0x0000_0000_0000_0001u64, Immediate::Zero)),
            Box::new((true, 0x0000_0000_0001_0000u64, Immediate::Zero)),
            Box::new((true, 0x0000_0001_0000_0000u64, Immediate::Zero)),

            Box::new((false, 0x0000_0000_0000_0002u64, Immediate::Two)),
            Box::new((true, 0x0000_0000_0001_0002u64, Immediate::Two)),
            Box::new((true, 0x0000_0001_0000_0002u64, Immediate::Two)),

            Box::new((false, 0xFFFF_FFFF_FFFF_FFFEu64, Immediate::MinusTwo)),
            Box::new((true, 0xFFFF_FFF0_FFFF_FFFEu64, Immediate::MinusTwo)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FFF0u64, Immediate::MinusTwo)),
            Box::new((true, 0xFFFF_FFFF_FFF0_FFFEu64, Immediate::MinusTwo)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const I2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TNEI, u5::new(2), 2);
        const I0: u32 = Assembler::make_regimm_trap(RegimmOpcode::TNEI, u5::new(2), 0);
        const INEG2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TNEI, u5::new(2), 0xFFFE);

        execute_test_case_register_with_immediate::<INEG2, I0, I2>(value)
    }
}

pub struct TGEI {}

impl Test for TGEI {
    fn name(&self) -> &str { "TGEI" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0x0000_0000_0000_0000u64, Immediate::Zero)),
            Box::new((true, 0x0000_0000_0000_0001u64, Immediate::Zero)),
            Box::new((true, 0x0000_0000_0001_0000u64, Immediate::Zero)),
            Box::new((true, 0x0000_0001_0000_0000u64, Immediate::Zero)),
            Box::new((true, 0x0000_0000_FFFF_FFFFu64, Immediate::Zero)),
            Box::new((false, 0x8000_0000_0000_0000u64, Immediate::Zero)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FFFFu64, Immediate::Zero)),

            Box::new((true, 0x0000_0000_0000_0002u64, Immediate::Two)),
            Box::new((true, 0x0000_0000_0000_0003u64, Immediate::Two)),
            Box::new((true, 0x0000_0000_FFFF_FFFFu64, Immediate::Two)),
            Box::new((true, 0x7FFF_FFFF_FFFF_FFFFu64, Immediate::Two)),
            Box::new((false, 0x0000_0000_0000_0001u64, Immediate::Two)),
            Box::new((false, 0x8000_0000_0000_0000u64, Immediate::Two)),
            Box::new((false, 0xFFFF_FFFF_0000_0002u64, Immediate::Two)),

            Box::new((true, 0xFFFF_FFFF_FFFF_FFFEu64, Immediate::MinusTwo)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FFFFu64, Immediate::MinusTwo)),
            Box::new((true, 0x0000_0000_0000_0000u64, Immediate::MinusTwo)),
            Box::new((true, 0x0000_0000_FFFF_FFFDu64, Immediate::MinusTwo)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FFFDu64, Immediate::MinusTwo)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FFFCu64, Immediate::MinusTwo)),
            Box::new((false, 0xFFFF_FFFF_0000_0000u64, Immediate::MinusTwo)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const I2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TGEI, u5::new(2), 2);
        const I0: u32 = Assembler::make_regimm_trap(RegimmOpcode::TGEI, u5::new(2), 0);
        const INEG2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TGEI, u5::new(2), 0xFFFE);

        execute_test_case_register_with_immediate::<INEG2, I0, I2>(value)
    }
}

pub struct TGEIU {}

impl Test for TGEIU {
    fn name(&self) -> &str { "TGEIU" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((true, 0x0000_0000_0000_0000u64, Immediate::Zero)),
            Box::new((true, 0x0000_0000_0000_0001u64, Immediate::Zero)),
            Box::new((true, 0x0000_0000_0001_0000u64, Immediate::Zero)),
            Box::new((true, 0x0000_0001_0000_0000u64, Immediate::Zero)),
            Box::new((true, 0x0000_0000_FFFF_FFFFu64, Immediate::Zero)),
            Box::new((true, 0x8000_0000_0000_0000u64, Immediate::Zero)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FFFFu64, Immediate::Zero)),

            Box::new((false, 0x0000_0000_0000_0000u64, Immediate::Two)),
            Box::new((false, 0x0000_0000_0000_0001u64, Immediate::Two)),
            Box::new((true, 0x0000_0000_0000_0002u64, Immediate::Two)),
            Box::new((true, 0x0000_0000_0000_0003u64, Immediate::Two)),
            Box::new((true, 0x0000_0000_FFFF_FFFFu64, Immediate::Two)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FFFFu64, Immediate::Two)),
            Box::new((true, 0xFFFF_FFFF_0000_0000u64, Immediate::Two)),

            Box::new((true, 0xFFFF_FFFF_FFFF_FFFFu64, Immediate::MinusTwo)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FFFEu64, Immediate::MinusTwo)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FFFDu64, Immediate::MinusTwo)),
            Box::new((false, 0x0000_0000_0000_0000u64, Immediate::MinusTwo)),
            Box::new((false, 0x0000_0000_FFFF_FFFFu64, Immediate::MinusTwo)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const I2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TGEIU, u5::new(2), 2);
        const I0: u32 = Assembler::make_regimm_trap(RegimmOpcode::TGEIU, u5::new(2), 0);
        const INEG2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TGEIU, u5::new(2), 0xFFFE);

        execute_test_case_register_with_immediate::<INEG2, I0, I2>(value)
    }
}

pub struct TLTI {}

impl Test for TLTI {
    fn name(&self) -> &str { "TLTI" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, 0x0000_0000_0000_0000u64, Immediate::Zero)),
            Box::new((false, 0x0000_0000_0000_0001u64, Immediate::Zero)),
            Box::new((false, 0x0000_0000_0001_0000u64, Immediate::Zero)),
            Box::new((false, 0x0000_0001_0000_0000u64, Immediate::Zero)),
            Box::new((false, 0x0000_0000_FFFF_FFFFu64, Immediate::Zero)),
            Box::new((true, 0x8000_0000_0000_0000u64, Immediate::Zero)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FFFFu64, Immediate::Zero)),

            Box::new((false, 0x0000_0000_0000_0002u64, Immediate::Two)),
            Box::new((false, 0x0000_0000_0000_0003u64, Immediate::Two)),
            Box::new((false, 0x0000_0000_FFFF_FFFFu64, Immediate::Two)),
            Box::new((false, 0x7FFF_FFFF_FFFF_FFFFu64, Immediate::Two)),
            Box::new((true, 0x0000_0000_0000_0001u64, Immediate::Two)),
            Box::new((true, 0x8000_0000_0000_0000u64, Immediate::Two)),
            Box::new((true, 0xFFFF_FFFF_0000_0002u64, Immediate::Two)),

            Box::new((false, 0xFFFF_FFFF_FFFF_FFFEu64, Immediate::MinusTwo)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FFFFu64, Immediate::MinusTwo)),
            Box::new((false, 0x0000_0000_0000_0000u64, Immediate::MinusTwo)),
            Box::new((false, 0x0000_0000_FFFF_FFFDu64, Immediate::MinusTwo)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FFFDu64, Immediate::MinusTwo)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FFFCu64, Immediate::MinusTwo)),
            Box::new((true, 0xFFFF_FFFF_0000_0000u64, Immediate::MinusTwo)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const I2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TLTI, u5::new(2), 2);
        const I0: u32 = Assembler::make_regimm_trap(RegimmOpcode::TLTI, u5::new(2), 0);
        const INEG2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TLTI, u5::new(2), 0xFFFE);

        execute_test_case_register_with_immediate::<INEG2, I0, I2>(value)
    }
}

pub struct TLTIU {}

impl Test for TLTIU {
    fn name(&self) -> &str { "TLTIU" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new((false, 0x0000_0000_0000_0000u64, Immediate::Zero)),
            Box::new((false, 0x0000_0000_0000_0001u64, Immediate::Zero)),
            Box::new((false, 0x0000_0000_0001_0000u64, Immediate::Zero)),
            Box::new((false, 0x0000_0001_0000_0000u64, Immediate::Zero)),
            Box::new((false, 0x0000_0000_FFFF_FFFFu64, Immediate::Zero)),
            Box::new((false, 0x8000_0000_0000_0000u64, Immediate::Zero)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FFFFu64, Immediate::Zero)),

            Box::new((true, 0x0000_0000_0000_0000u64, Immediate::Two)),
            Box::new((true, 0x0000_0000_0000_0001u64, Immediate::Two)),
            Box::new((false, 0x0000_0000_0000_0002u64, Immediate::Two)),
            Box::new((false, 0x0000_0000_0000_0003u64, Immediate::Two)),
            Box::new((false, 0x0000_0000_FFFF_FFFFu64, Immediate::Two)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FFFFu64, Immediate::Two)),
            Box::new((false, 0xFFFF_FFFF_0000_0000u64, Immediate::Two)),

            Box::new((false, 0xFFFF_FFFF_FFFF_FFFFu64, Immediate::MinusTwo)),
            Box::new((false, 0xFFFF_FFFF_FFFF_FFFEu64, Immediate::MinusTwo)),
            Box::new((true, 0xFFFF_FFFF_FFFF_FFFDu64, Immediate::MinusTwo)),
            Box::new((true, 0x0000_0000_0000_0000u64, Immediate::MinusTwo)),
            Box::new((true, 0x0000_0000_FFFF_FFFFu64, Immediate::MinusTwo)),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        const I2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TLTIU, u5::new(2), 2);
        const I0: u32 = Assembler::make_regimm_trap(RegimmOpcode::TLTIU, u5::new(2), 0);
        const INEG2: u32 = Assembler::make_regimm_trap(RegimmOpcode::TLTIU, u5::new(2), 0xFFFE);

        execute_test_case_register_with_immediate::<INEG2, I0, I2>(value)
    }
}
