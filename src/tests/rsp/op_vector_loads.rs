use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::cmp::min;

use arrayref::array_ref;

use crate::math::vector::Vector;
use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, GPR, RSPAssembler, VR};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq2, soft_assert_eq_vector};

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
//  - LPV loads 1 bytes into the higher half of a word. It has some weird element-handling wrt. overflow
//  - LUV is identical to LPV, but the result is shifted down by 1 bit
//  - LHV is close to LUV, but the source index is multiplied by 2. Also the offset is shifted by 4 instead of 3
//  - LFV is complicated
//  - LWV doesn't exist - it does nothing
//  - LTV loads data into multiple vectors (touching always a quarter of the vectors at a time)
// Only three instructions can overflow (meaning: read from the end and the beginning of DMEM): LSV, LLV, LDB.
// All the others use alignment to stay with 16b.

fn test<F: Fn(&mut RSPAssembler, E), F2: Fn(&mut [u8; 16], &[u8; 256], E, u32)>(base_offset: usize, load_emitter: F, calculate_expected: F2) -> Result<(), String> {
    // Alignment and element specifiers to test. If we pass these, we'll probably pass everything
    const TEST_MISALIGNMENTS: [u32; 11] = [0, 1, 6, 7, 8, 10, 11, 12, 13, 14, 15];
    const TEST_ELEMENTS: [E; 9] = [E::_0, E::_1, E::_4, E::_5, E::_6, E::_8, E::_12, E::_14, E::_15];
    const OUTPUT_MEMORY_START: u32 = 0x100;

    // Prepare input data: Each byte will simply be its index
    let mut test_data: [u8; 256] = [0; 256];
    for i in 0..test_data.len() {
        test_data[i] = i as u8;
    }
    let clear_vector: [u8; 16] = *array_ref![test_data, 0, 16];

    // Write into DMEM
    SPMEM::write_vector8_into_dmem(base_offset, &test_data);

    // Assemble RSP program
    let mut assembler = RSPAssembler::new(0);

    // Guard V0 and V1 by clearing them and verifying them below as well
    assembler.write_li(GPR::AT, base_offset as u32);
    assembler.write_lqv(VR::V0, E::_0, 0, GPR::AT);
    assembler.write_lqv(VR::V2, E::_0, 0, GPR::AT);

    // AT: base_offset (e.g. 0 or 0xFF0 for overflow tests)
    // A0: Offset in memory to read from
    // A1: Address to write result to
    assembler.write_li(GPR::A1, OUTPUT_MEMORY_START);
    for misalignment in TEST_MISALIGNMENTS {
        assembler.write_li(GPR::A0, base_offset as u32 + misalignment);
        for e in TEST_ELEMENTS {
            // clear
            assembler.write_lqv(VR::V1, E::_0, 0x000, GPR::AT);
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

    soft_assert_eq(SPMEM::read_vector8_from_dmem(0x030), clear_vector, "V0 was modified eventhough it wasn't being written to")?;
    soft_assert_eq(SPMEM::read_vector8_from_dmem(0x040), clear_vector, "V2 was modified eventhough it wasn't being written to")?;


    let mut memory_address = OUTPUT_MEMORY_START as usize;
    for offset in TEST_MISALIGNMENTS {
        assembler.write_li(GPR::A0, offset);
        for e in TEST_ELEMENTS {
            let mut expected = clear_vector;
            calculate_expected(&mut expected, &test_data, e, offset);
            soft_assert_eq2(SPMEM::read_vector8_from_dmem(memory_address), expected, || format!("Load with e={:?} from memory location {:x}", e, base_offset as u32 + offset))?;
            memory_address += 0x10;
        }
    }

    Ok(())
}

fn test_simple<F: Fn(&mut RSPAssembler, E), F2: Fn(u32) -> u32>(base_offset: usize, load_emitter: F, maximum_bytes_from_offset: F2) -> Result<(), String> {
    test(base_offset, load_emitter, |expected, test_data, e, offset| {
        let remaining_in_vector = 16 - (e as u32);
        let remaining_bytes_from_offset = maximum_bytes_from_offset(offset);
        for i in 0..min(remaining_in_vector, remaining_bytes_from_offset) {
            expected[(e as usize) + i as usize] = test_data[(offset + i) as usize + 0x20];
        }
    })
}

fn test_unpack<F: Fn(&mut RSPAssembler, E), const SHIFT: u32, const IFACTOR: u32>(base_offset: usize, load_emitter: F) -> Result<(), String> {
    test(base_offset, load_emitter,
         |expected, test_data, e, offset| {
             let misalignment = offset & 7;
             let aligned_offset = offset & !7;
             for i in 0..8 {
                 let element_offset = (16 - (e as u32) + ((i as u32) * IFACTOR) + misalignment) & 0xF;
                 let address = 0x20 + aligned_offset + element_offset;
                 let data = ((test_data[address as usize]) as u16) << SHIFT;
                 expected[i * 2] = (data >> 8) as u8;
                 expected[i * 2 + 1] = data as u8;
             }
         })
}

pub struct LBV {}

impl Test for LBV {
    fn name(&self) -> &str { "RSP LBV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0x1000, |assembler, e| assembler.write_lbv(VR::V1, e, 0x020, GPR::A0),
                    |_offset| 1)
    }
}

pub struct LSV {}

impl Test for LSV {
    fn name(&self) -> &str { "RSP LSV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0x1000, |assembler, e| assembler.write_lsv(VR::V1, e, 0x020, GPR::A0),
                    |_offset| 2)
    }
}

