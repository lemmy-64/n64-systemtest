use alloc::format;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::{print, println};
use crate::math::vector::Vector;
use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, Element, GPR, RSPAssembler, VR, VSARAccumulator};
use crate::rsp::rsp_macros::assemble_set_accumulator_to;
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq2, soft_assert_eq_vector};

// The generation of the RCP and RSP tables was ported from Ares: https://github.com/ares-emulator/ares/blob/acd2130a4d4c9e7208f61e0ff762895f7c9b8dc6/ares/n64/rsp/rsp.cpp#L102
// which uses the following license:

// Copyright (c) 2004-2021 ares team, Near et al
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

const fn rcp_table_value(index: usize) -> u16 {
    if index == 0 {
        return 0xFFFF;
    }

    ((((1u64 << 34) / ((index as u64) + 512)) + 1) >> 8) as u16
}

const fn rsq_table_value(index: usize) -> u16 {
    // The basic idea for this algorithm was taken from Ares, but modified:
    // - indexes are different to match the algorithm here
    // - using the increment loop this function was made faster; it is now fast enough to run
    //   in a const context
    let a = (if index < 256 { index + 256 } else { ((index - 256) << 1) + 512 }) as u64;
    let mut b = 1u64 << 17;
    // find the largest b where b < 1.0 / sqrt(a)
    let mut increment = 512;
    while increment != 0 {
        while a * (b + increment) * (b + increment) < (1u64 << 44) {
            b += increment;
        }
        increment >>= 1;
    }
    (b >> 1) as u16
}

const fn make_rcp_table() -> [u16; 512] {
    let mut result = [0u16; 512];
    let mut i = 0;
    while i < 512 {
        result[i] = rcp_table_value(i);
        i += 1;
    }
    result
}

const fn make_rsq_table() -> [u16; 512] {
    let mut result = [0u16; 512];
    let mut i = 0;
    while i < 512 {
        result[i] = rsq_table_value(i);
        i += 1;
    }
    result
}

const RCP_DATA: [u16; 512] = make_rcp_table();
const RSQ_DATA: [u16; 512] = make_rsq_table();

const TEST_VALUES_32: [u32; 32] = [
    0, 1, 2, 0x2000, 0x7FFF, 0x8000, 0x8001, 0xFFFF, 0x7FFF_FFFE, 0x7FFF_FFFF,
    0x8000_0000, 0x8000_0001, 0x8000_0002, 0x8040_0000, 0xC000_0000, 0xC000_0002,
    0xDEAD_F00D, 0xFFFE_FFFF, 0xFFFF_0000, 0xFFFF_0001, 0xFFFF_0012, 0xFFFF_7FFF,
    0xFFFF_8000, 0xFFFF_8001, 0xFFFF_801E, 0xFFFF_801F, 0xFFFF_8020, 0xFFFF_8021, 0xFFFF_8040,
    0xFFFF_FFFD, 0xFFFF_FFFE, 0xFFFF_FFFF
];

pub const fn rcp(value: u32) -> u32 {
    if value == 0 {
        return 0x7FFF_FFFF;
    }
    if value == 0xFFFF_8000 {
        return 0xFFFF_0000;
    }
    // After 0xFFFF_8000, everything is shifted by one. Why? No idea
    let adjusted_value = if value > 0xFFFF_8000 { value - 1 } else { value };
    let is_negative = (adjusted_value as i32).is_negative();
    let positive_value = if is_negative { !adjusted_value } else { adjusted_value };
    let shift = positive_value.leading_zeros() + 1;
    let index = ((positive_value << shift) >> 23) as usize;
    let positive_result = (0x4000_0000 | ((RCP_DATA[index] as u32) << 14)) >> (32 - shift);
    if is_negative { !positive_result } else { positive_result }
}

pub fn rsq(value: u32) -> u32 {
    if value == 0 {
        return 0x7FFF_FFFF;
    }
    if value == 0xFFFF_8000 {
        return 0xFFFF_0000;
    }
    // After 0xFFFF_8000, everything is offset by one. Why? No idea
    let adjusted_value = if value > 0xFFFF_8000 { value - 1 } else { value };
    let is_negative = (adjusted_value as i32).is_negative();
    let positive_value = if is_negative { !adjusted_value } else { adjusted_value };
    let shift = positive_value.leading_zeros() + 1;
    // For uneven shifts, take the second half of the table
    let index = (((positive_value << shift) >> 24) | ((shift & 1) << 8)) as usize;
    let positive_result = (0x4000_0000 | ((RSQ_DATA[index] as u32) << 14)) >> ((32 - shift) >> 1);
    if is_negative { !positive_result } else { positive_result }
}

