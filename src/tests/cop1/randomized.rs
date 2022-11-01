use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use core::mem::transmute;
use arbitrary_int::u2;
use oorandom::{Rand32, Rand64};
use crate::assembler::{Assembler, FR};
use crate::cop0::{CauseException};
use crate::cop1::{FCSR, fcsr, FCSRRoundingMode, set_fcsr};
use crate::exception_handler::expect_exception;
use crate::graphics::color::{Color, RGBA5551};
use crate::graphics::cursor::Cursor;
use crate::graphics::font::Font;
use crate::graphics::system_font::FONT_GENEVA_9;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;
use crate::VIDEO;

// The tests in here perform a lot of randomized calculations and hash the output/exception flags
// The final result is compared. If there is a mismatch, finding the culprit will probably require
// to print out some temp values in randomized_test.
// These tests rely on the deterministic nature of oorandom

fn randomized_test<FLOAT, INT: From<u32> + Into<u64>, FPERFORM: FnMut(u32) -> INT>(name: &str, progress_indicator: bool, iterations: u32, expected: u64, mut perform: FPERFORM) -> Result<(), String>  {
    let mut hash = 0u64;

    let font = Font::from_data(&FONT_GENEVA_9).unwrap();
    let mut cursor = Cursor::new_with_font(&font, RGBA5551::BLACK);

    for i in 0..iterations {
        let mut result: INT = i.into();  // in case of exception this won't be written to
        let mut result_fcsr = FCSR::new();

        // Disable exception firing, but toggle flush denorm to zero
        let ftz = (i & 1) != 0;
        let rounding_mode = FCSRRoundingMode::new_with_raw_value(u2::extract_u32(i, 1));
        set_fcsr(FCSR::new().with_flush_denorm_to_zero(ftz).with_rounding_mode(rounding_mode));
        let maybe_exception = expect_exception(CauseException::FPE, 1, || {
            result = perform(i);
            result_fcsr = fcsr();

            Ok(())
        });

        if let Ok(exception) = maybe_exception {
            result_fcsr = exception.fcsr;
        }

        hash = hash * 397 ^ result.into();
        hash = hash * 397 ^ (result_fcsr.raw_value() as u64);

        if progress_indicator {
            if (i & 65535) == 0 {
                let v = VIDEO.lock();
                {
                    let mut lock = v.framebuffers().backbuffer().lock();
                    let buffer = lock.as_mut().unwrap();
                    buffer.clear_with_color(RGBA5551::WHITE);

                    cursor.x = 16;
                    cursor.y = 16;
                    cursor.draw_text(buffer, format!("Stress testing {}. {}% complete", name, i * 100 / iterations).as_str());
                }
                v.swap_buffers();
            }
        }
    }

    soft_assert_eq(hash, expected, "Hash")?;

    Ok(())
}

fn randomized_test32<const FINSTRUCTION: u32>(name: &str, progress_indicator: bool, iterations: u32, expected: u64) -> Result<(), String> {
    let mut random = Rand32::new(0);

    /// Returns a random float with an equal distribution of its bits (so this includes NAN, subnormal etc)
    fn random_float(random: &mut Rand32) -> f32 {
        unsafe { transmute(random.rand_u32()) }
    }

    randomized_test::<f32, u32, _>(
        name, progress_indicator, iterations, expected,
        |_iteration| {
            let f1 = random_float(&mut random);
            let f2 = random_float(&mut random);
            unsafe {
                let float_result: f32;
                asm!("
                    .set noat
                    .set noreorder
                    .word {FINSTRUCTION}
                    nop
                ",
                FINSTRUCTION = const FINSTRUCTION,
                in("$f0") f1,
                in("$f2") f2,
                out("$f4") float_result,
                options(nostack, nomem));

                transmute(float_result)
            }
        })
}