pub struct LSVOverflow {}

impl Test for LSVOverflow {
    fn name(&self) -> &str { "RSP LSV (overflow)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0xFD0, |assembler, e| assembler.write_lsv(VR::V1, e, 0x020, GPR::A0),
                    |_offset| 2)
    }
}

pub struct LLV {}

impl Test for LLV {
    fn name(&self) -> &str { "RSP LLV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0x1000, |assembler, e| assembler.write_llv(VR::V1, e, 0x020, GPR::A0),
                    |_offset| 4)
    }
}

pub struct LLVOverflow {}

impl Test for LLVOverflow {
    fn name(&self) -> &str { "RSP LLV (overflow)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0xFD0, |assembler, e| assembler.write_llv(VR::V1, e, 0x020, GPR::A0),
                    |_offset| 4)
    }
}

pub struct LDV {}

impl Test for LDV {
    fn name(&self) -> &str { "RSP LDV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0x1000, |assembler, e| assembler.write_ldv(VR::V1, e, 0x020, GPR::A0),
                    |_offset| 8)
    }
}

pub struct LDVOverflow {}

impl Test for LDVOverflow {
    fn name(&self) -> &str { "RSP LDV (overflow)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0xFD0, |assembler, e| assembler.write_ldv(VR::V1, e, 0x020, GPR::A0),
                    |_offset| 8)
    }
}

pub struct LQV {}

impl Test for LQV {
    fn name(&self) -> &str { "RSP LQV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0x1000, |assembler, e| assembler.write_lqv(VR::V1, e, 0x020, GPR::A0),
                    |offset| 16 - (offset & 0xF))
    }
}

// LQV can not overflow, but we can test the behavior at the very end
pub struct LQVEndOfDMEM {}

impl Test for LQVEndOfDMEM {
    fn name(&self) -> &str { "RSP LQV (end of DMEM)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_simple(0xFD0, |assembler, e| assembler.write_lqv(VR::V1, e, 0x020, GPR::A0),
                    |offset| 16 - (offset & 0xF))
    }
}

pub struct LRV {}

impl Test for LRV {
    fn name(&self) -> &str { "RSP LRV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(0x1000, |assembler, e| assembler.write_lrv(VR::V1, e, 0x020, GPR::A0),
             |expected, test_data, e, offset| {
                 let bytes_from_offset = offset & 0xF;
                 for i in (16 - bytes_from_offset)..16 {
                     if (e as u32) + i > 15 {
                         break;
                     }
                     expected[(e as usize) + i as usize] = test_data[(16 + offset + i) as usize];
                 }
             })
    }
}

// LRV can not overflow, but we can test the behavior at the very start of DMEM
pub struct LRVStartOfDMEM {}

impl Test for LRVStartOfDMEM {
    fn name(&self) -> &str { "RSP LRV (start of DMEM)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(0xFE0, |assembler, e| assembler.write_lrv(VR::V1, e, 0x020, GPR::A0),
             |expected, test_data, e, offset| {
                 let bytes_from_offset = offset & 0xF;
                 for i in (16 - bytes_from_offset)..16 {
                     if (e as u32) + i > 15 {
                         break;
                     }
                     expected[(e as usize) + i as usize] = test_data[(16 + offset + i) as usize];
                 }
             })
    }
}

