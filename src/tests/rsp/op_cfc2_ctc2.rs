use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use core::cmp::min;
use arbitrary_int::u5;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{CP2FlagsRegister, GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq2};

// Lessons learned:
// There are three registers: VCO, VCC, VCE. When copying, CFC2 only looks at the bottom two
// bits: 0=VCO, 1=VCC, 2=VCE, 3=VCE. Higher number repeat this.
// CFC2 does sign extension, but for 16 to 32 bits, so VCE is never sign extended.

pub struct CTC2CFC2 {}

impl Test for CTC2CFC2 {
    fn name(&self) -> &str { "RSP CTC2/CFC2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        // Set all flags and see what we get back
        assembler.write_li(GPR::AT, 0x12345678);
        assembler.write_li(GPR::A0, 0x87654321);
        assembler.write_li(GPR::A1, 0x11223344);
        assembler.write_ctc2(CP2FlagsRegister::VCO, GPR::AT);
        assembler.write_ctc2(CP2FlagsRegister::VCC, GPR::A0);
        assembler.write_ctc2(CP2FlagsRegister::VCE, GPR::A1);

        assembler.write_cfc2(CP2FlagsRegister::VCO, GPR::S0);
        assembler.write_cfc2(CP2FlagsRegister::VCC, GPR::S1);
        assembler.write_cfc2(CP2FlagsRegister::VCE, GPR::S2);

        // Same, but this time with the highest bit set
        assembler.write_li(GPR::AT, 0x12348678);
        assembler.write_li(GPR::A0, 0x87658321);
        assembler.write_li(GPR::A1, 0x11223384);
        assembler.write_ctc2(CP2FlagsRegister::VCO, GPR::AT);
        assembler.write_ctc2(CP2FlagsRegister::VCC, GPR::A0);
        assembler.write_ctc2(CP2FlagsRegister::VCE, GPR::A1);

        assembler.write_cfc2(CP2FlagsRegister::VCO, GPR::S3);
        assembler.write_cfc2(CP2FlagsRegister::VCC, GPR::S4);
        assembler.write_cfc2(CP2FlagsRegister::VCE, GPR::S5);

        // Write all back
        for (index, gpr) in (GPR::S0..=GPR::S5).enumerate() {
            assembler.write_sw(gpr, GPR::R0, 4 * index as i16);
        }

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read(0x00), 0x00005678, "CTC2 0, 0x12345678, then read back")?;
        soft_assert_eq(SPMEM::read(0x04), 0x00004321, "CTC2 1, 0x87654321, then read back")?;
        soft_assert_eq(SPMEM::read(0x08), 0x00000044, "CTC2 2, 0x11223344, then read back")?;

        soft_assert_eq(SPMEM::read(0x0C), 0xFFFF8678, "CTC2 0, 0x12348678, then read back")?;
        soft_assert_eq(SPMEM::read(0x10), 0xFFFF8321, "CTC2 1, 0x87658321, then read back")?;
        soft_assert_eq(SPMEM::read(0x14), 0x00000084, "CTC2 2, 0x11223384, then read back")?;

        Ok(())
    }
}

pub struct CFC2WeirdIndexes {}

impl Test for CFC2WeirdIndexes {
    fn name(&self) -> &str { "RSP CFC2 (index higher than three)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        // Set all flags to 0x12345678 and see what we get back
        assembler.write_li(GPR::AT, 0x00008678);
        assembler.write_li(GPR::A0, 0x00008321);
        assembler.write_li(GPR::A1, 0x00000084);

        assembler.write_ctc2(CP2FlagsRegister::VCO, GPR::AT);
        assembler.write_ctc2(CP2FlagsRegister::VCC, GPR::A0);
        assembler.write_ctc2(CP2FlagsRegister::VCE, GPR::A1);

        // Read registers using invalid indexes, all the way to 32
        for i in 0..32 {
            // Clear target register
            assembler.write_li(GPR::S0, 0);
            assembler.write_cfc2_any_index(u5::new(i), GPR::S0);
            assembler.write_sw(GPR::S0, GPR::R0, 4 * i as i16);
        }

        assembler.write_break();

        RSP::run_and_wait(0);

        for i in 0..32 {
            let expected = match i & 3 {
                0 => 0xFFFF8678,
                1 => 0xFFFF8321,
                _ => 0x00000084,
            };
            soft_assert_eq2(SPMEM::read(i * 4), expected, || format!("CTC2 {}, then read back", i).to_string())?;
        }

        Ok(())
    }
}

pub struct CTC2WeirdIndexes {}

impl Test for CTC2WeirdIndexes {
    fn name(&self) -> &str { "RSP CTC2 (index higher than three)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        // Set flags to known values so that we can query below through all 32 possible indexes
        assembler.write_li(GPR::AT, 0x00008678);
        assembler.write_li(GPR::A0, 0x00008321);
        assembler.write_li(GPR::A1, 0x00000084);
        assembler.write_ctc2(CP2FlagsRegister::VCO, GPR::AT);
        assembler.write_ctc2(CP2FlagsRegister::VCC, GPR::A0);
        assembler.write_ctc2(CP2FlagsRegister::VCE, GPR::A1);

        // CTC2 using every possible index, but read back using one of the first three
        for i in 0..32 {
            assembler.write_li(GPR::S0, i as u32);
            assembler.write_ctc2_any_index(u5::new(i), GPR::S0);
            assembler.write_cfc2_any_index(min(u5::new(i & 3), u5::new(3)), GPR::S1);
            assembler.write_sw(GPR::S1, GPR::R0, i as i16 * 4);
        }

        assembler.write_break();

        RSP::run_and_wait(0);

        for i in 0..32 {
            soft_assert_eq2(SPMEM::read(i * 4), i as u32, || format!("CTC2 {}, then read back via CFC2 {}", i, min(i & 3, 3)).to_string())?;
        }

        Ok(())
    }
}
