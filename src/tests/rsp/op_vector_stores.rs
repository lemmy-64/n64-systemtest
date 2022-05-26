use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::ops::RangeInclusive;

use crate::math::vector::Vector;
use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, GPR, RSPAssembler, VR};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq_vector;

// Store some data via a store instruction and verify it via the CPU
// Findings (for stores):
// - The element specifier specifies the starting element. If there isn't enough room after e,
//   there is wrap-around inside of the register (this is different from loads)
// - The following number of bytes will therefore be store:
//     - SQV: As many as needed until the end of the current 16 byte region
//     - SDV: 8 bytes
//     - SLV: 4 bytes
//     - SSV: 2 bytes
//     - SBV: 1 byte
// - SPV, SUV, SHV do packed storage, where a u16 of a register is stored into a u8 in memory (by shifting right 7/8 bits)
// - SFV is pretty complicated - depending on the E it pulls different source elements (or even 0 for some E)

fn test<FEmit: Fn(&mut RSPAssembler, VR, i32, GPR, E), FSimulate: Fn(&Vector, &[Vector; 4], u32, E) -> [Vector; 4]>(base_offset: usize, emit: FEmit, simulate: FSimulate) -> Result<(), String> {
    // Alignment and element specifiers to test. If we pass these, we'll probably pass everything
    const TEST_MISALIGNMENTS: RangeInclusive<u32> = 0..=15;
    const TEST_ELEMENTS: RangeInclusive<E> = E::_0..=E::_15;
    const REGISTER_DATA: Vector = Vector::from_u16([0x1776, 0x8378, 0xE1FE, 0x138F, 0xA42F, 0x156D, 0xCF20, 0x18E2]);

    // Prepare previous data: This is data that will be (partially) overwritten by the instruction
    const PREVIOUS_DATA: [Vector; 4] = [
        Vector::from_u16([0x1111, 0x1221, 0x1331, 0x1441, 0x1551, 0x1661, 0x1771, 0x1881]),
        Vector::from_u16([0x2112, 0x2222, 0x2332, 0x2442, 0x2552, 0x2662, 0x2772, 0x2882]),
        Vector::from_u16([0x3113, 0x3223, 0x3333, 0x3443, 0x3553, 0x3663, 0x3773, 0x3883]),
        Vector::from_u16([0x4114, 0x4224, 0x4334, 0x4444, 0x4554, 0x4664, 0x4774, 0x4884]),
    ];

    // 0x500: The data we actually will be writing. 0x510: Marker in registers before and after to ensure they don't get written
    SPMEM::write_vector_into_dmem(0x500, &REGISTER_DATA);
    SPMEM::write_vector_into_dmem(0x510, &Vector::from_u16([0xBADB, 0xADBA, 0xDBAD, 0xBADB, 0xADBA, 0xDBAD, 0xBADB, 0xADBA]));

    for misalignment in TEST_MISALIGNMENTS {
        for e in TEST_ELEMENTS {
            // Write PREVIOUS_DATA to the target address. Start one vector earlier in case the instructions writes downwards
            for (i, v) in PREVIOUS_DATA.iter().enumerate() {
                SPMEM::write_vector_into_dmem(base_offset + i * 0x10 - 0x10, v);
            }

            // Assemble RSP program
            let mut assembler = RSPAssembler::new(0);

            // Load initial data into V1 and BAD markers into V0 and V2 to ensure they don't get written
            assert!(base_offset < 0x400 || base_offset >= 0x600);
            assembler.write_li(GPR::AT, 0x500);
            assembler.write_lqv(VR::V0, E::_0, 0x10, GPR::AT);
            assembler.write_lqv(VR::V1, E::_0, 0x00, GPR::AT);
            assembler.write_lqv(VR::V2, E::_0, 0x10, GPR::AT);

            assembler.write_li(GPR::A0, (base_offset as u32 + misalignment) - 0x10);

            // actual store for testing
            emit(&mut assembler, VR::V1, 0x10, GPR::A0, e);

            assembler.write_break();

            RSP::run_and_wait(0);

            let expected = simulate(&REGISTER_DATA, &PREVIOUS_DATA, 0x10 as u32 + misalignment, e);
            for i in 0..4 {
                let address = (base_offset + i * 0x10 - 0x10) & 0xFFF;
                soft_assert_eq_vector(SPMEM::read_vector_from_dmem(address), expected[i], || format!("Store with e={:?} into memory location 0x{:x}. Checking address 0x{:x}", e, base_offset as u32 + misalignment, address))?;
            }
        }
    }

    Ok(())
}

fn test_simple<FEmit: Fn(&mut RSPAssembler, VR, i32, GPR, E), FGetMaxBytes: Fn(u32) -> u32>(base_offset: usize, emit: FEmit, get_maximum_bytes_from_offset: FGetMaxBytes) -> Result<(), String> {
    test(base_offset, emit, |source, previous_data, offset, e| {
        let mut result = *previous_data;
        let remaining_bytes_from_offset = get_maximum_bytes_from_offset(offset);
        for i in 0..remaining_bytes_from_offset {
            let result_vector_index = ((offset + i) >> 4) as usize;
            let target_element_index = ((offset + i) as usize) & 0xF;
            let source_element_index = ((e as usize) + (i as usize)) & 0xF;
            result[result_vector_index].set8(target_element_index, source.get8(source_element_index));
        }
        result
    })
}

pub struct SBV {}

impl Test for SBV {
    fn name(&self) -> &str { "RSP SBV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0x1000, |assembler, vt, offset, base, e| assembler.write_sbv(vt, e, offset, base),
                    |_offset| 1)
    }
}

pub struct SSV {}

