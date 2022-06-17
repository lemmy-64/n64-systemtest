use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{CP0Register, GPR, RSPAssembler};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_neq};

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
pub mod op_mfc2_mtc2;
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
pub mod op_vector_arithmetic;
pub mod op_vector_loads;
pub mod op_vector_stores;
pub mod op_vmacf;
pub mod op_vmacq;
pub mod op_vmacu;
pub mod op_vmadh;
pub mod op_vmadl;
pub mod op_vmadm;
pub mod op_vmadn;
pub mod op_vmov_vrcp;
pub mod op_vmudh;
pub mod op_vmudl;
pub mod op_vmudm;
pub mod op_vmudn;
pub mod op_vmulf;
pub mod op_vmulq;
pub mod op_vmulu;
pub mod op_vrndn;
pub mod op_vrndp;
pub mod op_vsar;
pub mod op_xor;
pub mod op_xori;
pub mod stresstests;
pub mod stresstests_div;
pub mod wrap_around;

/// Ensure that the PC reg is properly masked with 0xFFC when being written to
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

/// This test ensures that the RSP and CPU are actually running in parallel
pub struct ParallelRunning { }

impl Test for ParallelRunning {
    fn name(&self) -> &str { "RSP running in parallel to the CPU" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        const MAX_ITERATIONS: u32 = 10000;

        let mut assembler = RSPAssembler::new(0);
        // Send signal 0 now that we're started
        assembler.write_li(GPR::A0, RSP::get_set_signal_bit(0));
        assembler.write_mtc0(CP0Register::SPStatus, GPR::A0);

        // Wait until signal 1 is set. Have a maximum number of iterations to not loop forever on broken emulators
        assembler.write_li(GPR::S1, MAX_ITERATIONS);

        let loop_start = assembler.get_jump_target();
        assembler.write_mfc0(CP0Register::SPStatus, GPR::S0);
        assembler.write_andi(GPR::S0, GPR::S0, RSP::get_is_signal_bit(1) as u16);
        assembler.write_bne(GPR::S0, GPR::R0, 4);
        assembler.write_nop();
        assembler.write_addiu(GPR::S1, GPR::S1, -1);
        assembler.write_bgtz_backwards(GPR::S1, &loop_start);
        assembler.write_nop();

        // BNE target: Write counter to see whether we timed out or got the signal
        assembler.write_sw(GPR::S1, GPR::R0, 0);
        assembler.write_break();

        // Ensure both signals are off, then start RSP
        RSP::clear_signal(0);
        RSP::clear_signal(1);
        RSP::start_running(0);

        // Wait for RSP to signal us
        let mut counter = MAX_ITERATIONS;
        while (!RSP::is_signal(0)) && (counter > 0) {
            counter -= 1;
        }

        // Signal RSP back
        RSP::set_signal(1);

        RSP::wait_until_rsp_is_halted();

        soft_assert_neq(counter, 0, "Timed out in CPU waiting for signal 0 from RSP")?;
        soft_assert_neq(SPMEM::read(0x0), 0, "Timed out on RSP waiting for signal 1 from CPU")?;

        Ok(())
    }
}