fn run_test<FEmitter: Fn(&mut RSPAssembler, VR, VR, VR, Element), FEmulate: Fn(u16) -> u16>(apply_element_to_vt_for_result: bool, check_accumulators: bool, vt_vector: Vector, vd: VR, vs: VR, vt: VR, e: Element, emit: FEmitter, emulate: FEmulate) -> Result<(), String> {
    const INPUT_ACC_TOP: Vector = Vector::new_with_broadcast_16(0x0123);
    const INPUT_ACC_MID: Vector = Vector::new_with_broadcast_16(0x4567);
    const INPUT_ACC_LOW: Vector = Vector::new_with_broadcast_16(0x89AB);
    if check_accumulators {
        // Preexisting accumulator data
        SPMEM::write_vector_into_dmem(0x00, &INPUT_ACC_TOP);
        SPMEM::write_vector_into_dmem(0x10, &INPUT_ACC_MID);
        SPMEM::write_vector_into_dmem(0x20, &INPUT_ACC_LOW);
    }

    // Data that is in source and target vectors
    let vd_pre_vector = Vector::from_u16([0x0000, 0x1001, 0x2002, 0x3003, 0x4004, 0x5005, 0x6006, 0x7007]);
    SPMEM::write_vector_into_dmem(0x30, &vd_pre_vector);
    SPMEM::write_vector_into_dmem(0x40, &vt_vector);
    SPMEM::write_vector_into_dmem(0x50, &Vector::from_u16([0xDECA, 0xF15B, 0xADC0, 0xFFEE, 0xDECA, 0xF15B, 0xADC0, 0xFFEE]));

    let mut assembler = RSPAssembler::new(0);

    if check_accumulators {
        assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
        assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);
        assembler.write_lqv(VR::V2, E::_0, 0x020, GPR::R0);

        assemble_set_accumulator_to(&mut assembler, VR::V0, VR::V1, VR::V2, VR::V3, VR::V4, VR::V5, GPR::AT);
    }

    assembler.write_lqv(vs, E::_0, 0x050, GPR::R0);
    assembler.write_lqv(vd, E::_0, 0x030, GPR::R0);
    assembler.write_lqv(vt, E::_0, 0x040, GPR::R0);

    emit(&mut assembler, vd, vt, vs, e);

    assembler.write_sqv(vd, E::_0, 0x100, GPR::R0);

    if check_accumulators {
        assembler.write_vsar(VR::V4, VSARAccumulator::High);
        assembler.write_vsar(VR::V5, VSARAccumulator::Mid);
        assembler.write_vsar(VR::V6, VSARAccumulator::Low);

        assembler.write_sqv(VR::V4, E::_0, 0x110, GPR::R0);
        assembler.write_sqv(VR::V5, E::_0, 0x120, GPR::R0);
        assembler.write_sqv(VR::V6, E::_0, 0x130, GPR::R0);
    }

    assembler.write_break();

    RSP::run_and_wait(0);

    let result = SPMEM::read_vector_from_dmem(0x100);
    let vt_with_elements = vt_vector.copy_with_element_specifier_applied(e);
    if check_accumulators {
        let acc_top = SPMEM::read_vector_from_dmem(0x110);
        let acc_mid = SPMEM::read_vector_from_dmem(0x120);
        let acc_low = SPMEM::read_vector_from_dmem(0x130);
        soft_assert_eq_vector(acc_top, INPUT_ACC_TOP, || format!("Acc[32..48] for {:?}, {:?}, {:?}, {:?} is expected to be unchanged", vd, vs, vt, e))?;
        soft_assert_eq_vector(acc_mid, INPUT_ACC_MID, || format!("Acc[16..32] for {:?}, {:?}, {:?}, {:?} is expected to be unchanged", vd, vs, vt, e))?;
        soft_assert_eq_vector(acc_low, vt_with_elements, || format!("Acc[0..16] for {:?}, {:?}, {:?}, {:?} is expected to be equal to input vt", vd, vs, vt, e))?;
    }

    let mut expected = Vector::new();
    for i in 0..8 {
        expected.set16(i, if i == vs.index() & 7 {
            let source =
                if apply_element_to_vt_for_result {
                    vt_with_elements.get16(i)
                } else {
                    vt_vector.get16((e as usize) & 7)
                };
            emulate(source)
        } else {
            if vd == vt {
                vt_vector.get16(i)
            } else {
                vd_pre_vector.get16(i)
            }
        });
    }
    soft_assert_eq_vector(result, expected, || format!("Result (vd) for {:?}, {:?}, {:?}, {:?}", vd, vs, vt, e))?;

    Ok(())
}

