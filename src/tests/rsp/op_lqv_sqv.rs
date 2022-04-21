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

// - The total number of bytes for LQV: min(16 - element, remaining-bytes-until-128bit)
// - The total number of bytes for SQV: remaining-bytes-until-128bit

pub struct LQVSQV {}

impl Test for LQVSQV {
    fn name(&self) -> &str { "RSP LQV/SQV" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Prepare input data
        SPMEM::write_vector16_into_dmem(0x00, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        SPMEM::write_vector16_into_dmem(0x10, &[0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);

        // Assemble RSP program
        let mut assembler = RSPAssembler::new(0);

        assembler.write_ori(GPR::V0, GPR::R0, 5);
        assembler.write_ori(GPR::V1, GPR::R0, 12);
        assembler.write_ori(GPR::A0, GPR::R0, 0x400);

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

        // SQV test. Write downwards to help catch writing out-of-bounds


        // Write with offset. Go backwards to ensure that subsequent ones don't get overwritten
        SPMEM::write_vector16_into_dmem(0x220, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        SPMEM::write_vector16_into_dmem(0x290, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assembler.write_sqv(VR::V0, E::_0, 0x240, GPR::R0);
        assembler.write_sqv(VR::V0, E::_0, 0x250, GPR::R0);
        assembler.write_sqv(VR::V0, E::_0, 0x260, GPR::R0);
        assembler.write_sqv(VR::V0, E::_0, 0x270, GPR::R0);
        assembler.write_sqv(VR::V0, E::_0, 0x280, GPR::R0);
        assembler.write_sqv(VR::V0, E::_0, 0x280, GPR::V0);
        assembler.write_sqv(VR::V0, E::_0, 0x270, GPR::V1);
        assembler.write_sqv(VR::V0, E::_13, 0x260, GPR::V1);
        assembler.write_sqv(VR::V0, E::_12, 0x250, GPR::V1);
        assembler.write_sqv(VR::V0, E::_11, 0x240, GPR::V1);

        assembler.write_sqv(VR::V0, E::_8, 0x220, GPR::R0);

        assembler.write_sqv(VR::V0, E::_0, 0x230, GPR::R0);
        assembler.write_sqv(VR::V0, E::_15, 0x230, GPR::R0);

        // Do every possible element with 0 offset
        assembler.write_sqv(VR::V0, E::_0, 0x350, GPR::R0);
        assembler.write_sqv(VR::V0, E::_1, 0x360, GPR::R0);
        assembler.write_sqv(VR::V0, E::_2, 0x370, GPR::R0);
        assembler.write_sqv(VR::V0, E::_3, 0x380, GPR::R0);
        assembler.write_sqv(VR::V0, E::_4, 0x390, GPR::R0);
        assembler.write_sqv(VR::V0, E::_5, 0x3A0, GPR::R0);
        assembler.write_sqv(VR::V0, E::_6, 0x3B0, GPR::R0);
        assembler.write_sqv(VR::V0, E::_7, 0x3C0, GPR::R0);
        assembler.write_sqv(VR::V0, E::_8, 0x3D0, GPR::R0);
        assembler.write_sqv(VR::V0, E::_9, 0x3E0, GPR::R0);
        assembler.write_sqv(VR::V0, E::_10, 0x3F0, GPR::R0);
        // continueing with A0 = 0x400 base
        assembler.write_sqv(VR::V0, E::_11, 0x000, GPR::A0);
        assembler.write_sqv(VR::V0, E::_12, 0x010, GPR::A0);
        assembler.write_sqv(VR::V0, E::_13, 0x020, GPR::A0);
        assembler.write_sqv(VR::V0, E::_14, 0x030, GPR::A0);
        assembler.write_sqv(VR::V0, E::_15, 0x040, GPR::A0);

        assembler.write_break();

        RSP::run_and_wait(0);

        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x100), [0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF], "LQV[e=0]/SQV")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x110), [0xFF, 0x100, 0x2300, 0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00], "LQV[e=1]/SQV")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x120), [0x00, 0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD], "LQV[e=2]/SQV")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x130), [0x00, 0xFF, 0x0100, 0x2300, 0x4500, 0x6700, 0x8900, 0xab00], "LQV[e=3]/SQV")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x140), [0x00, 0x00, 0x00, 0x00, 0xFF01, 0x23, 0x45, 0x67], "LQV[e=8]/SQV")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x150), [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x0100], "LQV[e=13]/SQV")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x160), [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF01], "LQV[e=14]/SQV")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x170), [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00FF], "LQV[e=15]/SQV")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x180), [0xFF01, 0x23, 0x45, 0x67, 0xFF01, 0x23, 0x45, 0x67], "LQV[e=0]/LQV[e=8]/SQV")?;

        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x190), [0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00, 0xEFAB, 0xCD, 0xEF], "LQV (unaligned)/SQV")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x200), [0xFF01, 0x23, 0x45, 0x4500, 0x6700, 0x8900, 0xab00, 0xcd00], "LQV (unaligned > element specifier)/SQV")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x210), [0xFF01, 0x23, 0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00, 0xEFEF], "LQV (unaligned < element specifier)/SQV")?;

        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x220), [0x89, 0xAB, 0xCD, 0xEF, 0xFF01, 0x23, 0x45, 0x67], "LQV/SQV[e=0]/SQV[e=8]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x230), [0xEFFF, 0x100, 0x2300, 0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00], "LQV/SQV[e=0]/SQV[e=15]")?;

        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x240), [0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xAB00, 0xCD00], "LQV/SQV[e=0]/SQV[e=11, offset=12]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x250), [0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0x00CD, 0x00EF], "LQV/SQV[e=0]/SQV[e=12, offset=12]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x260), [0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD00, 0xEFFF], "LQV/SQV[e=0]/SQV[e=13, offset=12]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x270), [0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xFF01, 0x0023], "LQV/SQV[e=0]/SQV[e=0, offset=12]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x280), [0xFF01, 0x23, 0xFF, 0x100, 0x2300, 0x4500, 0x6700, 0x8900], "LQV/SQV[e=0]/SQV[e=0, offset=5]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x290), [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], "memory afterwards is expected empty")?;

        // even elements
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x350), [0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF], "SQV[e=0]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x370), [0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFF01], "SQV[e=2]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x390), [0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFF01, 0x23], "SQV[e=4]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x3B0), [0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFF01, 0x23, 0x45], "SQV[e=6]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x3D0), [0x89, 0xAB, 0xCD, 0xEF, 0xFF01, 0x23, 0x45, 0x67], "SQV[e=8]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x3F0), [0xAB, 0xCD, 0xEF, 0xFF01, 0x23, 0x45, 0x67, 0x89], "SQV[e=10]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x410), [0xCD, 0xEF, 0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB], "SQV[e=12]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x430), [0xEF, 0xFF01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD], "SQV[e=14]")?;

        // odd elements
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x360), [0x0100, 0x2300, 0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00, 0xEFFF], "SQV[e=1]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x380), [0x2300, 0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00, 0xEFFF, 0x0100], "SQV[e=3]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x3A0), [0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00, 0xEFFF, 0x0100, 0x2300], "SQV[e=5]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x3C0), [0x6700, 0x8900, 0xAB00, 0xCD00, 0xEFFF, 0x0100, 0x2300, 0x4500], "SQV[e=7]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x3E0), [0x8900, 0xAB00, 0xCD00, 0xEFFF, 0x0100, 0x2300, 0x4500, 0x6700], "SQV[e=9]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x400), [0xAB00, 0xCD00, 0xEFFF, 0x0100, 0x2300, 0x4500, 0x6700, 0x8900], "SQV[e=11]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x420), [0xCD00, 0xEFFF, 0x0100, 0x2300, 0x4500, 0x6700, 0x8900, 0xAB00], "SQV[e=13]")?;
        soft_assert_eq(SPMEM::read_vector16_from_dmem(0x440), [0xEFFF, 0x0100, 0x2300, 0x4500, 0x6700, 0x8900, 0xAB00, 0xCD00], "SQV[e=15]")?;


        Ok(())
    }
}