impl Test for SSV {
    fn name(&self) -> &str { "RSP SSV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0x1000, |assembler, vt, offset, base, e| assembler.write_ssv(vt, e, offset, base),
                    |_offset| 2)
    }
}

pub struct SLV {}

impl Test for SLV {
    fn name(&self) -> &str { "RSP SLV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0x1000, |assembler, vt, offset, base, e| assembler.write_slv(vt, e, offset, base),
                    |_offset| 4)
    }
}

pub struct SDV {}

impl Test for SDV {
    fn name(&self) -> &str { "RSP SDV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0x1000, |assembler, vt, offset, base, e| assembler.write_sdv(vt, e, offset, base),
                    |_offset| 8)
    }
}

pub struct SQV {}

impl Test for SQV {
    fn name(&self) -> &str { "RSP SQV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0x1000, |assembler, vt, offset, base, e| assembler.write_sqv(vt, e, offset, base),
                    |offset| 16 - (offset & 0xF))
    }
}

pub struct SRV {}

impl Test for SRV {
    fn name(&self) -> &str { "RSP SRV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for base_offset in [0x1000, 0xFF0, 0xFE0] {
            test(base_offset, |assembler, vt, offset, base, e| assembler.write_srv(vt, e, offset, base),
                 |source, previous_data, offset, e| {
                     let mut result = *previous_data;

                     for i in (16 - (offset & 0xF))..16 {
                         let result_vector_index = ((offset - 16 + i) >> 4) as usize;
                         let target_element_index = ((offset + i) as usize) & 0xF;
                         let source_element_index = ((e as usize) + (i as usize)) & 0xF;

                         result[result_vector_index].set8(target_element_index, source.get8(source_element_index));
                     }

                     result
                 })?;
        }

        Ok(())
    }
}

fn test_pack<FEmit: Fn(&mut RSPAssembler, VR, i32, GPR, E), const SHIFT1: u32, const SHIFT2: u32>(base_offset: usize, emit: FEmit) -> Result<(), String> {
    test(base_offset, emit, |source, previous_data, offset, e| {
        let mut result = *previous_data;

        for i in 0..8 {
            let element_index = e as usize + i;
            let shift = if (element_index & 8) == 0 { SHIFT1 } else { SHIFT2 };

            let data16 = source.get16(element_index & 0x7) as u16;
            let data = (data16 >> shift) as u8;

            let result_vector_index = (offset as usize + i) >> 4;
            let target_element_index = (offset as usize + i) & 0xF;

            result[result_vector_index].set8(target_element_index, data);
        }
        result
    })
}

pub struct SPV {}

impl Test for SPV {
    fn name(&self) -> &str { "RSP SPV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_pack::<_, 8, 7>(0x1000, |assembler, vt, offset, base, e| assembler.write_spv(vt, e, offset, base))
    }
}

pub struct SUV {}

impl Test for SUV {
    fn name(&self) -> &str { "RSP SUV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_pack::<_, 7, 8>(0x1000, |assembler, vt, offset, base, e| assembler.write_suv(vt, e, offset, base))
    }
}

pub struct SHV {}

impl Test for SHV {
    fn name(&self) -> &str { "RSP SHV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(0x1000,
             |assembler, vt, offset, base, e| assembler.write_shv(vt, e, offset, base),
             |source, previous_data, offset, e| {
                 let mut result = *previous_data;

                 let a = (offset & 7) as usize;
                 let b = ((offset as usize) - a) as usize;
                 for i in 0..8 {
                     let element_index = (e as usize) + (i << 1);

                     let data16 = ((source.get8(element_index & 0xF) as u16) << 8) | (source.get8((element_index + 1) & 0xF) as u16);
                     let data = (data16 >> 7) as u8;

                     let addr = b + ((a + (i << 1)) & 0xF);
                     let result_vector_index = addr >> 4;
                     let target_element_index = addr & 0xF;

                     result[result_vector_index].set8(target_element_index, data);
                 }
                 result
             })
    }
}

pub struct SFV {}

impl Test for SFV {
    fn name(&self) -> &str { "RSP SFV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(0x1000,
             |assembler, vt, offset, base, e| assembler.write_sfv(vt, e, offset, base),
             |source, previous_data, offset, e| {
                 let mut result = *previous_data;

                 let a = (offset & 7) as usize;
                 let b = ((offset as usize) - a) as usize;
                 // The starting element depends on E. The three next ones can be determined by adding 1,2,3, but staying within the vector half:
                 //   (element_index & 4) | ((element_index + i) & 3))
                 // For readability, spelling them out here.
                 let maybe_source_element_offsets = match e {
                     E::_0 => Some([0, 1, 2, 3]),
                     E::_1 => Some([6, 7, 4, 5]),
                     E::_4 => Some([1, 2, 3, 0]),
                     E::_5 => Some([7, 4, 5, 6]),
                     E::_8 => Some([4, 5, 6, 7]),
                     E::_11 => Some([3, 0, 1, 2]),
                     E::_12 => Some([5, 6, 7, 4]),
                     E::_15 => Some([0, 1, 2, 3]),
                     _ => None
                 };

                 for i in 0..4 {
                     let data = if let Some(source_element_offsets) = maybe_source_element_offsets {
                         let element_index = source_element_offsets[i];
                         let data16 = source.get16(element_index);
                         (data16 >> 7) as u8
                     } else {
                         0
                     };

                     let addr = b + ((a + (i << 2)) & 0xF);
                     let result_vector_index = addr >> 4;
                     let target_element_index = addr & 0xF;

                     result[result_vector_index].set8(target_element_index, data);
                 }
                 result
             })
    }
}
