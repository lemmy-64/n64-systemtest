use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::cop0;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct ContextMasking {}

impl Test for ContextMasking {
    fn name(&self) -> &str { "Context (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::context_64();
        for value in [0, 1, 5, 15, 30, 31, 32, 63, 64, 64, 1020, 63102, 0x0F000000, 0xFFFF0002, 0xFFFFFFFF, 0xF2345678_0000000, 0xFFFFFFFF_FFFFFFFF, 0] {
            unsafe { cop0::set_context_64(value); }
            let expected = (value & 0xFFFFFFFF_FF800000) | (previous & 0x7FFFFF);
            let readback = cop0::context_64();
            soft_assert_eq(readback, expected, format!("Context was written as {:x}", value).as_str())?;
        }
        Ok(())
    }
}

pub struct WiredMasking;

/// Tests if read/write masking is correct for the COP0 Wired register.
impl Test for WiredMasking {
    fn name(&self) -> &str { "Wired (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_wired(0xFFFFFFFF); }
        soft_assert_eq(cop0::wired(), 63, "Wired was written as 0xFFFFFFFF")?;
        
        Ok(())
    }
}

pub struct ContextMixedBitWriting {}

impl Test for ContextMixedBitWriting {
    fn name(&self) -> &str { "Context (sign extension)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::context_64();

        unsafe { cop0::set_context_64(0x12345678_00000000); }
        let expected1 = 0x12345678_00000000 | (previous & 0x7FFFFF);
        soft_assert_eq(cop0::context_64(), expected1, format!("Context after writing 64 bit").as_str())?;

        unsafe { cop0::set_context_32(0x8B000000); }
        let expected2 = 0xFFFFFFFF_8B000000 | (previous & 0x7FFFFF);
        soft_assert_eq(cop0::context_64(), expected2, format!("Writing Context (32 bit) should sign extend").as_str())?;

        unsafe { cop0::set_context_32(0x7B000000); }
        let expected3 = 0x7B000000 | (previous & 0x7FFFFF);
        soft_assert_eq(cop0::context_64(), expected3, format!("Writing Context (32 bit) should sign extend").as_str())?;

        Ok(())
    }
}

pub struct XContextMasking {}

impl Test for XContextMasking {
    fn name(&self) -> &str { "XContext (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::xcontext_64();
        for value in [0xF1234567_89ABCDEF, 0xFFFFFFFF_FFFFFFFF, 0x00000000_FFFFFFFF, 0x00000001_FFFFFFFF] {
            unsafe { cop0::set_xcontext_64(value); }
            let expected = (value & 0xFFFFFFFE_00000000) | (previous & 0x00000001_FFFFFFFF);
            let readback = cop0::xcontext_64();
            soft_assert_eq(readback, expected, format!("XContext was written as {:x}. But as it is readonly it shouldn't change", value).as_str())?;
        }
        Ok(())
    }
}

pub struct XContextMaskingMixed {}

impl Test for XContextMaskingMixed {
    fn name(&self) -> &str { "XContext (masking, mixed)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::xcontext_64();

        unsafe { cop0::set_xcontext_64(0x12345678_00000000); }
        let expected1 = 0x12345678_00000000 | (previous & 0x00000001_FFFFFFFF);
        soft_assert_eq(cop0::xcontext_64(), expected1, format!("XContext after writing 64 bit").as_str())?;

        unsafe { cop0::set_xcontext_32(0x8B000000); }
        let expected2 = 0xFFFFFFFE_00000000 | (previous & 0x00000001_FFFFFFFF);
        soft_assert_eq(cop0::xcontext_64(), expected2, format!("Writing XContext (32 bit) should sign extend").as_str())?;

        unsafe { cop0::set_xcontext_32(0x7B000000); }
        let expected3 = previous & 0x00000001_FFFFFFFF;
        soft_assert_eq(cop0::xcontext_64(), expected3, format!("Writing XContext (32 bit) should sign extend").as_str())?;
        Ok(())
    }
}

pub struct BadVAddrReadOnly {}

impl Test for BadVAddrReadOnly {
    fn name(&self) -> &str { "BadVAddr (readonly)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::badvaddr();
        for value in [0xF1234567_89ABCDEF, 0x00000000_00000000, 0x00000000_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF] {
            unsafe { cop0::set_badvaddr(value); }
            let expected = previous;
            let readback = cop0::badvaddr();
            soft_assert_eq(readback, expected, format!("BadVAddr was written as {:x}. But as it is readonly it shouldn't change", value).as_str())?;
        }
        Ok(())
    }
}

pub struct ExceptPCNoMasking {}

impl Test for ExceptPCNoMasking {
    fn name(&self) -> &str { "ExceptPC (no masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0xF7654321_89ABCDEF, 0x00000000_00000000, 0x00000000_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF] {
            unsafe { cop0::set_exceptpc(value); }
            let expected = value;
            let readback = cop0::exceptpc();
            soft_assert_eq(readback, expected, format!("ExceptPC was written as 0x{:x}. It shouldn't be masked", value).as_str())?;
        }
        Ok(())
    }
}

pub struct ErrorEPCNoMasking {}

impl Test for ErrorEPCNoMasking {
    fn name(&self) -> &str { "ErrorEPC (no masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0xF7654321_89ABCDEF, 0x00000000_00000000, 0x00000000_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF, 0x12345678_AABBCCDD] {
            unsafe { cop0::set_errorepc(value); }
            let expected = value;
            let readback = cop0::errorepc();
            soft_assert_eq(readback, expected, format!("ErrorEPC was written as 0x{:x}. It shouldn't be masked", value).as_str())?;
        }
        Ok(())
    }
}

pub struct LLAddrIs32Bit {}

impl Test for LLAddrIs32Bit {
    fn name(&self) -> &str { "LLAddr (32 bit)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0xF7654321_89ABCDEF, 0x00000000_00000000, 0x00000000_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF] {
            unsafe { cop0::set_lladdr(value); }
            let expected = value & 0xFFFFFFFF;
            let readback = cop0::lladdr();
            soft_assert_eq(readback, expected, format!("LLAddr was written as 0x{:x}. Only its lower 32 bit should change", value).as_str())?;
        }
        Ok(())
    }
}

pub struct StatusIs32Bit {}

impl Test for StatusIs32Bit {
    fn name(&self) -> &str { "Status (upper 32 bit ignored)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::status_64();
        for value in [0xFFFFFFFF_00000000, 0x00000000_00000000] {
            let write_value = (value & 0xFFFFFFFF_00000000) | (previous & 0xFFFFFFFF);
            unsafe { cop0::set_status_64(write_value); }
            let expected = previous;
            let readback = cop0::status_64();
            soft_assert_eq(readback, expected, format!("Status was written as 0x{:x}. It's upper 32 bit should be constant", write_value).as_str())?;
        }
        Ok(())
    }
}
