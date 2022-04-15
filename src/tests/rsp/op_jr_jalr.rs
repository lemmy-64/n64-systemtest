use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

fn test(jalr: bool, change_target_address_in_delay: bool, jalr_change_ra_in_delay: bool, jalr_ra: GPR) -> Result<(), String> {
    // Assemble RSP program at end of DMEM
    const START_OFFSET: usize = 0xFE4;
    let mut assembler = RSPAssembler::new(START_OFFSET);

    assembler.write_ori(GPR::S0, GPR::R0, 0);
    assembler.write_ori(GPR::S1, GPR::R0, 0);
    assembler.write_ori(GPR::S2, GPR::R0, 0);
    assembler.write_ori(GPR::RA, GPR::R0, 0x1234);
    assembler.write_lui(GPR::AT, 0xFFFF);
    assembler.write_ori(GPR::AT, GPR::AT, 0xF00B); // target address (0x008) with unnecessary bits

    assert!(assembler.writer().offset() == 0xFFC);

    if jalr {
        assembler.write_jalr(jalr_ra, GPR::AT);
    } else {
        assembler.write_jr(GPR::AT);
    }

    // 0x000: Delay slot (after overflow)
    assert!(!(change_target_address_in_delay && jalr_change_ra_in_delay));
    if change_target_address_in_delay {
        assembler.write_ori(GPR::AT, GPR::R0, 0x0004);
    } else if jalr_change_ra_in_delay {
        assert!(jalr);
        assembler.write_ori(jalr_ra, GPR::R0, 0x7654);
    } else {
        assembler.write_addiu(GPR::S0, GPR::S0, 1);
    }

    // 0x004: This is skipped
    assembler.write_addiu(GPR::S1, GPR::S1, 1);

    // 0x008: Jump target
    assembler.write_addiu(GPR::S2, GPR::S2, 1);

    assembler.write_sw(GPR::S0, GPR::R0, 0x0);
    assembler.write_sw(GPR::S1, GPR::R0, 0x4);
    assembler.write_sw(GPR::S2, GPR::R0, 0x8);
    assembler.write_sw(jalr_ra, GPR::R0, 0xC);

    assembler.write_break();

    RSP::run_and_wait(START_OFFSET);

    if !change_target_address_in_delay {
        soft_assert_eq(SPMEM::read(0x0), 1, "Delay slot in 0x000 is expected to executed")?;
    }
    soft_assert_eq(SPMEM::read(0x4), 0, "Instruction at 0x004 is expected to be skipped")?;
    soft_assert_eq(SPMEM::read(0x8), 1, "Instruction at 0x008 is expected to be executed")?;
    if jalr {
        soft_assert_eq(SPMEM::read(0xC), 0x004, "JALR's return address not valid")?;
    } else {
        soft_assert_eq(SPMEM::read(0xC), 0x1234, "RA must not be changed")?;
    }

    Ok(())
}

pub struct JR {}

impl Test for JR {
    fn name(&self) -> &str { "RSP JR: Simple" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(false, false, false, GPR::RA)
    }
}

pub struct JRWithRegisterChangeInDelaySlot {}

impl Test for JRWithRegisterChangeInDelaySlot {
    fn name(&self) -> &str { "RSP JR: Register change in delay slot" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(false, true, false, GPR::RA)
    }
}

pub struct JALR {}

impl Test for JALR {
    fn name(&self) -> &str { "RSP JALR: Simple" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(true, false, false, GPR::V0)
    }
}

pub struct JALRWithRegisterChangeInDelaySlot {}

impl Test for JALRWithRegisterChangeInDelaySlot {
    fn name(&self) -> &str { "RSP JALR: Register change in delay slot" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(true, true, false, GPR::V0)
    }
}

pub struct JALRWithReturnAddressChangeInDelaySlot {}

impl Test for JALRWithReturnAddressChangeInDelaySlot {
    fn name(&self) -> &str { "RSP JALR: Return address in delay slot" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(true, false, true, GPR::V0)
    }
}

pub struct JALRWithReturnAddressEqualToTargetAddress {}

impl Test for JALRWithReturnAddressEqualToTargetAddress {
    fn name(&self) -> &str { "RSP JALR: Return register is equal to target register" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(true, false, false, GPR::AT)
    }
}
