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

/// The importance level of a [test](Test).
#[derive(Eq, PartialEq)]
pub enum Level {
    /// Very basic functionality. If this is broken, expect things to go bad.
    BasicFunctionality,

    /// Rarely used basic functionality.
    RarelyUsed,

    /// A weird hardware quirk. This probably won't matter too much.
    Weird,

    /// A hardware quirk that is so weird that the test won't be run by default.
    TooWeird,

    /// Slow test of basic functionality. Only enabled when compiled with stresstest feature flags.
    StressTest,

    /// A test that requires cycle-accurate emulation in the emulator. In general, this is poorly understood
    /// (and test coverage is quite spotty), so this is off by default.
    CycleExact,

    /// Basic RDP functionality (e.g. triangle drawing). Tests in this category can
    /// be mapped to a hardware renderer (Direct3D, OpenGL, Vulkan)
    RDPBasic,

    /// RDP coverage emulation, which is thought to not be runnable on dedicated 3D hardware
    /// (unless doing GPU compute). Off by default
    RDPPrecise,
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
    let mut succeeded = 0;
    let mut skipped = 0;
    let mut failed = 0;

    fn test_value(test: &Box<dyn Test>, value: &Box::<dyn Any>, failed: &mut u32, succeeded: &mut u32, skipped: &mut u32, time: &mut u32) {
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
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, f64), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, i32), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, Result<(FCSRFlags, i64), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(f32, Result<(FCSRFlags, i64), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f32, f32, Result<(FCSRFlags, f32), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, f32), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, f64), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, i32), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, f64, Result<(FCSRFlags, i64), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(f64, Result<(FCSRFlags, i64), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, i32, Result<(FCSRFlags, f32), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, i32, Result<(FCSRFlags, f64), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
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
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, FCSRRoundingMode, i64, Result<(FCSRFlags, f32), ()>)>() {
                Some(v) => return format!(" with '{:x?}'", v),
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
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(f32, f32, Ordering, FPUSpecialNumber)>() {
                Some(v) => return format!(" with '{:x?}'", v),
                None => {}
            }
            return " with unknown arguments".to_string();
        }

        if test.level() == Level::TooWeird || test.level() == Level::CycleExact {
            *skipped += 1
        } else {
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
        let mut time = 0u32;
        if values.len() == 0 {
            test_value(&test, &dummy_test_value, &mut failed, &mut succeeded, &mut skipped, &mut time);
        } else {
            for value in values {
                test_value(&test, &value, &mut failed, &mut succeeded, &mut skipped, &mut time);
            }
        }
        test_times.push((index, time));
    }
    let counter_after = crate::cop0::count();

    println!();
    if (failed + succeeded) == 0 {
        println!("Done, but no tests were executed");
    } else {
        let debug_msg = format!("Finished in {:0.2}s. Tests: {}. Failed: {}. Success rate: {}%. Skipped {} tests.\n\n", cycles_to_seconds(counter_after - counter_before), failed + succeeded, failed, succeeded * 100 / (failed + succeeded), skipped);
        // Print to the console, at the end
        text_out(&debug_msg);

        // For the on-screen console, prepend it. This way it's visible even if there are a lot of failed tests
        FramebufferConsole::instance().lock().prepend(&debug_msg);
    }

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