use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::cmp::{min, Ordering};
use arbitrary_int::{u2, u27, u5};

use crate::cop0::{set_status, Status};
use crate::exception_handler::drain_seen_exception;
use crate::{FramebufferConsole, print, println};
use crate::cop1::{FCSR, FCSRFlags, FCSRRoundingMode, set_fcsr};
use crate::isviewer::text_out;
use crate::math::soft_float::{SoftF32, SoftF64};
use crate::tests::cop1::compares::FPUSpecialNumber;
use crate::tests::traps::Immediate;

mod arithmetic;
mod address_error_exception;
mod cart_memory;
mod cop_unusable;
mod cop0;
mod cop1;
mod exception_instructions;
mod jumps;
mod overflow_exception;
mod pif_memory;
mod rdp;
mod rsp;
mod startup;
mod soft_asserts;
mod sp_memory;
mod testlist;
mod tlb;
mod tlb64;
mod traps;

mod configuration {
    pub const BASE: bool = cfg!(feature = "base");
    pub const TIMING: bool = cfg!(feature = "timing");
    pub const CYCLE: bool = cfg!(feature = "cycle");
    pub const COP0HAZARD: bool = cfg!(feature = "cop0hazard");
    pub const POORLY_UNDERSTOOD_QUIRK: bool = cfg!(feature = "poorly_understood_quirk");
}

/// The importance level of a [test](Test).
#[derive(Eq, PartialEq)]
#[repr(u8)]
pub enum Level {
    /// Very basic functionality. If this is broken, expect things to go bad.
    BasicFunctionality = 0,

    /// Rarely used basic functionality.
    RarelyUsed = 1,

    /// A weird hardware quirk. This probably won't matter too much.
    Weird = 2,

    /// Basic RDP functionality (e.g. triangle drawing). Tests in this category can
    /// be mapped to a hardware renderer (Direct3D, OpenGL, Vulkan)
    RDPBasic = 3,

    /// RDP coverage emulation, which is thought to not be runnable on dedicated 3D hardware
    /// (unless doing GPU compute).
    RDPPrecise = 4,

    /// A test that requires emulation of accurate time (e.g. a NOP is 0.5 cycles, Random decrements by 1 per instruction).
    Timing = 5,

    /// A test that requires emulation of COP0 hazards
    COP0Hazard = 6,

    /// A test that requires spreading an instruction over several cycles (like hardware does)
    Cycle = 7,

    /// Some observed hardware quirk that isn't properly understood - a test exists as a starting point
    PoorlyUnderstoodQuirk = 8,

    /// Slow test of basic functionality. Only enabled when compiled with stresstest feature flags.
    StressTest = 9,

    // Make sure this is the last value
    _COUNT = 10,
}

/// Trait for a test or group of tests that are performed together.
pub trait Test {
    /// Human-readable name for the test.
    fn name(&self) -> &str;

    /// Returns the [level](Level) of the test.
    /// 
    /// Some levels may be filtered out of the compiled test rom by default.
    fn level(&self) -> Level;

    /// Returns a list of values to run the test with.
    /// 
    /// If the list is empty, the test will only be run once, using a dummy value. Otherwise, the
    /// test will be run once for every value in the list.
    fn values(&self) -> Vec<Box<dyn Any>>;

    /// Run the test with a provided value.
    /// 
    /// Value may be a dummy, if [`Self::values()`] returned an empty list.
    /// 
    /// If the test fails, return a human-readable description of the issue.
    fn run(&self, value: &Box<dyn Any>) -> Result<(), String>;
}

fn cycles_to_seconds(value: u32) -> f32
{
    value as f32 / (93_750_000f32 / 2f32)
}

