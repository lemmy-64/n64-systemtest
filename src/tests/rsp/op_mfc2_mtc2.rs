use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::math::vector::Vector;
use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, GPR, RSPAssembler, VR};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq2, soft_assert_eq_vector};

// Lessons learned:
// - MTC2:
//   - Moves the lower 16 bits from a GPR into a vector register starting at byte index e. The upper 16 bit are discarded
//   - If e==15, the lower 8 bit are discarded (there is no wrap-around)
// - MFC2:
//   - MFC2 does the same in reverse. The result is sign-extended from i16
//   - Unlike for MTC2, for e==15, there IS wrap-around.
pub struct MTC2 {}

impl Test for MTC2 {
    fn name(&self) -> &str { "RSP MTC2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Preload the vector with this
        const CLEARED_VECTOR: Vector = Vector::from_u16([0xAABB, 0xCCDD, 0xEEFF, 0xABBA, 0xBCCB, 0xCDDC, 0xEFFE, 0xACCA]);
        SPMEM::write_vector_into_dmem(0x00, &CLEARED_VECTOR);

        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        // Preload the target vectors. Load an extra at the end
        for vr in VR::V0..=VR::V16 {
            assembler.write_lqv(vr, E::_0, 0, GPR::R0);
        }

        // Move a GPR into the vector
        assembler.write_li(GPR::AT, 0x12345678);
        for (vr, e) in (VR::V0..=VR::V15).zip(E::_0..=E::_15) {
            assembler.write_mtc2(vr, GPR::AT, e);
        }

        // Write them back
        for vr in VR::V0..=VR::V16 {
            assembler.write_sqv(vr, E::_0, (0x100 + vr.index() * 0x10) as i32, GPR::R0);
        }

        assembler.write_break();

        RSP::run_and_wait(0);

        for i in 0..=15 {
            let mut expected = CLEARED_VECTOR;
            expected.set8(i & 0xF, 0x56);
            if i < 15 {
                expected.set8((i + 1) & 0xF, 0x78);
            }
            soft_assert_eq_vector(SPMEM::read_vector_from_dmem(0x100 + i * 0x10), expected, || format!("MTC2 (e={})", i))?;
        }
        soft_assert_eq_vector(SPMEM::read_vector_from_dmem(0x100 + 16 * 0x10), CLEARED_VECTOR, || format!("MTC2 (with e=15) spilled into unrelated vector"))?;

        Ok(())
    }
}

pub struct MFC2 {}

impl Test for MFC2 {
    fn name(&self) -> &str { "RSP MFC2" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Preload the vector with this
        const TEST_VECTOR1: Vector = Vector::from_u16([0x1122, 0x3344, 0x5566, 0x7788, 0x9887, 0x7665, 0x5443, 0x3221]);
        const TEST_VECTOR2: Vector = Vector::from_u16([0xDEAD, 0xDEAD, 0xDEAD, 0xDEAD, 0xDEAD, 0xDEAD, 0xDEAD, 0xDEAD]);
        SPMEM::write_vector_into_dmem(0x00, &TEST_VECTOR1);
        SPMEM::write_vector_into_dmem(0x10, &TEST_VECTOR2);

        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        // Load the source vector and the one after it
        assembler.write_lqv(VR::V5, E::_0, 0x00, GPR::R0);
        assembler.write_lqv(VR::V6, E::_0, 0x10, GPR::R0);

        // Move a GPR into the vector
        for (gpr, e) in (GPR::T0..=GPR::S7).zip(E::_0..=E::_15) {
            assembler.write_li(gpr, 0xEEEEEEEE);
            assembler.write_mfc2(VR::V5, gpr, e);
        }

        // Write them back
        for (i, gpr) in (GPR::T0..=GPR::S7).enumerate() {
            assembler.write_sw(gpr, GPR::R0, (i * 4) as i16);
        }

        assembler.write_break();

        RSP::run_and_wait(0);

        for i in 0..=15 {
            let expected = ((TEST_VECTOR1.get8(i) as i16) << 8) | (TEST_VECTOR1.get8((i + 1) & 0xF) as i16);
            soft_assert_eq2(SPMEM::read(i * 4), expected as u32, || format!("MFC2 (e={})", i))?;
        }

        Ok(())
    }
}
