use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::cmp::min;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, GPR, RSPAssembler, VR};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq2};

// Load some data via a load instruction and store it back via SQV. Then verify the result
// Findings (for loads):
// - The element specifier specifies the starting element. If there isn't enough room after e,
//   there is no wrap-around but the number of bytes load is reduced
// - Apart from the "enough room after e" restriction above, the following number of bytes is being loaded:
//     - LQV: Loads until the end of the current 16 byte region
//     - LDV: 8 bytes
//     - LLV: 4 bytes
//     - LSV: 2 bytes
//     - LBV: 1 byte

// TODO: Tests for remaining loads
// TODO: Write dmem-overflow tests for LDB, LLB, LSV (the others can't overflow, but we might verify that, too)
// TODO: Do the same for stores

fn test<F: Fn(&mut RSPAssembler,E), F2: Fn(u32) -> u32>(load_emitter: F, maximum_bytes_from_offset: F2) -> Result<(), String> {
    // Alignment and element specifiers to test. If we pass these, we'll probably pass everything
    const TEST_ALIGNMENTS: [u32; 6] = [0, 1, 5, 6, 7, 10];
    const TEST_ELEMENT: [E; 8] = [E::_0, E::_1, E::_4, E::_5, E::_6, E::_12, E::_14, E::_15];
    const OUTPUT_MEMORY_START: u32 = 0x050;

    // Prepare input data
    let clear_vector: [u8; 16] = [0x88, 0x88, 0x99, 0x99, 0xAA, 0xAA, 0xBB, 0xBB, 0xCC, 0xCC, 0xDD, 0xDD, 0xEE, 0xEE, 0xFF, 0xFF];
    let test_vector: [u8; 16] = [0x00, 0x01, 0x11, 0x12, 0x22, 0x23, 0x33, 0x34, 0x44, 0x45, 0x55, 0x56, 0x66, 0x67, 0x77, 0x76];
    let test_vector2: [u8; 16] = [0x65, 0x64, 0x63, 0x62, 0x61, 0x60, 0x54, 0x53, 0x52, 0x51, 0x43, 0x42, 0x41, 0x32, 0x31, 0x21];

    SPMEM::write_vector_8(0x000, &clear_vector);
    SPMEM::write_vector_8(0x010, &test_vector);
    SPMEM::write_vector_8(0x020, &test_vector2);

    // Assemble RSP program
    let mut assembler = RSPAssembler::new(0);

    // Guard V0 and V1 by clearing them and verifying them below as well
    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V2, E::_0, 0x000, GPR::R0);

    // A0: Offset in memory to read from
    // A1: Address to write result to
    assembler.write_li(GPR::A1, OUTPUT_MEMORY_START);
    for offset in TEST_ALIGNMENTS {
        assembler.write_li(GPR::A0, offset);
        for e in TEST_ELEMENT {
            // clear
            assembler.write_lqv(VR::V1, E::_0, 0x000, GPR::R0);
            // load
            load_emitter(&mut assembler, e);
            // save result
            assembler.write_sqv(VR::V1, E::_0, 0x000, GPR::A1);
            assembler.write_addiu(GPR::A1, GPR::A1, 0x10);
        }
    }

    // Emulators might write out-of-bounds and accidentally modify the next register. Verify by writing back V1
    assembler.write_sqv(VR::V0, E::_0, 0x030, GPR::R0);
    assembler.write_sqv(VR::V2, E::_0, 0x040, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq(SPMEM::read_vector_8(0x030), clear_vector, "V0 was modified eventhough it wasn't being written to")?;
    soft_assert_eq(SPMEM::read_vector_8(0x040), clear_vector, "V2 was modified eventhough it wasn't being written to")?;


    let mut memory_address = OUTPUT_MEMORY_START as usize;
    for offset in TEST_ALIGNMENTS {
        assembler.write_li(GPR::A0, offset);
        for e in TEST_ELEMENT {
            let mut expected = clear_vector;
            let remaining_in_vector = 16 - (e as u32);
            let remaining_bytes_from_offset = maximum_bytes_from_offset(offset);
            for i in 0..min(remaining_in_vector, remaining_bytes_from_offset) {
                let source_offset = (offset + i) as usize;
                let soure_value = if source_offset >= 16 { test_vector2[source_offset - 16] } else { test_vector[source_offset] };
                expected[(e as usize) + i as usize] = soure_value;
            }
            soft_assert_eq2(SPMEM::read_vector_8(memory_address), expected, || format!("Load with e={:?} from memory location {}", e, offset))?;
            memory_address += 0x10;
        }
    }


    Ok(())
}

pub struct LBV {}

impl Test for LBV {
    fn name(&self) -> &str { "RSP LBV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(|assembler, e| assembler.write_lbv(VR::V1, e, 0x010, GPR::A0),
             |_offset| 1)
    }
}

pub struct LSV {}

impl Test for LSV {
    fn name(&self) -> &str { "RSP LSV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(|assembler, e| assembler.write_lsv(VR::V1, e, 0x010, GPR::A0),
             |_offset| 2)
    }
}

pub struct LLV {}

impl Test for LLV {
    fn name(&self) -> &str { "RSP LLV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(|assembler, e| assembler.write_llv(VR::V1, e, 0x010, GPR::A0),
             |_offset| 4)
    }
}

pub struct LDV {}

impl Test for LDV {
    fn name(&self) -> &str { "RSP LDV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(|assembler, e| assembler.write_ldv(VR::V1, e, 0x010, GPR::A0),
             |_offset| 8)
    }
}

pub struct LQV {}

impl Test for LQV {
    fn name(&self) -> &str { "RSP LQV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(|assembler, e| assembler.write_lqv(VR::V1, e, 0x010, GPR::A0),
             |offset| 16 - (offset & 0xF))
    }
}