fn randomized_test64<const FINSTRUCTION: u32>(name: &str, progress_indicator: bool, iterations: u32, expected: u64) -> Result<(), String> {
    let mut random = Rand64::new(0);

    /// Returns a random float with an equal distribution of its bits (so this includes NAN, subnormal etc)
    fn random_float(random: &mut Rand64) -> f64 {
        unsafe { transmute(random.rand_u64()) }
    }

    randomized_test::<f64, u64, _>(
        name,
        progress_indicator, iterations, expected,
        |_iteration| {
            let f1 = random_float(&mut random);
            let f2 = random_float(&mut random);
            unsafe {
                let float_result: f64;
                asm!("
                    .set noat
                    .set noreorder
                    .word {FINSTRUCTION}
                    nop
                ",
                FINSTRUCTION = const FINSTRUCTION,
                in("$f0") f1,
                in("$f2") f2,
                out("$f4") float_result,
                options(nostack, nomem));

                transmute(float_result)
            }
        })
}

pub struct AddS;

impl Test for AddS {
    fn name(&self) -> &str { "ADD.S (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_add(FR::F4, FR::F0, FR::F2).s();
        randomized_test32::<INSTRUCTION>("ADD.S", false, 3000, 0x341aa5e3310b4ab0)
    }
}

pub struct AddD;

impl Test for AddD {
    fn name(&self) -> &str { "ADD.D (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_add(FR::F4, FR::F0, FR::F2).d();
        randomized_test64::<INSTRUCTION>("ADD.D", false, 3000, 0xe888e60dc53b5a03)
    }
}

pub struct SubS;

impl Test for SubS {
    fn name(&self) -> &str { "SUB.S (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sub(FR::F4, FR::F0, FR::F2).s();
        randomized_test32::<INSTRUCTION>("SUB.S", false, 3000, 0xa56dbeaf17cd7173)
    }
}

pub struct SubD;

impl Test for SubD {
    fn name(&self) -> &str { "SUB.D (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sub(FR::F4, FR::F0, FR::F2).d();
        randomized_test64::<INSTRUCTION>("SUB.D", false, 3000, 0x79d3b74f1bc3ff24)
    }
}

pub struct MulS;

impl Test for MulS {
    fn name(&self) -> &str { "MUL.S (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mul(FR::F4, FR::F0, FR::F2).s();
        randomized_test32::<INSTRUCTION>("MUL.S", false, 2000, 0x170788e9b03e2c0a)
    }
}

pub struct MulD;

impl Test for MulD {
    fn name(&self) -> &str { "MUL.D (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mul(FR::F4, FR::F0, FR::F2).d();
        randomized_test64::<INSTRUCTION>("MUL.D", false, 2000, 0x15e6d76853f4df59)
    }
}

pub struct DivS;

impl Test for DivS {
    fn name(&self) -> &str { "DIV.S (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_div(FR::F4, FR::F0, FR::F2).s();
        randomized_test32::<INSTRUCTION>("DIV.S", false, 2000, 0x55c6839600fd0fa2)
    }
}

pub struct DivD;

impl Test for DivD {
    fn name(&self) -> &str { "DIV.D (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_div(FR::F4, FR::F0, FR::F2).d();
        randomized_test64::<INSTRUCTION>("DIV.D",false, 2000, 0xa2ae9ba9a3fec554)
    }
}

pub struct SqrtS;

impl Test for SqrtS {
    fn name(&self) -> &str { "SQRT.S (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sqrt(FR::F4, FR::F0).s();
        randomized_test32::<INSTRUCTION>("SQRT.S", false, 2000, 0x276c42a0e3a11a1)
    }
}

pub struct SqrtD;

impl Test for SqrtD {
    fn name(&self) -> &str { "SQRT.D (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sqrt(FR::F4, FR::F0).d();
        randomized_test64::<INSTRUCTION>("SQRT.D", false, 2000, 0x6cb8986281efca80)
    }
}

pub struct CvtSFromW;

impl Test for CvtSFromW {
    fn name(&self) -> &str { "CVT.S.W (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_cvt_s(FR::F4, FR::F0).w();
        randomized_test32::<INSTRUCTION>("CVT.S.W", false, 2000, 0x8ac09585da537355)
    }
}

pub struct CvtWFromS;

impl Test for CvtWFromS {
    fn name(&self) -> &str { "CVT.W.S (randomized - quick)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_cvt_w(FR::F4, FR::F0).s();
        randomized_test32::<INSTRUCTION>("CVT.W.S", false, 2000, 0x41ca013057f72659)
    }
}