pub struct LPV {}

impl Test for LPV {
    fn name(&self) -> &str { "RSP LPV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_unpack::<_, 8, 1>(0x1000, |assembler, e| assembler.write_lpv(VR::V1, e, 0x020, GPR::A0))
    }
}

pub struct LPVEndOfDMEM {}

impl Test for LPVEndOfDMEM {
    fn name(&self) -> &str { "RSP LPV (end of DMEM)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_unpack::<_, 8, 1>(0xFD0, |assembler, e| assembler.write_lpv(VR::V1, e, 0x020, GPR::A0))
    }
}

pub struct LUV {}

impl Test for LUV {
    fn name(&self) -> &str { "RSP LUV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_unpack::<_, 7, 1>(0x1000, |assembler, e| assembler.write_luv(VR::V1, e, 0x020, GPR::A0))
    }
}

pub struct LUVEndOfDMEM {}

impl Test for LUVEndOfDMEM {
    fn name(&self) -> &str { "RSP LUV (end of DMEM)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_unpack::<_, 7, 1>(0xFD0, |assembler, e| assembler.write_luv(VR::V1, e, 0x020, GPR::A0))
    }
}

pub struct LHV {}

impl Test for LHV {
    fn name(&self) -> &str { "RSP LHV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_unpack::<_, 7, 2>(0x1000, |assembler, e| assembler.write_lhv(VR::V1, e, 0x020, GPR::A0))
    }
}

pub struct LHVEndOfDMEM {}

impl Test for LHVEndOfDMEM {
    fn name(&self) -> &str { "RSP LHV (end of DMEM)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_unpack::<_, 7, 2>(0xFD0, |assembler, e| assembler.write_lhv(VR::V1, e, 0x020, GPR::A0))
    }
}

pub struct LFV {}

impl Test for LFV {
    fn name(&self) -> &str { "RSP LFV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(0, |assembler, e| assembler.write_lfv(VR::V1, e, 0x020, GPR::A0),
             |expected, test_data, e, offset| {
                 let mut temp = Vector::new();
                 let address = 0x20 + offset as usize;
                 let aligned_address = address & !0x7;
                 let misalignment = address & 0x7;
                 let e_ = e as usize;

                 temp.set16(0, (test_data[aligned_address + ((misalignment + e_) & 0xF)] as u16) << 7);
                 temp.set16(1, (test_data[aligned_address + ((misalignment + 4 - e_) & 0xF)] as u16) << 7);
                 temp.set16(2, (test_data[aligned_address + ((misalignment + 8 - e_) & 0xF)] as u16) << 7);
                 temp.set16(3, (test_data[aligned_address + ((misalignment + 12 - e_) & 0xF)] as u16) << 7);
                 temp.set16(4, (test_data[aligned_address + ((misalignment + 8 - e_) & 0xF)] as u16) << 7);
                 temp.set16(5, (test_data[aligned_address + ((misalignment + 12 - e_) & 0xF)] as u16) << 7);
                 temp.set16(6, (test_data[aligned_address + ((misalignment - e_) & 0xF)] as u16) << 7);
                 temp.set16(7, (test_data[aligned_address + ((misalignment + 4 - e_) & 0xF)] as u16) << 7);
                 let length = min(8, 16 - e_);
                 for i in e_..length + e_ {
                     expected[i] = temp.get8(i);
                 }
             })
    }
}

pub struct LFVZeroExtended {}

impl Test for LFVZeroExtended {
	fn name(&self) -> &str { "RSP LFV Zero Extended" }
	
	fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(0, |assembler, e| assembler.write_lfv(VR::V1, e, 0x0c0, GPR::A0),
             |expected, test_data, e, offset| {
                 let mut temp = Vector::new();
                 let address = 0xc0 + offset as usize;
                 let aligned_address = address & !0x7;
                 let misalignment = address & 0x7;
                 let e_ = e as usize;

                 temp.set16(0, (test_data[aligned_address + ((misalignment + e_) & 0xF)] as u16) << 7);
                 temp.set16(1, (test_data[aligned_address + ((misalignment + 4 - e_) & 0xF)] as u16) << 7);
                 temp.set16(2, (test_data[aligned_address + ((misalignment + 8 - e_) & 0xF)] as u16) << 7);
                 temp.set16(3, (test_data[aligned_address + ((misalignment + 12 - e_) & 0xF)] as u16) << 7);
                 temp.set16(4, (test_data[aligned_address + ((misalignment + 8 - e_) & 0xF)] as u16) << 7);
                 temp.set16(5, (test_data[aligned_address + ((misalignment + 12 - e_) & 0xF)] as u16) << 7);
                 temp.set16(6, (test_data[aligned_address + ((misalignment - e_) & 0xF)] as u16) << 7);
                 temp.set16(7, (test_data[aligned_address + ((misalignment + 4 - e_) & 0xF)] as u16) << 7);
                 let length = min(8, 16 - e_);
                 for i in e_..length + e_ {
                     expected[i] = temp.get8(i);
                 }
             })
    }
}

pub struct LWV {}

impl Test for LWV {
    fn name(&self) -> &str { "RSP LWV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test(0, |assembler, e| assembler.write_lwv(VR::V1, e, 0x020, GPR::A0),
             |_, _, _, _| {
                 // This doesn't seem to exist - the register will be unchanged
             })
    }
}

pub struct LTV {}

impl Test for LTV {
    fn name(&self) -> &str { "RSP LTV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // There are 16 values for E, 32 target registers and 16 possible misalignments, resulting in 8192 tests.
        // Let's cut this down somewhat to 576
        const TEST_ELEMENT: [E; 8] = [E::_0, E::_1, E::_2, E::_7, E::_8, E::_9, E::_14, E::_15];
        const TEST_OFFSETS: [u32; 8] = [0, 1, 2, 7, 8, 14, 15, 16];
        const TEST_VT: [VR; 9] = [VR::V0, VR::V1, VR::V2, VR::V7, VR::V8, VR::V9, VR::V18, VR::V25, VR::V31];

        let mut test_data: [u8; 256] = [0; 256];
        for i in 0..test_data.len() {
            test_data[i] = i as u8;
        }
        let clear_vector= Vector::from_u8(*array_ref![test_data, 0, 16]);
        SPMEM::write_vector8_into_dmem(0, &test_data);

        for offset in TEST_OFFSETS {
            for vt in TEST_VT {
                for e in TEST_ELEMENT {
                    let mut assembler = RSPAssembler::new(0);

                    // Clear all registers with 0, 1, 2, 3, 4, 5, 6, 7, 8
                    for vr in VR::V0..=VR::V31 {
                        assembler.write_lqv(vr, E::_0, 0x000, GPR::R0);
                    }

                    // Load register via LTV
                    assembler.write_li(GPR::A0, offset);
                    assembler.write_ltv(vt, e, 0x20, GPR::A0);

                    // Write back all registers
                    for vr in VR::V0..=VR::V31 {
                        assembler.write_sqv(vr, E::_0, (0x100 + vr.index() * 0x10) as i32, GPR::R0);
                    }
                    assembler.write_break();
                    RSP::start_running(0);

                    // Simulate on the CPU
                    let mut expected: [Vector; 32] = [clear_vector; 32];

                    let vt_base_index = vt.index() & !7;
                    // Every other vector is rotated by 8 bytes
                    let odd_offset =  if (offset & 8) != 0 { 8 } else { 0 };
                    let base_address = 0x20 + ((offset as usize) & !7);
                    for i in 0..8 {
                        let reg_offset = ((e.index() >> 1) + i) & 0x7;
                        expected[vt_base_index + reg_offset].set8(i * 2, test_data[base_address + ((odd_offset + e.index() + i * 2) & 0xF)]);
                        expected[vt_base_index + reg_offset].set8(i * 2 + 1, test_data[base_address + ((odd_offset + e.index() + i * 2 + 1) & 0xF)]);
                    }

                    RSP::wait_until_rsp_is_halted();

                    // Verify results
                    for i in 0..31 {
                        soft_assert_eq_vector(SPMEM::read_vector_from_dmem(0x100 + i * 0x10), expected[i], || format!("Register[{}] after LTV {:?}[{}] from 0x{:x}", i, vt, e.index(), 0x20 + offset))?;
                    }
                }
            }
        }
        Ok(())
    }
}

