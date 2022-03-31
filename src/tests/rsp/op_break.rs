use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::RSPAssembler;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Send over a NOP and BREAK to the RSP, execute them and wait until the RSP is done
pub struct BREAK {

}

impl Test for BREAK {
    fn name(&self) -> &str { "RSP BREAK" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut assembler = RSPAssembler::new(0);
        assembler.write_nop();
        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(RSP::pc(), 0x8, "RSP PC isn't as expected after running")?;
        soft_assert_eq(RSP::status(), 0x3, "RSP STATUS isn't as expected after running")?;

        RSP::clear_broke();

        soft_assert_eq(RSP::status(), 0x1, "RSP STATUS isn't as expected after clearing BROKE")?;

        Ok(())
    }
}
