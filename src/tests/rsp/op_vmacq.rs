use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::math::vector::Vector;
use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, Element, GPR, RSPAssembler, VR, VSARAccumulator};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq2;

/// Sets the accumulator to the given values
fn assemble_set_accumulator_to(assembler: &mut RSPAssembler, top: VR, mid: VR, low: VR, scratch: VR, scratch2: VR, scratch3: VR, scratch_gpr: GPR) {
    // Put some constants that we need into scratch2:
    // 0000 40000 0001
    assembler.write_mtc2(scratch2, GPR::R0, E::_0);
    assembler.write_li(scratch_gpr, 0x4000);
    assembler.write_mtc2(scratch2, scratch_gpr, E::_2);
    assembler.write_li(scratch_gpr, 1);
    assembler.write_mtc2(scratch2, scratch_gpr, E::_4);

    // Set top part through 4 VMADH, then middle part through 1 VMADH.
    // However, when the middle part is negative, it will reduce 1 from top. To compensate,
    // add one to high whenever mid is negative

    // Set VCO.low if number is negative
    assembler.write_vaddc(scratch, mid, mid, Element::All);
    assembler.write_vxor(scratch, scratch, scratch, Element::All);
    // Set scratch to 1 if VCO.low; 0 otherwise
    assembler.write_vadd(scratch, scratch2, scratch, Element::_0);
    // Add 0/1 to top, without saturation
    assembler.write_vaddc(scratch, scratch, top, Element::All);

    // Accumulator top: Multiply by 16384 and add up four times.
    assembler.write_vmudh(scratch3, scratch2, scratch, Element::_1);
    assembler.write_vmadh(scratch3, scratch2, scratch, Element::_1);
    assembler.write_vmadh(scratch3, scratch2, scratch, Element::_1);
    assembler.write_vmadh(scratch3, scratch2, scratch, Element::_1);

    // Accumulator mid: Multiply by 1 and add to accumulator
    assembler.write_vmadh(scratch3, scratch2, mid, Element::_2);

    // For low, we can use VADDC with 0 which just sets low

    assembler.write_vaddc(scratch3, scratch2, low, Element::_0);
}

fn simulate(acc_top: u16, acc_mid: u16, acc_low: u16) -> (u16, u16, u16, u16) {
    let acc_input = (((acc_top as i64) << 48) | ((acc_mid as i64) << 32) | ((acc_low as i64) << 16)) >> 16;

    // Add/Remove value, depending on bits in the accumulator
    let should_change = (acc_input & 0x20_0000) == 0;
    let acc_output = if should_change {
        let upper_bits = (acc_input >> 22) as i32;
        if upper_bits < 0 {
            acc_input + 0x20_0000
        } else if upper_bits > 0 {
            acc_input - 0x20_0000
        } else {
            acc_input
        }
    } else {
        acc_input
    };

    // We removed 0x20_0000 only if the original number was larger than 0x40_0000. Same for negative.
    // Therefore, the sign of the result can't change and we don't need to sign-extend from 48 to 64

    let clamped_and_shifted = if acc_output < 0 {
        if ((!acc_output) >> 32) != 0 {
            0x8000
        } else {
            (acc_output >> 17) as u16
        }
    } else {
        if (acc_output >> 32) != 0 {
            0x7FFF
        } else {
            (acc_output >> 17) as u16
        }
    };
    let result = clamped_and_shifted & 0xFFF0;
    (result, (acc_output >> 32) as u16, (acc_output >> 16) as u16, acc_output as u16)
}

