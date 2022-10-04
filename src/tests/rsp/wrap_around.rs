use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::RSPAssembler;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// When the PC reaches the end of IMEM, it should wrap around to the beginning
pub struct WrapAround {

}

impl Test for WrapAround {
    fn name(&self) -> &str { "RSP Wrap around" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Write 2 NOPs at the end so that we can wrap around
        let mut assembler = RSPAssembler::new(0xFF8);
        assembler.write_nop();
        assembler.write_nop();

        // Now write the beginning
        let mut assembler = RSPAssembler::new(0);
        assembler.write_break();

        RSP::run_and_wait(0xFF8);

        soft_assert_eq(RSP::pc(), 0x4, "RSP PC isn't as expected after running")?;
        soft_assert_eq(RSP::status(), 0x3, "RSP STATUS isn't as expected after running")?;

        Ok(())
    }
}