/// Like run_test above, but this one checks two outputs. input values is in first two 16 bit values, output value is next 2
fn run_test_result_only_16<FEmitter: Fn(&mut RSPAssembler, VR, VR, Element)>(input_value: u16, expected_value: u16, emit: FEmitter) -> Result<(), String> {
    SPMEM::write(0x40, (input_value as u32) << 16);

    let mut assembler = RSPAssembler::new(0);
    assembler.write_lqv(VR::V0, E::_0, 0x040, GPR::R0);
    emit(&mut assembler, VR::V0, VR::V0, Element::_0);
    assembler.write_sqv(VR::V0, E::_0, 0x100, GPR::R0);
    assembler.write_break();

    RSP::run_and_wait(0);

    let result = SPMEM::read(0x100) as u16;
    soft_assert_eq2(result, expected_value, || format!("Result for value {}", input_value))?;

    Ok(())
}

/// Like run_test above, but this one checks two outputs. input values is in first two 16 bit values, output value is next 2
fn run_test_result_only_32<FEmitter: Fn(&mut RSPAssembler, VR)>(input_value: u32, expected_value: u32, emit: FEmitter) -> Result<(), String> {
    SPMEM::write(0x40, input_value);

    let mut assembler = RSPAssembler::new(0);
    assembler.write_lqv(VR::V0, E::_0, 0x040, GPR::R0);
    emit(&mut assembler, VR::V0);
    assembler.write_sqv(VR::V0, E::_0, 0x100, GPR::R0);
    assembler.write_break();

    RSP::run_and_wait(0);

    let result = SPMEM::read(0x104);
    soft_assert_eq2(result, expected_value, || format!("Result for value 0x{:x}", input_value))?;

    Ok(())
}

pub struct VMOV {}

