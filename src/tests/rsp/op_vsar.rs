use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, Element, GPR, RSPAssembler, VR};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Test VSAR

pub struct VSAR {}

impl Test for VSAR {
    fn name(&self) -> &str { "RSP VSAR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Prepare input data
        SPMEM::write_vector_16(0x00, &[0xEEEE, 0xFFFF, 0xDDDD, 0xCCCC, 0xBBBB, 0xAAAA, 0x9999, 0x8888]);
        SPMEM::write_vector_16(0x10, &[0x0010, 0x0001, 0xFFF1, 0x0200, 0xF1E2, 0x0810, 0x7FFF, 0x8100]);
        SPMEM::write_vector_16(0x20, &[0x0020, 0x0002, 0xFFF2, 0x0300, 0xF2E2, 0x0820, 0x7FFF, 0x8200]);
        SPMEM::write_vector_16(0x30, &[0x0030, 0x0003, 0xFFF3, 0x0400, 0xF3E2, 0x0830, 0x7FFF, 0x8300]);

        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
        assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);
        assembler.write_lqv(VR::V2, E::_0, 0x020, GPR::R0);
        assembler.write_lqv(VR::V3, E::_0, 0x030, GPR::R0);

        // Perform a multiplication so that something non-random is in the accumulator
        assembler.write_vmulf(VR::V4, VR::V1, VR::V2, Element::All);

        for i in 0..15 {
            // Fill the target register with junk (to see whether it gets overwritten)
            let vt = VR::from_index(5 + i);
            assembler.write_lqv(vt,  E::_0, 0x000, GPR::R0);
            assembler.write_vsar(vt, VR::V0, VR::V0, E::from_index(i));
            assembler.write_sqv(vt, E::_0, (0x100 + i * 16) as i32, GPR::R0);
        }

        assembler.write_break();

        RSP::run_and_wait(0);

        for i in 0..15 {
            let expected;
            match i {
                8 => {
                    expected = [0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000];
                },
                9 => {
                    expected = [0x0000, 0x0000, 0x0000, 0x000C, 0x0172, 0x0083, 0x7FFE, 0x7D04];
                },
                10 => {
                    expected = [0x8400, 0x8004, 0x81A4, 0x8000, 0xDB08, 0x8400, 0x8002, 0x8000];
                },
                _ => {
                    expected = [0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000];
                },
            }
            soft_assert_eq(SPMEM::read_vector_16(0x100 + i * 16), expected, format!("VSAR e={}", i).as_str())?;
        }

        Ok(())
    }
}
