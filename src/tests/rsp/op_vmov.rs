use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::math::vector::Vector;
use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, Element, GPR, RSPAssembler, VR, VSARAccumulator};
use crate::rsp::rsp_macros::assemble_set_accumulator_to;
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq_vector;

fn run_test(vd: VR, vs: VR, vt: VR, e: Element) -> Result<(), String> {
    // Preexisting accumulator data
    let input_acc_top = Vector::new_with_broadcast_16(0x0123);
    let input_acc_mid = Vector::new_with_broadcast_16(0x4567);
    let input_acc_low = Vector::new_with_broadcast_16(0x89AB);
    SPMEM::write_vector_into_dmem(0x00, &input_acc_top);
    SPMEM::write_vector_into_dmem(0x10, &input_acc_mid);
    SPMEM::write_vector_into_dmem(0x20, &input_acc_low);

    // Data that is in source and target vectors
    let vt_vector = Vector::from_u16([0x0880, 0x0990, 0x0AA0, 0x0BB0, 0x0CC0, 0x0DD0, 0x0EE0, 0x0FF0]);
    let vd_pre_vector = Vector::from_u16([0x0000, 0x1001, 0x2002, 0x3003, 0x4004, 0x5005, 0x6006, 0x7007]);
    SPMEM::write_vector_into_dmem(0x30, &vd_pre_vector);
    SPMEM::write_vector_into_dmem(0x40, &vt_vector);
    SPMEM::write_vector_into_dmem(0x50, &Vector::from_u16([0xDECA, 0xF15B, 0xADC0, 0xFFEE, 0xDECA, 0xF15B, 0xADC0, 0xFFEE]));

    let mut assembler = RSPAssembler::new(0);

    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);
    assembler.write_lqv(VR::V2, E::_0, 0x020, GPR::R0);

    assemble_set_accumulator_to(&mut assembler, VR::V0, VR::V1, VR::V2, VR::V3, VR::V4, VR::V5, GPR::AT);

    assembler.write_lqv(vs,  E::_0, 0x050, GPR::R0);
    assembler.write_lqv(vd,  E::_0, 0x030, GPR::R0);
    assembler.write_lqv(vt,  E::_0, 0x040, GPR::R0);

    assembler.write_vmov(vd, vt, vs, e);

    assembler.write_sqv(vd, E::_0, 0x100, GPR::R0);

    assembler.write_vsar(VR::V4, VSARAccumulator::High);
    assembler.write_vsar(VR::V5, VSARAccumulator::Mid);
    assembler.write_vsar(VR::V6, VSARAccumulator::Low);

    assembler.write_sqv(VR::V4, E::_0, 0x110, GPR::R0);
    assembler.write_sqv(VR::V5, E::_0, 0x120, GPR::R0);
    assembler.write_sqv(VR::V6, E::_0, 0x130, GPR::R0);

    assembler.write_break();

    RSP::run_and_wait(0);

    let result = SPMEM::read_vector_from_dmem(0x100);
    let acc_top = SPMEM::read_vector_from_dmem(0x110);
    let acc_mid = SPMEM::read_vector_from_dmem(0x120);
    let acc_low = SPMEM::read_vector_from_dmem(0x130);

    let vt_with_elements = vt_vector.copy_with_element_specifier_applied(e);

    soft_assert_eq_vector(acc_top, input_acc_top, || format!("Acc[32..48] for VMOV {:?}, {:?}, {:?}, {:?} is expected to be unchanged", vd, vs, vt, e))?;
    soft_assert_eq_vector(acc_mid, input_acc_mid, || format!("Acc[16..32] for VMOV {:?}, {:?}, {:?}, {:?} is expected to be unchanged", vd, vs, vt, e))?;
    soft_assert_eq_vector(acc_low, vt_with_elements, || format!("Acc[0..16] for VMOV {:?}, {:?}, {:?}, {:?} is expected to be equal to input vt", vd, vs, vt, e))?;
    let mut expected = Vector::new();
    for i in 0..8 {
        expected.set16(i, if i == vs.index() & 7 {
            vt_with_elements.get16(i)
        } else {
            if vd == vt {
                vt_vector.get16(i)
            } else {
                vd_pre_vector.get16(i)
            }
        });
    }
    soft_assert_eq_vector(result, expected, || format!("Result (vd) for VMOV {:?}, {:?}, {:?}, {:?}", vd, vs, vt, e))?;

    Ok(())
}

pub struct VMOV {}

impl Test for VMOV {
    fn name(&self) -> &str { "RSP VMOV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for vt in [VR::V0, VR::V1] {
            for vd in [VR::V0, VR::V1] {
                for vs in VR::V0..=VR::V31 {
                    for e in Element::range() {
                        run_test(
                            vd,
                            vs,
                            vt,
                            e,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}