pub struct StresstestAddS;

impl Test for StresstestAddS {
    fn name(&self) -> &str { "ADD.S (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_add(FR::F4, FR::F0, FR::F2).s();
        randomized_test32::<INSTRUCTION>("1/10 ADD.S", true, 3000000, 0x464cfdae544a9be0)
    }
}

pub struct StresstestAddD;

impl Test for StresstestAddD {
    fn name(&self) -> &str { "ADD.D (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_add(FR::F4, FR::F0, FR::F2).d();
        randomized_test64::<INSTRUCTION>("2/10 ADD.D", true, 3000000, 0x25d261c14617031e)
    }
}

pub struct StresstestSubS;

impl Test for StresstestSubS {
    fn name(&self) -> &str { "SUB.S (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sub(FR::F4, FR::F0, FR::F2).s();
        randomized_test32::<INSTRUCTION>("3/10 SUB.S", true, 3000000, 0x5906d66d6ed35481)
    }
}

pub struct StresstestSubD;

impl Test for StresstestSubD {
    fn name(&self) -> &str { "SUB.D (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sub(FR::F4, FR::F0, FR::F2).d();
        randomized_test64::<INSTRUCTION>("4/10 SUB.D", true, 3000000, 0xe6dee76e5196222f)
    }
}

pub struct StresstestMulS;

impl Test for StresstestMulS {
    fn name(&self) -> &str { "MUL.S (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mul(FR::F4, FR::F0, FR::F2).s();
        randomized_test32::<INSTRUCTION>("5/10 MUL.S", true, 2000000, 0xd6fcbec0926cc113)
    }
}

pub struct StresstestMulD;

impl Test for StresstestMulD {
    fn name(&self) -> &str { "MUL.D (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_mul(FR::F4, FR::F0, FR::F2).d();
        randomized_test64::<INSTRUCTION>("6/10 MUL.D", true, 2000000, 0xe2c05aaec9cc1dc4)
    }
}

pub struct StresstestDivS;

impl Test for StresstestDivS {
    fn name(&self) -> &str { "DIV.S (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_div(FR::F4, FR::F0, FR::F2).s();
        randomized_test32::<INSTRUCTION>("7/10 DIV.S", true, 2000000, 0x76efe398daa10453)
    }
}

pub struct StresstestDivD;

impl Test for StresstestDivD {
    fn name(&self) -> &str { "DIV.D (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_div(FR::F4, FR::F0, FR::F2).d();
        randomized_test64::<INSTRUCTION>("8/10 DIV.D", true, 2000000, 0xf64eb203f41fe776)
    }
}

pub struct StresstestSqrtS;

impl Test for StresstestSqrtS {
    fn name(&self) -> &str { "SQRT.S (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sqrt(FR::F4, FR::F0).s();
        randomized_test32::<INSTRUCTION>("9/10 SQRT.S", true, 2000000, 0x4fde48202eec2625)
    }
}

pub struct StresstestSqrtD;

impl Test for StresstestSqrtD {
    fn name(&self) -> &str { "SQRT.D (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_sqrt(FR::F4, FR::F0).d();
        randomized_test64::<INSTRUCTION>("10/10 SQRT.D", true, 2000000, 0x88edc4c0aec12da4)
    }
}

pub struct StresstestCvtSFromW;

impl Test for StresstestCvtSFromW {
    fn name(&self) -> &str { "CVT.S.W (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_cvt_s(FR::F4, FR::F0).w();
        randomized_test32::<INSTRUCTION>("CVT.S.W", true, 2000000, 0x1adc160325245c23)
    }
}

pub struct StresstestCvtWFromS;

impl Test for StresstestCvtWFromS {
    fn name(&self) -> &str { "CVT.W.S (randomized - stresstest)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const INSTRUCTION: u32 = Assembler::make_cvt_w(FR::F4, FR::F0).s();
        randomized_test32::<INSTRUCTION>("CVT.W.S", true, 2000000, 0x1e1b3c375a34d773)
    }
}