impl Test for VMOV {
    fn name(&self) -> &str { "RSP VMOV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let vt_vector = Vector::from_u16([0x0880, 0x0990, 0x0AA0, 0x0BB0, 0x0CC0, 0x0DD0, 0x0EE0, 0x0FF0]);
        for vt in [VR::V0, VR::V1] {
            for vd in [VR::V0, VR::V1] {
                for vs in VR::V0..=VR::V31 {
                    for e in Element::range() {
                        run_test(
                            true,
                            true,
                            vt_vector,
                            vd,
                            vs,
                            vt,
                            e,
                            &|assembler: &mut RSPAssembler, vd, vt, vs, e| { assembler.write_vmov(vd, vt, vs, e); },
                            |value| value,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct VRCPRegisterCombinations {}

impl Test for VRCPRegisterCombinations {
    fn name(&self) -> &str { "RSP VRCP (register and element combinations)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for vt in [VR::V0, VR::V1] {
            for vd in [VR::V0, VR::V1] {
                for vs in [VR::V0, VR::V1, VR::V2, VR::V3, VR::V15, VR::V16, VR::V17, VR::V30, VR::V31] {
                    for e in Element::range() {
                        for i in [0, 1835, 31456, 32767, 65535] {
                            run_test(
                                false,
                                true,
                                Vector::from_u16([i, i + 1, i + 2, i + 3, i + 4, i + 5, i + 6, i + 7]),
                                vd,
                                vs,
                                vt,
                                e,
                                &|assembler: &mut RSPAssembler, vd, vt, vs, e| { assembler.write_vrcp(vd, vt, vs, e); },
                                |value| { rcp(value as i16 as u32) as u16 },
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct VRCPHRegisterCombinations {}

impl Test for VRCPHRegisterCombinations {
    fn name(&self) -> &str { "RSP VRCPH (register and element combinations)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for vt in [VR::V0, VR::V1] {
            for vd in [VR::V0, VR::V1] {
                for vs in [VR::V0, VR::V1, VR::V2, VR::V3, VR::V15, VR::V16, VR::V17, VR::V30, VR::V31] {
                    for e in Element::range() {
                        for i in [0, 1835, 31456, 32767, 65535] {
                            run_test(
                                false,
                                true,
                                Vector::from_u16([i, i + 1, i + 2, i + 3, i + 4, i + 5, i + 6, i + 7]),
                                vd,
                                vs,
                                vt,
                                e,
                                &|assembler: &mut RSPAssembler, vd, vt, vs, e| {
                                    assembler.write_vrcp(VR::V31, vt, vs, e);
                                    assembler.write_vrcph(vd, vt, vs, e);
                                },
                                |value| { (rcp(value as i16 as u32) >> 16) as u16 },
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct VRCPLRegisterCombinations {}

impl Test for VRCPLRegisterCombinations {
    fn name(&self) -> &str { "RSP VRCPL (register and element combinations)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for vt in [VR::V0, VR::V1] {
            for vd in [VR::V0, VR::V1] {
                for vs in [VR::V0, VR::V1, VR::V2, VR::V3, VR::V15, VR::V16, VR::V17, VR::V30, VR::V31] {
                    for e in Element::range() {
                        for i in [0, 1835, 31456, 32767, 65535] {
                            run_test(
                                false,
                                true,
                                Vector::from_u16([i, i + 1, i + 2, i + 3, i + 4, i + 5, i + 6, i + 7]),
                                vd,
                                vs,
                                vt,
                                e,
                                &|assembler: &mut RSPAssembler, vd, vt, vs, e| {
                                    assembler.write_vrcp(VR::V31, vt, vs, e);
                                    assembler.write_vrcpl(vd, vt, vs, e);
                                },
                                |value| { rcp(value as i16 as u32) as u16 },
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct VRSQRegisterCombinations {}

impl Test for VRSQRegisterCombinations {
    fn name(&self) -> &str { "RSP VRSQ (register and element combinations)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for vt in [VR::V0, VR::V1] {
            for vd in [VR::V0, VR::V1] {
                for vs in [VR::V0, VR::V1, VR::V2, VR::V3, VR::V15, VR::V16, VR::V17, VR::V30, VR::V31] {
                    for e in Element::range() {
                        for i in [0, 1835, 31456, 32767, 65535] {
                            run_test(
                                false,
                                true,
                                Vector::from_u16([i, i + 1, i + 2, i + 3, i + 4, i + 5, i + 6, i + 7]),
                                vd,
                                vs,
                                vt,
                                e,
                                &|assembler: &mut RSPAssembler, vd, vt, vs, e| { assembler.write_vrsq(vd, vt, vs, e); },
                                |value| { rsq(value as i16 as u32) as u16 },
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct VRSQLRegisterCombinations {}

impl Test for VRSQLRegisterCombinations {
    fn name(&self) -> &str { "RSP VRSQL (register and element combinations)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for vt in [VR::V0, VR::V1] {
            for vd in [VR::V0, VR::V1] {
                for vs in [VR::V0, VR::V1, VR::V2, VR::V3, VR::V15, VR::V16, VR::V17, VR::V30, VR::V31] {
                    for e in Element::range() {
                        for i in [0, 1835, 31456, 32767, 65535] {
                            run_test(
                                false,
                                true,
                                Vector::from_u16([i, i + 1, i + 2, i + 3, i + 4, i + 5, i + 6, i + 7]),
                                vd,
                                vs,
                                vt,
                                e,
                                &|assembler: &mut RSPAssembler, vd, vt, vs, e| {
                                    assembler.write_vrsq(VR::V31, vt, vs, e);
                                    assembler.write_vrsql(vd, vt, vs, e);
                                },
                                |value| { rsq(value as i16 as u32) as u16 },
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct VRSQHRegisterCombinations {}

impl Test for VRSQHRegisterCombinations {
    fn name(&self) -> &str { "RSP VRSQH (register and element combinations)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for vt in [VR::V0, VR::V1] {
            for vd in [VR::V0, VR::V1] {
                for vs in [VR::V0, VR::V1, VR::V2, VR::V3, VR::V15, VR::V16, VR::V17, VR::V30, VR::V31] {
                    for e in Element::range() {
                        for i in [0, 1835, 31456, 32767, 65535] {
                            run_test(
                                false,
                                true,
                                Vector::from_u16([i, i + 1, i + 2, i + 3, i + 4, i + 5, i + 6, i + 7]),
                                vd,
                                vs,
                                vt,
                                e,
                                &|assembler: &mut RSPAssembler, vd, vt, vs, e| {
                                    assembler.write_vrsq(VR::V31, vt, vs, e);
                                    assembler.write_vrsqh(vd, vt, vs, e);
                                },
                                |value| { (rsq(value as i16 as u32) >> 16) as u16 },
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct VRCPValues {}

impl Test for VRCPValues {
    fn name(&self) -> &str { "RSP VRCP (all 16 bit values)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // We'll skip testing Element-specifiers, accumulators etc as we trust that VRCPRegisterCombinations
        // found everything. Here we focus on testing every possible value
        for i in 0..=65535 {
            run_test_result_only_16(
                i,
                rcp(i as i16 as u32) as u16,
                &|assembler: &mut RSPAssembler, vd, vt, e| { assembler.write_vrcp(vd, vt, VR::V1, e); },
            )?;
        }

        Ok(())
    }
}

pub struct VRSQValues {}

impl Test for VRSQValues {
    fn name(&self) -> &str { "RSP VRSQ (all 16 bit values)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // We'll skip testing Element-specifiers, accumulators etc as we trust that VRSQRegisterCombinations
        // found everything. Here we focus on testing every possible value
        for i in 0..=65535 {
            run_test_result_only_16(
                i,
                rsq(i as i16 as u32) as u16,
                &|assembler: &mut RSPAssembler, vd, vt, e| { assembler.write_vrsq(vd, vt, VR::V1, e); },
            )?;
        }

        Ok(())
    }
}

pub struct VRCP32Bit {}

impl Test for VRCP32Bit {
    fn name(&self) -> &str { "RSP VRCPH/VRCPL (32 bit)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for i in TEST_VALUES_32 {
            run_test_result_only_32(
                i,
                rcp(i),
                &|assembler: &mut RSPAssembler, reg| {
                    assembler.write_vrcph(reg, reg, VR::V2, Element::_0);
                    assembler.write_vrcpl(reg, reg, VR::V3, Element::_1);
                    assembler.write_vrcph(reg, reg, VR::V2, Element::_0);
                },
            )?;
        }

        Ok(())
    }
}

pub struct VRSQ32Bit {}

impl Test for VRSQ32Bit {
    fn name(&self) -> &str { "RSP VRSQH/VRSQL (32 bit)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for i in TEST_VALUES_32 {
            run_test_result_only_32(
                i,
                rsq(i),
                &|assembler: &mut RSPAssembler, reg| {
                    assembler.write_vrsqh(reg, reg, VR::V2, Element::_0);
                    assembler.write_vrsql(reg, reg, VR::V3, Element::_1);
                    assembler.write_vrsqh(reg, reg, VR::V2, Element::_0);
                },
            )?;
        }

        Ok(())
    }
}

pub struct RCPTable {}

impl Test for RCPTable {
    fn name(&self) -> &str { "RSP RCP (verify table)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let table = make_rcp_table();
        soft_assert_eq2(table.len(), 512, || format!("RCP-Table is setup incorrectly"))?;
        for (i, &expected) in table.iter().enumerate() {
            let value = 0x1000 + (i << 3) as u16;
            SPMEM::write_vector_into_dmem(0x0, &Vector::new_with_broadcast_16(value));

            let mut assembler = RSPAssembler::new(0);

            assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);

            assembler.write_vrcp(VR::V1, VR::V0, VR::V1, Element::_0);
            assembler.write_vrcph(VR::V1, VR::V0, VR::V0, Element::_0);

            assembler.write_sqv(VR::V1, E::_0, 0x100, GPR::R0);

            assembler.write_break();

            RSP::run_and_wait(0);

            let result = SPMEM::read_vector_from_dmem(0x100);

            let actual = (result.get32(0) >> 2) & !0x10000;
            soft_assert_eq2(actual, expected as u32, || format!("RCP_DATA[{}]", i))?;
        }
        Ok(())
    }
}

pub struct RSQTable {}

impl Test for RSQTable {
    fn name(&self) -> &str { "RSP RSQ (verify table)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for (i, &expected) in RSQ_DATA.iter().enumerate() {
            let value = (if i > 255 { 0x2000 + ((i - 256) << 5) } else { 0x1000 + (i << 4) }) as u16;
            SPMEM::write_vector_into_dmem(0x0, &Vector::new_with_broadcast_16(value));

            let mut assembler = RSPAssembler::new(0);

            assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);

            assembler.write_vrsq(VR::V1, VR::V0, VR::V1, Element::_0);
            assembler.write_vrsqh(VR::V1, VR::V0, VR::V0, Element::_0);

            assembler.write_sqv(VR::V1, E::_0, 0x100, GPR::R0);

            assembler.write_break();

            RSP::run_and_wait(0);

            let result = SPMEM::read_vector_from_dmem(0x100);

            let actual = (result.get32(0) >> 8) & !0x10000;
            soft_assert_eq2(actual, expected as u32, || format!("RSQ_DATA[{}]", i))?;
        }
        Ok(())
    }
}

fn test_high_uses_output<FEmit: Fn(&mut RSPAssembler)>(expected_result: u16, emit: FEmit) -> Result<(), String> {
    SPMEM::write_vector_into_dmem(0, &Vector::new_with_broadcast_16(0xE834));

    let mut assembler = RSPAssembler::new(0);

    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);

    emit(&mut assembler);

    // We can read the high part of the reciprocal with either instruction, as many times as we want to
    assembler.write_vrcph(VR::V2, VR::V0, VR::V0, Element::_0);
    assembler.write_vrcph(VR::V3, VR::V0, VR::V0, Element::_0);
    assembler.write_vrsqh(VR::V4, VR::V0, VR::V0, Element::_0);
    assembler.write_vrsqh(VR::V5, VR::V0, VR::V0, Element::_0);

    assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
    assembler.write_sqv(VR::V2, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V3, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V4, E::_0, 0x130, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    soft_assert_eq2((SPMEM::read(0x100) >> 16) as u16, expected_result, || format!("VRCPH should write upper value (first time)"))?;
    soft_assert_eq2((SPMEM::read(0x110) >> 16) as u16, expected_result, || format!("VRCPH should write upper value (second time)"))?;
    soft_assert_eq2((SPMEM::read(0x120) >> 16) as u16, expected_result, || format!("VRSQH should write upper value (first time)"))?;
    soft_assert_eq2((SPMEM::read(0x130) >> 16) as u16, expected_result, || format!("VRSQH should write upper value (second time)"))?;

    Ok(())
}

pub struct HighUsesOutputVRCPTest {}

impl Test for HighUsesOutputVRCPTest {
    fn name(&self) -> &str { "RSP VRCPH/VRSQH (read high VRCP) )" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_high_uses_output(0xfffa, |assembler| assembler.write_vrcp(VR::V1, VR::V0, VR::V1, Element::_0))
    }
}

pub struct HighUsesOutputVRCPLTest {}

impl Test for HighUsesOutputVRCPLTest {
    fn name(&self) -> &str { "RSP VRCPH/VRSQH (read high VRCPL) )" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_high_uses_output(0xfffe, |assembler| assembler.write_vrcpl(VR::V1, VR::V0, VR::V1, Element::_0))
    }
}

pub struct HighUsesOutputVRSQTest {}

impl Test for HighUsesOutputVRSQTest {
    fn name(&self) -> &str { "RSP VRCPH/VRSQH (read high VRSQ) )" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_high_uses_output(0xfe5b, |assembler| assembler.write_vrsq(VR::V1, VR::V0, VR::V1, Element::_0))
    }
}

pub struct HighUsesOutputVRSQLTest {}

impl Test for HighUsesOutputVRSQLTest {
    fn name(&self) -> &str { "RSP VRCPH/VRSQH (read high VRSQL) )" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_high_uses_output(0xfffe, |assembler| assembler.write_vrsql(VR::V1, VR::V0, VR::V1, Element::_0))
    }
}

pub struct VRCPHSetsInputForVRCPL {}

impl Test for VRCPHSetsInputForVRCPL {
    fn name(&self) -> &str { "RSP VRCPH sets input for VRCPL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // There's a hidden input register. It is:
        // - set by VRCPH/VRSQH
        // - read by VRCPL/VRSQL
        // - cleared by VRCP/VRSQ and VRCPL/VRSQL
        // Go through various combinations of the instructions above to ensure they actually operate on
        // the same register
        for i in 0..32 {
            SPMEM::write_vector_into_dmem(0, &Vector::new_with_broadcast_16(0xE834));

            let mut assembler = RSPAssembler::new(0);

            assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);

            // Set the hidden input register (twice to ensure this is not a stack)
            if (i & 1) != 0 {
                assembler.write_vrcph(VR::V31, VR::V0, VR::V0, Element::_0);
                assembler.write_vrcph(VR::V30, VR::V0, VR::V0, Element::_0);
            } else {
                assembler.write_vrsqh(VR::V31, VR::V0, VR::V0, Element::_0);
                assembler.write_vrsqh(VR::V30, VR::V0, VR::V0, Element::_0);
            }

            // Read. The first and second call should be different as the first one clears the register for the second
            if (i & 2) != 0 {
                assembler.write_vrcpl(VR::V2, VR::V0, VR::V0, Element::_0);
                assembler.write_vrcpl(VR::V3, VR::V0, VR::V0, Element::_0);
            } else {
                assembler.write_vrsql(VR::V2, VR::V0, VR::V0, Element::_0);
                assembler.write_vrsql(VR::V3, VR::V0, VR::V0, Element::_0);
            }

            // Set again
            if (i & 4) != 0 {
                assembler.write_vrcph(VR::V31, VR::V0, VR::V0, Element::_0);
            } else {
                assembler.write_vrsqh(VR::V31, VR::V0, VR::V0, Element::_0);
            }

            // Have VRCP clear it then read again
            if (i & 8) != 0 {
                assembler.write_vrcp(VR::V31, VR::V0, VR::V0, Element::_0);
            } else {
                assembler.write_vrsq(VR::V31, VR::V0, VR::V0, Element::_0);
            }
            if (i & 16) != 0 {
                assembler.write_vrcpl(VR::V4, VR::V0, VR::V0, Element::_0);
            } else {
                assembler.write_vrsql(VR::V4, VR::V0, VR::V0, Element::_0);
            }


            assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
            assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
            assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);

            assembler.write_break();

            RSP::run_and_wait(0);

            if (i & 2) != 0 {
                soft_assert_eq2((SPMEM::read(0x100) >> 16) as u16, 0xFFFA, || format!("{} should write the hidden register for VRCPL", if (i & 1) != 0 { "VRCPH" } else { "VRSQH" }))?;
                soft_assert_eq2((SPMEM::read(0x110) >> 16) as u16, 0x9E1B, || format!("VRCPL clears the hidden register"))?;
            } else {
                soft_assert_eq2((SPMEM::read(0x100) >> 16) as u16, 0x5BC2, || format!("{} should write the hidden register for VRSQL", if (i & 1) != 0 { "VRCPH" } else { "VRSQH" }))?;
                soft_assert_eq2((SPMEM::read(0x110) >> 16) as u16, 0xC2FF, || format!("VRSQL clears the hidden register"))?;
            }
            if (i & 16) != 0 {
                soft_assert_eq2((SPMEM::read(0x120) >> 16) as u16, 0x9E1B, || format!("{} should clear the hidden register", if (i & 8) != 0 { "VRCP" } else { "VRSQ" }))?;
            } else {
                soft_assert_eq2((SPMEM::read(0x120) >> 16) as u16, 0xC2FF, || format!("{} should clear the hidden register", if (i & 8) != 0 { "VRCP" } else { "VRSQ" }))?;
            }
        }

        Ok(())
    }
}

pub struct VRCPHSetsInputForVRSQL {}

impl Test for VRCPHSetsInputForVRSQL {
    fn name(&self) -> &str { "RSP VRCPH sets input for VRSQL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // There's a hidden input register. It is:
        // - set by VRCPH/VRSQH
        // - read by VRCPL/VRSQL
        // - cleared by VRCP/VRSQ and VRCPL/VRSQL

        SPMEM::write_vector_into_dmem(0, &Vector::new_with_broadcast_16(0xE834));

        let mut assembler = RSPAssembler::new(0);

        assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);

        // Set the hidden input register (twice to ensure this is not a stack)
        assembler.write_vrcph(VR::V31, VR::V0, VR::V0, Element::_0);
        assembler.write_vrcph(VR::V30, VR::V0, VR::V0, Element::_0);

        // Read by VRCPL. The first and second call should be different as the first one clears the register for the second
        assembler.write_vrcpl(VR::V2, VR::V0, VR::V0, Element::_0);
        assembler.write_vrcpl(VR::V3, VR::V0, VR::V0, Element::_0);

        // Set again
        assembler.write_vrcph(VR::V31, VR::V0, VR::V0, Element::_0);

        // Have VRCP clear it then read again
        assembler.write_vrcp(VR::V31, VR::V0, VR::V0, Element::_0);
        assembler.write_vrcpl(VR::V4, VR::V0, VR::V0, Element::_0);

        assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
        assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);
        assembler.write_sqv(VR::V4, E::_0, 0x120, GPR::R0);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq2((SPMEM::read(0x100) >> 16) as u16, 0xFFFA, || format!("VRCPH should write the hidden register for VRCPL"))?;
        soft_assert_eq2((SPMEM::read(0x110) >> 16) as u16, 0x9E1B, || format!("VRCPL clears the hidden register"))?;
        soft_assert_eq2((SPMEM::read(0x120) >> 16) as u16, 0x9E1B, || format!("VRCP should clear the hidden register"))?;

        Ok(())
    }
}

pub struct VRCPLHiddenRegisterFlagExists {}

impl Test for VRCPLHiddenRegisterFlagExists {
    fn name(&self) -> &str { "RSP hidden flag register" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // There are two (reasonable) ways how whether the temp register is being filled could be expressed:
        // a) a special value (e.g. 0x0000) means "not set"
        // b) another hidden register flag determines whether the value is set
        // The correct answer seems to be b. This test proves that it is not a by trying every single value

        let mut assembler = RSPAssembler::new(0);

        assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);

        // Set the hidden input register
        assembler.write_vrcph(VR::V31, VR::V0, VR::V0, Element::_0);

        // Use it through VRCPL. If it is used, the next two calculations should give different results
        assembler.write_vrcpl(VR::V2, VR::V0, VR::V0, Element::_1);
        assembler.write_vrcpl(VR::V3, VR::V0, VR::V0, Element::_1);

        assembler.write_sqv(VR::V2, E::_0, 0x100, GPR::R0);
        assembler.write_sqv(VR::V3, E::_0, 0x110, GPR::R0);

        assembler.write_break();

        for upper in 0..=0xFFFFu16 {
            // Pick lower bits so that we can tell the difference
            let lower = if upper == 0xFFFF { 0x1234 } else { 0xE834 };

            SPMEM::write(0, ((upper as u32) << 16) | lower as u32);

            RSP::run_and_wait(0);

            if SPMEM::read(0x100) == SPMEM::read(0x110) {
                return Err(format!("VRCPL should put 0x{:x} in the upper 16 bits of the first call, but not for the second call", upper));
            }
        }

        Ok(())
    }
}



/// This is not a real test. It dumps the tables used by VRCP/VRSQ to screen
/// Run via "make rcq_rsq_dump". Adjust the rsq and page variables to get the whole table
pub struct GenerateDump {}

impl Test for GenerateDump {
    fn name(&self) -> &str { "RSP RCP/RSQ (generate dump)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let rsq = false;  // true for RSQ, false for RCP
        let page = 0;
        let range = if rsq {
            match page {
                0 => (0x1000..0x1800).step_by(16),
                1 => (0x1800..0x2000).step_by(16),
                2 => (0x2000..0x3000).step_by(32),
                3 => (0x3000..0x4000).step_by(32),
                _ => panic!(),
            }
        } else {
            match page {
                0 => (0x1000..0x1400).step_by(8),
                1 => (0x1400..0x1800).step_by(8),
                2 => (0x1800..0x1C00).step_by(8),
                3 => (0x1C00..0x2000).step_by(8),
                _ => panic!(),
            }
        };
        let mut counter = 0;
        for value in range {
            SPMEM::write_vector_into_dmem(0x0, &Vector::new_with_broadcast_16(value));

            let mut assembler = RSPAssembler::new(0);

            assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);

            if rsq {
                assembler.write_vrsq(VR::V1, VR::V0, VR::V1, Element::_0);
                assembler.write_vrsqh(VR::V1, VR::V0, VR::V0, Element::_0);
            } else {
                assembler.write_vrcp(VR::V1, VR::V0, VR::V1, Element::_0);
                assembler.write_vrcph(VR::V1, VR::V0, VR::V0, Element::_0);
            }

            assembler.write_sqv(VR::V1, E::_0, 0x100, GPR::R0);

            assembler.write_break();

            RSP::run_and_wait(0);

            let result = SPMEM::read_vector_from_dmem(0x100);

            print!("{:x} ", (result.get32(0) >> (if rsq { 8 } else { 2 })) & !0x10000);
            counter += 1;
            if counter % 8 == 0 {
                println!();
            }
        }
        println!();

        Ok(())
    }
}
