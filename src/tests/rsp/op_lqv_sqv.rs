use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, GPR, RSPAssembler, VR};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

// Load some data via LQV and store it back via SQV. Then verify the result
// Findings:
// - Both for LQV and SQV, the element specifier specifies the starting byte index into the register
// - LQV/SQV operate until the end of the 128bit element in memory (when unaligned)

// - The total number of bytes for LQV: min(15-element, remaining-bytes-until-128bit)
// - The total number of bytes for SQV: remaining-bytes-until-128bit

pub struct LQVSQV {}

impl Test for LQVSQV {
    fn name(&self) -> &str { "RSP LQV/SQV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Prepare input data
        SPMEM::write_vector_16(0x00, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        SPMEM::write_vector_16(0x10, &[0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);

        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        assembler.write_ori(GPR::V0, GPR::R0, 5);

        // LQV using various element specifier
        assembler.write_lqv(VR::V0, E::_0, 0x010, GPR::R0);

        assembler.write_lqv(VR::V1, E::_0, 0x000, GPR::R0);  // clear with zero
        assembler.write_lqv(VR::V1, E::_1, 0x010, GPR::R0);

        assembler.write_lqv(VR::V2, E::_0, 0x000, GPR::R0);  // clear with zero
        assembler.write_lqv(VR::V2, E::_2, 0x010, GPR::R0);

        assembler.write_lqv(VR::V3, E::_0, 0x000, GPR::R0);  // clear with zero
        assembler.write_lqv(VR::V3, E::_3, 0x010, GPR::R0);

        assembler.write_lqv(VR::V4, E::_0, 0x000, GPR::R0);  // clear with zero
        assembler.write_lqv(VR::V4, E::_8, 0x010, GPR::R0);

        assembler.write_lqv(VR::V5, E::_0, 0x000, GPR::R0);  // clear with zero
        assembler.write_lqv(VR::V5, E::_13, 0x010, GPR::R0);

        assembler.write_lqv(VR::V6, E::_0, 0x000, GPR::R0);  // clear with zero
        assembler.write_lqv(VR::V6,E::_14, 0x010, GPR::R0);

        assembler.write_lqv(VR::V7, E::_0, 0x000, GPR::R0);  // clear with zero
        assembler.write_lqv(VR::V7, E::_15, 0x010, GPR::R0);

        // LQV twice to ensure that elements that are ignored by the element specifier are left untouched
        assembler.write_lqv(VR::V8, E::_0, 0x010, GPR::R0);
        assembler.write_lqv(VR::V8, E::_8, 0x010, GPR::R0);

        // Load unaligned
        assembler.write_lqv(VR::V9, E::_0, 0x010, GPR::R0);
        assembler.write_lqv(VR::V9, E::_0, 0x010, GPR::V0);

        // Load unaligned  with element specifier (element specifier limits size)
        assembler.write_lqv(VR::V10, E::_0, 0x010, GPR::R0);
        assembler.write_lqv(VR::V10, E::_6, 0x010, GPR::V0);

        // Load unaligned  with element specifier (misalignment limits size)
        assembler.write_lqv(VR::V11, E::_0, 0x010, GPR::R0);
        assembler.write_lqv(VR::V11, E::_4, 0x010, GPR::V0);

        // SQV back to memory
        assembler.write_sqv(VR::V0, E::_0, 0x100, GPR::R0);
        assembler.write_sqv(VR::V1, E::_0, 0x110, GPR::R0);
        assembler.write_sqv(VR::V2, E::_0, 0x120, GPR::R0);
        assembler.write_sqv(VR::V3, E::_0, 0x130, GPR::R0);
        assembler.write_sqv(VR::V4, E::_0, 0x140, GPR::R0);
        assembler.write_sqv(VR::V5, E::_0, 0x150, GPR::R0);
        assembler.write_sqv(VR::V6, E::_0, 0x160, GPR::R0);
        assembler.write_sqv(VR::V7, E::_0, 0x170, GPR::R0);
        assembler.write_sqv(VR::V8, E::_0, 0x180, GPR::R0);
        assembler.write_sqv(VR::V9, E::_0, 0x190, GPR::R0);
        assembler.write_sqv(VR::V10, E::_0, 0x200, GPR::R0);
        assembler.write_sqv(VR::V11, E::_0, 0x210, GPR::R0);

        // Use SQV element specifier (with output cleared)
        SPMEM::write_vector_16(0x220, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assembler.write_sqv(VR::V0, E::_8, 0x220, GPR::R0);

        assembler.write_sqv(VR::V0, E::_0, 0x230, GPR::R0);
        assembler.write_sqv(VR::V0, E::_15, 0x230, GPR::R0);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read_vector_16(0x100), [0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF], "LQV[e=0]/SQV")?;
        soft_assert_eq(SPMEM::read_vector_16(0x110), [0xFF, 0x100, 0x2300, 0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00], "LQV[e=1]/SQV")?;
        soft_assert_eq(SPMEM::read_vector_16(0x120), [0x00, 0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD], "LQV[e=2]/SQV")?;
        soft_assert_eq(SPMEM::read_vector_16(0x130), [0x00, 0xFF, 0x0100, 0x2300, 0x4500, 0x6700, 0x8900, 0xab00], "LQV[e=3]/SQV")?;
        soft_assert_eq(SPMEM::read_vector_16(0x140), [0x00, 0x00, 0x00, 0x00, 0xFF01, 0x23, 0x45, 0x67], "LQV[e=8]/SQV")?;
        soft_assert_eq(SPMEM::read_vector_16(0x150), [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x0100], "LQV[e=13]/SQV")?;
        soft_assert_eq(SPMEM::read_vector_16(0x160), [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF01], "LQV[e=14]/SQV")?;
        soft_assert_eq(SPMEM::read_vector_16(0x170), [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00FF], "LQV[e=15]/SQV")?;
        soft_assert_eq(SPMEM::read_vector_16(0x180), [0xFF01, 0x23, 0x45, 0x67, 0xFF01, 0x23, 0x45, 0x67], "LQV[e=0]/LQV[e=8]/SQV")?;

        soft_assert_eq(SPMEM::read_vector_16(0x190), [0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00, 0xEFAB, 0xCD, 0xEF], "LQV (unaligned)/SQV")?;
        soft_assert_eq(SPMEM::read_vector_16(0x200), [0xFF01, 0x23, 0x45, 0x4500, 0x6700, 0x8900, 0xab00, 0xcd00], "LQV (unaligned > element specifier)/SQV")?;
        soft_assert_eq(SPMEM::read_vector_16(0x210), [0xFF01, 0x23, 0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00, 0xEFEF], "LQV (unaligned < element specifier)/SQV")?;

        soft_assert_eq(SPMEM::read_vector_16(0x220), [0x89, 0xAB, 0xCD, 0xEF, 0xFF01, 0x23, 0x45, 0x67], "LQV/SQV[e=0]/SQV[e=8]")?;
        soft_assert_eq(SPMEM::read_vector_16(0x230), [0xEFFF, 0x100, 0x2300, 0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00], "LQV/SQV[e=0]/SQV[e=15]")?;


        Ok(())
    }
}