pub fn run() {
    const LEVEL_COUNT: usize = Level::_COUNT as usize;
    let mut succeeded = [0u32; LEVEL_COUNT];
    let mut failed = [0u32; LEVEL_COUNT];

    fn test_value(test: &Box<dyn Test>, value: &Box::<dyn Any>, failed: &mut u32, succeeded: &mut u32, time: &mut u32) {
        fn value_desc(value: &Box<dyn Any>) -> String {
            match (*value).downcast_ref::<()>() {
                Some(_) => return String::new(),
                None => {},
            }
            match (*value).downcast_ref::<u32>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<bool>() {
                Some(v) => return format!(" with '{:?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, u32)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, u64)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, u32, u32)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(u32, u32, u32)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(u32, u5, u32)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(u64, u32, u64)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(u64, u32, u8)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(u64, u27, u2)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, i64, i64)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, u64, u64)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, u64, Immediate)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, f32), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let new_expected = expected.map(|(flags, f)| (flags, SoftF32::new(f)));
                    let temp = (*flush_denorm_to_zero, *rounding_mode, SoftF32::new(*value), new_expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, f64), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let new_expected = expected.map(|(flags, f)| (flags, SoftF64::new(f)));
                    let temp = (*flush_denorm_to_zero, *rounding_mode, SoftF32::new(*value), new_expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, i32), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let temp = (*flush_denorm_to_zero, *rounding_mode, SoftF32::new(*value), expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, i64), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let temp = (*flush_denorm_to_zero, *rounding_mode, SoftF32::new(*value), expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(f32, Result<(FCSRFlags, i64), ()>)>() {
                Some((value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let temp = (SoftF32::new(*value), expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, f32, Result<(FCSRFlags, f32), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value1, value2, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let new_expected = expected.map(|(flags, f)| (flags, SoftF32::new(f)));
                    let temp = (*flush_denorm_to_zero, *rounding_mode, SoftF32::new(*value1), SoftF32::new(*value2), new_expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, f32), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let new_expected = expected.map(|(flags, f)| (flags, SoftF32::new(f)));
                    let temp = (*flush_denorm_to_zero, *rounding_mode, SoftF64::new(*value), new_expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, f64), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let new_expected = expected.map(|(flags, f)| (flags, SoftF64::new(f)));
                    let temp = (*flush_denorm_to_zero, *rounding_mode, SoftF64::new(*value), new_expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, i32), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let temp = (*flush_denorm_to_zero, *rounding_mode, SoftF64::new(*value), expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, i64), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let temp = (*flush_denorm_to_zero, *rounding_mode, SoftF64::new(*value), expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(f64, Result<(FCSRFlags, i64), ()>)>() {
                Some((value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let temp = (SoftF64::new(*value), expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, i32, Result<(FCSRFlags, f32), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let new_expected = expected.map(|(flags, f)| (flags, SoftF32::new(f)));
                    let temp = (*flush_denorm_to_zero, *rounding_mode, value, new_expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, i32, Result<(FCSRFlags, f64), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let new_expected = expected.map(|(flags, f)| (flags, SoftF64::new(f)));
                    let temp = (*flush_denorm_to_zero, *rounding_mode, value, new_expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(i32, Result<(FCSRFlags, i32), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(i32, Result<(FCSRFlags, i64), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, i64, Result<(FCSRFlags, f64), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let new_expected = expected.map(|(flags, f)| (flags, SoftF64::new(f)));
                    let temp = (*flush_denorm_to_zero, *rounding_mode, value, new_expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, i64, Result<(FCSRFlags, f32), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, value, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let new_expected = expected.map(|(flags, f)| (flags, SoftF32::new(f)));
                    let temp = (*flush_denorm_to_zero, *rounding_mode, value, new_expected);
                    return format!(" with '{:x?}'", temp);
                }
                None => {}
            }
            match (*value).downcast_ref::<(f32, Result<(FCSRFlags, i32), ()>)>() {
                Some((value, expected)) => {
                    let temp = (SoftF32::new(*value), expected);
                    return format!(" with '{:x?}'", temp)
                },
                None => {}
            }
            match (*value).downcast_ref::<(f64, Result<(FCSRFlags, i32), ()>)>() {
                Some((value, expected)) => {
                    let temp = (SoftF64::new(*value), expected);
                    return format!(" with '{:x?}'", temp)
                },
                None => {}
            }
            match (*value).downcast_ref::<(i64, Result<(FCSRFlags, i32), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(i64, Result<(FCSRFlags, i64), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, f64, Result<(FCSRFlags, f64), ()>)>() {
                Some((flush_denorm_to_zero, rounding_mode, f1, f2, expected)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let temp = (*flush_denorm_to_zero, *rounding_mode, SoftF64::new(*f1), SoftF64::new(*f2), *expected);
                    return format!(" with '{:x?}'", temp);
                },
                None => {}
            }
            match (*value).downcast_ref::<(f32, f32, Ordering, FPUSpecialNumber)>() {
                Some((f1, f2, ordering, special)) => {
                    // Convert f32 to SoftF32 - it prints more nicely
                    let temp = (SoftF32::new(*f1), SoftF32::new(*f2), *ordering, *special);
                    return format!(" with '{:x?}'", temp);
                },
                None => {}
            }
            match (*value).downcast_ref::<(f64, f64, Ordering, FPUSpecialNumber)>() {
                Some((f1, f2, ordering, special)) => {
                    // Convert f64 to SoftF64 - it prints more nicely
                    let temp = (SoftF64::new(*f1), SoftF64::new(*f2), *ordering, *special);
                    return format!(" with '{:x?}'", temp);
                },
                None => {}
            }
            return " with unknown arguments".to_string();
        }

        if test.name() != "StartupTest" {
            // Set sane environment for any test except the startup test (that one tests values that
            // were set at boot/reset
            unsafe { set_status(Status::DEFAULT); }
            set_fcsr(FCSR::DEFAULT);
        }

        let counter_before = crate::cop0::count();
        let test_result = test.run(&value);
        let counter_after = crate::cop0::count();
        *time += counter_after - counter_before;

        unsafe { set_status(Status::DEFAULT); }
        set_fcsr(FCSR::DEFAULT);

        match drain_seen_exception() {
            Some((exception, _)) => {
                // If the test caused an exception, don't even bother looking at the result. Just count it as failed
                match exception.cause.exception() {
                    Ok(e) => println!("Test '{}'{} failed with exception: {:?}\n", test.name(), value_desc(value), e),
                    Err(e) => println!("Test '{}'{} failed with unknown exception: {:?}\n", test.name(), value_desc(value), e),
                }

                *failed += 1;
            }
            None => {
                match test_result {
                    Ok(_) => {
                        *succeeded += 1
                    }
                    Err(error) => {
                        println!("Test '{}'{} failed: {}\n", test.name(), value_desc(value), error);
                        *failed += 1;
                    }
                }
            }
        }
    }

    let tests = testlist::tests();
    let mut test_times: Vec<(usize, u32)> = Vec::new();
    let dummy_test_value: Box<dyn Any> = Box::new(());
    let counter_before = crate::cop0::count();
    for (index, test) in tests.iter().enumerate() {
        text_out("Running ");
        text_out(test.name());
        text_out("...\n");

        let values = test.values();
        let level = test.level();

        let execute_test = match level {
            Level::BasicFunctionality | Level::RarelyUsed | Level::Weird | Level::RDPBasic | Level::RDPPrecise => configuration::BASE,
            Level::Timing => configuration::TIMING,
            Level::Cycle => configuration::CYCLE,
            Level::COP0Hazard => configuration::COP0HAZARD,
            Level::PoorlyUnderstoodQuirk => configuration::POORLY_UNDERSTOOD_QUIRK,
            Level::StressTest => {
                // stresstests have individual feature flags in cargo.toml - if we see it here it means it's supposed
                // to be included
                true
            },
            Level::_COUNT => panic!("Don't use _COUNT as Level"),
        };

        if execute_test {
            let mut time = 0u32;
            if values.len() == 0 {
                test_value(&test, &dummy_test_value, &mut failed[level as usize], &mut succeeded[level as usize], &mut time);
            } else {
                for value in values {
                    test_value(&test, &value, &mut failed[level as usize], &mut succeeded[level as usize], &mut time);
                }
            }
            test_times.push((index, time));
        }
    }
    let counter_after = crate::cop0::count();

    println!();
    let succeeded_total: u32 = succeeded.iter().sum();
    let failed_total: u32 = failed.iter().sum();
    if (failed_total + succeeded_total) == 0 {
        println!("Done, but no tests were executed");
    } else {
        const VERSION: &str = env!("CARGO_PKG_VERSION");

        let mut succeeded_base = succeeded_total;
        let mut failed_base = failed_total;

        let mut category_stat = |friendly_name, level| {
            let succeeded = succeeded[level as usize];
            let failed = failed[level as usize];
            succeeded_base -= succeeded;
            failed_base -= failed;
            if succeeded + failed != 0 {
                format!("\n{}: Failed {} of {} tests ({}% success rate)", friendly_name, failed, failed + succeeded, succeeded * 100 / (failed + succeeded))
            } else {
                format!("")
            }
        };

        let timing_stat = category_stat("Timing", Level::Timing);
        let cycle_stat = category_stat("Cycle", Level::Cycle);
        let cp0_hazards_stat = category_stat("CP0-hazards", Level::COP0Hazard);
        let poorly_understood_quirk_stat = category_stat("Poorly-understood-quirk", Level::PoorlyUnderstoodQuirk);

        let debug_msg = format!(
            "n64-systemtest {} (base={} timing={} cycle={} cp0-hazards={})
Finished in {:0.2}s. Base: Failed {} of {} tests ({}% success rate){}{}{}{}\n",
            VERSION, configuration::BASE as u8, configuration::TIMING as u8, configuration::CYCLE as u8, configuration::COP0HAZARD as u8,
            cycles_to_seconds(counter_after - counter_before), failed_base, failed_base + succeeded_base, succeeded_base * 100 / (failed_base + succeeded_base),
            timing_stat, cycle_stat, cp0_hazards_stat, poorly_understood_quirk_stat
        );
        // Print to the console, at the end
        text_out(&debug_msg);

        // For the on-screen console, prepend it. This way it's visible even if there are a lot of failed tests
        FramebufferConsole::instance().lock().prepend(&debug_msg);

        test_times.sort_by(|(_, a), (_, b)| { a.cmp(b).reverse() });
        println!("");
        print!("Slowest tests: ");
        for i in 0..min(5, test_times.len()) {
            let (test_index, test_time) = test_times[i];
            let test_name = tests[test_index].name();
            if i > 0 {
                print!(", ");
            }
            print!("{} ({:0.2}s)", test_name, cycles_to_seconds(test_time));
        }
        println!("");
    }
}