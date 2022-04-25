use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;

use crate::cop0::cause_extract_exception;
use crate::exception_handler::drain_seen_exception;
use crate::{print, println};
use crate::tests::traps::Immediate;

mod arithmetic;
mod address_error_exception;
mod cart_memory;
mod cop0;
mod exception_instructions;
mod jumps;
mod overflow_exception;
mod pif_memory;
mod rsp;
mod startup;
mod soft_asserts;
mod sp_memory;
mod testlist;
mod tlb;
mod traps;

#[derive(Eq, PartialEq)]
pub enum Level {
    // Very basic functionality - if this is broken, expect things to go bad
    BasicFunctionality,

    // Basic functionality that is rarely used
    RarelyUsed,

    // Some weird hardware quirk - this probably won't matter too much
    Weird,

    // Some hardware quirk that is so weird that the test won't be run by default
    TooWeird,

    // Basic functionality, but extremely slow so not run by default
    StressTest,
}

pub trait Test {
    fn name(&self) -> &str;

    fn level(&self) -> Level;

    /// Returns a set of values to run the test with.
    /// Tests that don't support multiple values can return an empty Vec and will still
    /// get called once, in which case the value argument should be ignored
    fn values(&self) -> Vec<Box<dyn Any>>;

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String>;
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
                Some(v) => return format!(" with '{:?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<bool>() {
                Some(v) => return format!(" with '{:?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, u32)>() {
                Some(v) => return format!(" with '{:?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, u32, u32)>() {
                Some(v) => return format!(" with '{:?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(u32, u32, u32)>() {
                Some(v) => return format!(" with '{:?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(u64, u32, u64)>() {
                Some(v) => return format!(" with '{:?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, i64, i64)>() {
                Some(v) => return format!(" with '{:?}'", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, u64, u64)>() {
                Some(v) => return format!(" with '{:?}'", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, u64, Immediate)>() {
                Some(v) => return format!(" with '{:?}'", v),
                None => {}
            }
            return " with unknown arguments".to_string();
        }

        if test.level() == Level::TooWeird {
            *skipped += 1
        } else {
            // Kernel mode, erl/exl off. 32 bit addressing mode. Tests that want to test something else
            // will have to set that themselves
            unsafe { crate::cop0::set_status(0x24000000); }

            let counter_before = crate::cop0::count();
            let test_result = test.run(&value);
            let counter_after = crate::cop0::count();
            *time += counter_after - counter_before;

            unsafe { crate::cop0::set_status(0x24000000); }

            match drain_seen_exception() {
                Some(exception) => {
                    // If the test caused an exception, don't even bother looking at the result. Just count it as failed
                    crate::println!("Test '{}'{} failed with exception: {:?}\n", test.name(), value_desc(value), cause_extract_exception(exception.cause));
                    *failed += 1;
                }
                None => {
                    match test_result {
                        Ok(_) => {
                            *succeeded += 1
                        }
                        Err(error) => {
                            crate::println!("Test '{}'{} failed: {}\n", test.name(), value_desc(value), error);
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
    for (index, test) in tests.iter().enumerate() {
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

    println!();
    if (failed + succeeded) == 0 {
        println!("Done, but no tests were executed");
    } else {
        println!("Done! Tests: {}. Failed: {}. Success rate: {}%. Skipped {} tests", failed + succeeded, failed, succeeded * 100 / (failed + succeeded), skipped);
    }

    test_times.sort_by(|(_, a), (_, b)| { a.cmp(b).reverse() });
    let total_time = test_times.iter().fold(0u32, |acc, (_, time)| acc + time);

    println!("");
    print!("Slowest tests: ");
    for i in 0..5 {
        let (test_index, test_time) = test_times[i];
        let test_name = tests[test_index].name();
        if i > 0 {
            print!(", ");
        }
        print!("{} ({}%)", test_name, test_time * 100 / total_time);
    }
    println!("");
}