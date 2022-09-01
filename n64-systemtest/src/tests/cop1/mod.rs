use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use crate::cop1::{cfc1, ctc1, fcsr, FCSR, set_fcsr};
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Lessons learned:
// - CFC1 for 0 returns a constent. CTC1 to 0 is ignored.
// - CFC1 for 1 returns random garbage that seems to contain the original value of the register as well
//   no test for those
// - Untested, but we're assuming that 2..=30 behave the same way
// - CFC1 for 31 returns the FCSR. It has a write-mask. If an interrupt is set AND enabled at the same time,
//   it is fired right away

/// Tests if read/write masking is correct for the COP0 CacheError register.
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

/// Tests if read/write masking is correct for the COP0 CacheError register.
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