fn run_test(input_acc_top: Vector, input_acc_mid: Vector, input_acc_low: Vector, vs: VR, vt: VR, e: Element) -> Result<(), String> {
    // Prepare input data
    SPMEM::write_vector_into_dmem(0x00, &input_acc_top);
    SPMEM::write_vector_into_dmem(0x10, &input_acc_mid);
    SPMEM::write_vector_into_dmem(0x20, &input_acc_low);

    let mut assembler = RSPAssembler::new(0);

    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);
    assembler.write_lqv(VR::V2, E::_0, 0x020, GPR::R0);

    assemble_set_accumulator_to(&mut assembler, VR::V0, VR::V1, VR::V2, VR::V3, VR::V4, VR::V5, GPR::AT);

    // TODO: Test various vs, vt, e
    assembler.write_vmacq(VR::V3, vt, vs, e);

    assembler.write_vsar(VR::V4, VSARAccumulator::High);
    assembler.write_vsar(VR::V5, VSARAccumulator::Mid);
    assembler.write_vsar(VR::V6, VSARAccumulator::Low);

    assembler.write_sqv(VR::V3, E::_0, 0x100, GPR::R0);
    assembler.write_sqv(VR::V4, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V5, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V6, E::_0, 0x130, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    let result = SPMEM::read_vector_from_dmem(0x100);
    let acc_top = SPMEM::read_vector_from_dmem(0x110);
    let acc_mid = SPMEM::read_vector_from_dmem(0x120);
    let acc_low = SPMEM::read_vector_from_dmem(0x130);

    for i in 0..8 {
        let (expected_result, expected_acc_top, expected_acc_mid, expected_acc_low) = simulate(input_acc_top.get16(i), input_acc_mid.get16(i), input_acc_low.get16(i));

        soft_assert_eq2(acc_top.get16(i), expected_acc_top, || format!("Acc[32..48] for VMACQ (element {}) with previous accumulator {:04x} {:04x} {:04x}", i, input_acc_top.get16(i), input_acc_mid.get16(i), input_acc_low.get16(i)))?;
        soft_assert_eq2(acc_mid.get16(i), expected_acc_mid, || format!("Acc[16..32] for VMACQ (element {}) with previous accumulator {:04x} {:04x} {:04x}", i, input_acc_top.get16(i), input_acc_mid.get16(i), input_acc_low.get16(i)))?;
        soft_assert_eq2(acc_low.get16(i), expected_acc_low, || format!("Acc[0..16] for VMACQ (element {}) with previous accumulator {:04x} {:04x} {:04x}", i, input_acc_top.get16(i), input_acc_mid.get16(i), input_acc_low.get16(i)))?;
        soft_assert_eq2(result.get16(i), expected_result, || format!("Result(vd) for 3VMACQ (element {}) with previous accumulator {:04x} {:04x} {:04x}", i, input_acc_top.get16(i), input_acc_mid.get16(i), input_acc_low.get16(i)))?;
    }

    Ok(())
}

pub struct VMACQ {}

impl Test for VMACQ {
    fn name(&self) -> &str { "RSP VMACQ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let test_mid_top = [
            0x0000_0000u32,
            0x0000_001F,
            0x0000_0020,
            0x0000_0040,
            0x0000_0060,
            0x0000_FFDF,
            0x0000_FFE0,
            0x0000_FFE1,
            0x0001_0000,
            0x0001_001F,
            0x0001_0020,
            0x0001_0021,
            0x0001_0021,
            0x7001_0020,
            0x7001_0040,
            0x7001_0060,
            0x7000_0000,
            0x7000_0020,
            0x7FFF_FFA0,
            0x7FFF_FFB0,
            0x7FFF_FFC0,
            0x7FFF_FFD0,
            0x7FFF_FFE0,
            0x7FFF_FFF0,
            0x8000_0000,
            0x8000_0020,
            0x8000_0040,
            0x8000_0060,
            0xC000_0020,
            0xC000_0040,
            0xC000_0060,
            0xC001_0000,
            0xC001_0019,
            0xC001_0020,
            0xC001_0021,
            0xFFFF_FFA0,
            0xFFFF_FFB0,
            0xFFFF_FFC0,
            0xFFFF_FFD0,
            0xFFFF_FFE0,
            0xFFFF_FFF0,
            0xFFFF_FFFF,
        ];
        // vt, vs, e are ignored by VMACQ, so put some "random" things in there
        let pseudo_random_bits = 0x75128a93_bb2e5a63u64;
        let mut pseudo_random_index = 0;
        fn get_pseudo_random(bits: &u64, index: &mut i32) -> u16 {
            if *index == 48 {
                *index = 0
            } else {
                *index += 1;
            }
            (*bits >> *index) as u16
        }
        let low = Vector::from_u16([0x00, 0x11, 0x22, 0x44, 0x88, 0x0F, 0xF0, 0xFF]);
        for mid_top in test_mid_top {
            let top = Vector::new_with_broadcast_16((mid_top >> 16) as u16);
            let mid = Vector::new_with_broadcast_16(mid_top as u16);
            run_test(
                top,
                mid,
                low,
                VR::from_index(get_pseudo_random(&pseudo_random_bits, &mut pseudo_random_index) as usize & 0x1F).unwrap(),
                VR::from_index(get_pseudo_random(&pseudo_random_bits, &mut pseudo_random_index) as usize & 0x1F).unwrap(),
                Element::from_index(get_pseudo_random(&pseudo_random_bits, &mut pseudo_random_index) as usize & 0xF).unwrap(),
            )?;
        }

        Ok(())
    }
}
