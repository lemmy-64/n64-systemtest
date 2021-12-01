use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::{format, vec};
use alloc::vec::Vec;
use core::any::Any;

use crate::cop0::cause_extract_exception;
use crate::exception_handler::drain_seen_exception;
use crate::tests::traps::Immediate;

mod address_error_exception;
mod cart_memory;
mod cop0;
mod exception_instructions;
mod overflow_exception;
mod startup;
mod soft_asserts;
mod tlb;
mod traps;

enum Level {
    // Very basic functionality - if this is broken, expect things to go bad
    BasicFunctionality,

    // Basic functionality that is rarely used
    RarelyUsed,

    // Some weird hardware quirk - this probably won't matter too much
    Weird,
}

trait Test {
    fn name(&self) -> &str;

    fn level(&self) -> Level;

    /// Returns a set of values to run the test with.
    /// Tests that don't support multiple values can return an empty Vec and will still
    /// get called once, in which case the value argument should be ignored
    fn values(&self) -> Vec<Box<dyn Any>>;

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String>;
}

pub fn run() {
    let tests: Vec<Box<dyn Test>> = vec! {
        Box::new(startup::StartupTest {}),

        Box::new(address_error_exception::UnalignedLW {}),
        Box::new(address_error_exception::UnalignedLW2 {}),
        Box::new(address_error_exception::UnalignedLWDelay {}),
        Box::new(address_error_exception::UnalignedSW {}),
        Box::new(cart_memory::LW {}),
        Box::new(cart_memory::LH {}),
        Box::new(cart_memory::LB {}),
        Box::new(cart_memory::write::WriteAndReadback {}),
        Box::new(cart_memory::write::WriteAndReadback2 {}),
        Box::new(cart_memory::write::WriteAndReadback3 {}),
        Box::new(cart_memory::write::WriteAndReadback4 {}),
        Box::new(cart_memory::write::WriteAndReadback5 {}),
        Box::new(cart_memory::write::DecayAfterSomeClockCycles {}),
        Box::new(cart_memory::write::Write32AndReadback8 {}),
        Box::new(cart_memory::write::Write32AndReadback16 {}),
        Box::new(cart_memory::write::Write8AndReadback32 {}),
        Box::new(cart_memory::write::Write16AndReadback32 {}),
        Box::new(cart_memory::write::Write64AndReadback32 {}),
        Box::new(cop0::ContextMasking {}),
        Box::new(cop0::ContextMixedBitWriting {}),
        Box::new(cop0::XContextMasking {}),
        Box::new(cop0::XContextMaskingMixed {}),
        Box::new(cop0::BadVAddrReadOnly {}),
        Box::new(cop0::ExceptPCNoMasking {}),
        Box::new(cop0::ErrorEPCNoMasking {}),
        Box::new(cop0::LLAddrIs32Bit {}),
        Box::new(cop0::StatusIs32Bit {}),
        Box::new(exception_instructions::Break {}),
        Box::new(exception_instructions::BreakDelay {}),
        Box::new(exception_instructions::Syscall {}),
        Box::new(exception_instructions::SyscallDelay {}),
        Box::new(overflow_exception::AddOverflowPositive {}),
        Box::new(overflow_exception::AddOverflowNegative {}),
        Box::new(overflow_exception::AddOverflowIntoR0 {}),
        Box::new(overflow_exception::AddOverflowDelaySlot1 {}),
        Box::new(overflow_exception::AddOverflowDelaySlot2 {}),
        Box::new(overflow_exception::DoubleAddOverflow {}),
        Box::new(overflow_exception::DoubleAddOverflowIntoR0 {}),
        Box::new(overflow_exception::SubOverflow {}),
        Box::new(overflow_exception::SubOverflowIntoR0 {}),
        Box::new(overflow_exception::DoubleSubOverflow {}),
        Box::new(overflow_exception::DoubleSubOverflowIntoR0 {}),
        Box::new(overflow_exception::AddImmediateOverflow {}),
        Box::new(overflow_exception::AddImmediateOverflowIntoR0 {}),
        Box::new(overflow_exception::DoubleAddImmediateOverflow {}),
        Box::new(overflow_exception::DoubleAddImmediateOverflowIntoR0 {}),

        Box::new(tlb::WiredRandom {}),
        Box::new(tlb::WiredOutOfBoundsRandom {}),
        Box::new(tlb::WriteRandomExpectIgnored {}),
        Box::new(tlb::IndexMasking {}),
        Box::new(tlb::EntryLo0Masking {}),
        Box::new(tlb::EntryLo0Masking64 {}),
        Box::new(tlb::EntryLo1Masking {}),
        Box::new(tlb::EntryLo1Masking64 {}),
        Box::new(tlb::EntryHiMasking {}),
        Box::new(tlb::PageMaskMasking {}),
        Box::new(tlb::ConfigReadWrite {}),
        Box::new(tlb::TLBWriteReadPageMask {}),
        Box::new(tlb::TLBWriteReadBackEntry {}),
        Box::new(tlb::TLBUseTestRead0 {}),
        Box::new(tlb::TLBUseTestRead1 {}),
        Box::new(tlb::TLBUseTestReadMatchViaASID {}),

        Box::new(tlb::exceptions::ReadMiss4k {}),
        Box::new(tlb::exceptions::ReadMiss16k {}),
        Box::new(tlb::exceptions::ReadMiss64k {}),
        Box::new(tlb::exceptions::ReadMiss256k {}),
        Box::new(tlb::exceptions::ReadMiss1M {}),
        Box::new(tlb::exceptions::ReadMiss4M {}),
        Box::new(tlb::exceptions::ReadMiss16M {}),
        Box::new(tlb::exceptions::StoreMiss4k {}),
        Box::new(tlb::exceptions::ReadNonValid4k {}),
        Box::new(tlb::exceptions::StoreNonValid4k {}),
        Box::new(tlb::exceptions::StoreNonDirty4k {}),
        Box::new(tlb::exceptions::StoreNonDirtyAndNonValid4k {}),

        Box::new(traps::TLT {}),
        Box::new(traps::TLTU {}),
        Box::new(traps::TGE {}),
        Box::new(traps::TGEU {}),
        Box::new(traps::TEQ {}),
        Box::new(traps::TNE {}),
        Box::new(traps::TEQI {}),
        Box::new(traps::TNEI {}),
        Box::new(traps::TGEI {}),
        Box::new(traps::TGEIU {}),
        Box::new(traps::TLTI {}),
        Box::new(traps::TLTIU {}),
        Box::new(traps::delay::TNEDelay1 {}),
        Box::new(traps::delay::TNEDelay2 {}),
    };

    let mut succeeded = 0;
    let mut failed = 0;

    fn test_value(test: &Box<dyn Test>, value: &Box::<dyn Any>, failed: &mut u32, succeeded: &mut u32) {
        fn value_desc(value: &Box<dyn Any>) -> String {
            match (*value).downcast_ref::<u32>() {
                Some(v) => return format!("{:?}", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, i64, i64)>() {
                Some(v) => return format!("{:?}", v),
                None => {},
            }
            match (*value).downcast_ref::<(bool, u64, u64)>() {
                Some(v) => return format!("{:?}", v),
                None => {}
            }
            match (*value).downcast_ref::<(bool, u64, Immediate)>() {
                Some(v) => return format!("{:?}", v),
                None => {}
            }
            return "(value)".to_string();
        }

        let test_result = test.run(&value);
        match drain_seen_exception() {
            Some(exception) => {
                // If the test caused an exception, don't even bother looking at the result. Just count it as failed
                crate::println!("Test \"{:?}\' with '{:?}' failed with exception: {:?}", test.name(), value_desc(value), cause_extract_exception(exception.cause));
                *failed += 1;
            }
            None => {
                match test_result {
                    Ok(_) => {
                        *succeeded += 1
                    }
                    Err(error) => {
                        crate::println!("Test \"{:?}\' with '{:?}' failed: {}", test.name(), value_desc(value), error);
                        *failed += 1;
                    }
                }
            }
        }
    }

    let dummy_test_value: Box<dyn Any> = Box::new(0u32);
    for test in tests {
        let values = test.values();
        if values.len() == 0 {
            test_value(&test, &dummy_test_value, &mut failed, &mut succeeded);
        } else {
            for value in values {
                test_value(&test, &value, &mut failed, &mut succeeded);
            }
        }
    }

    crate::println!();
    if (failed + succeeded) == 0 {
        crate::println!("Done, but no tests were executed");
    } else {
        crate::println!("Done! Tests: {}. Failed: {}. Success rate: {}%", failed + succeeded, failed, succeeded * 100 / (failed + succeeded));
    }

}