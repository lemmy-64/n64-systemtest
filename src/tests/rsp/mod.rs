use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub mod registers;
pub mod op_addi;
pub mod op_addiu;
pub mod op_and;
pub mod op_andi;
pub mod op_add_addu;
pub mod op_branches;
pub mod op_break;
pub mod op_cfc2_ctc2;
pub mod op_j;
pub mod op_jal;
pub mod op_jr_jalr;
pub mod op_lb;
pub mod op_lbu;
pub mod op_lh;
pub mod op_lhu;
pub mod op_lw;
pub mod op_lwu;
pub mod op_lqv_sqv;
pub mod op_nor;
pub mod op_or;
pub mod op_ori;
pub mod op_sb;
pub mod op_sh;
pub mod op_shifts;
pub mod op_slt;
pub mod op_slti;
pub mod op_sltiu;
pub mod op_sltu;
pub mod op_sub_subu;
pub mod op_sw;
pub mod op_vector_loads;
pub mod op_vector_arithmetic;
pub mod op_vmacf;
pub mod op_vmadh;
pub mod op_vmadm;
pub mod op_vmadn;
pub mod op_vmudh;
pub mod op_vmudm;
pub mod op_vmudn;
pub mod op_vmulf;
pub mod op_vsar;
pub mod op_xor;
pub mod op_xori;
pub mod stresstests;
pub mod wrap_around;

// Ensure that the PC reg is properly masked with 0xFFC when being written to
pub struct PCRegMasking {

}

impl Test for PCRegMasking {
    fn name(&self) -> &str { "RSP PC REG" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        RSP::set_pc(0xFFFFFFFF);
        soft_assert_eq(RSP::pc(), 0xFFC, "RSP PC isn't masked properly on write (0xFFFFFFFF was written)")?;

        Ok(())
    }
}