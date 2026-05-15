use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use arbitrary_int::{u2, u5, u27};

use crate::assembler::{Assembler, GPR};
use crate::cop0::{self, CauseException, Status, StatusKSU, make_entry_hi, make_entry_lo};
use crate::exception_handler::{
    ExceptionContext, clear_exception_return_override, expect_exception, set_exception_return_override,
};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq2};
use crate::tests::{Level, Test};
use crate::uncached_memory::UncachedHeapMemory;

const KUSEG_EXEC_BASE: u32 = 0x0001_0000;
const PAGE_WORDS: usize = 4096 / 4;

#[naked]
extern "C" fn return_via_s0_stub() {
    unsafe {
        asm!(
            ".set noat",
            ".set noreorder",
            "jr $16",
            "nop",
            options(noreturn)
        );
    }
}

fn load_u64_sequence(target: GPR, temp: GPR, value: u64) -> [u32; 6] {
    let hi = (value >> 32) as u32;
    let lo = value as u32;
    [
        Assembler::make_lui(target, (hi >> 16) as u16),
        Assembler::make_ori(target, target, hi as u16),
        Assembler::make_dsll32(target, target, 0),
        Assembler::make_lui(temp, (lo >> 16) as u16),
        Assembler::make_ori(temp, temp, lo as u16),
        Assembler::make_or(target, target, temp),
    ]
}

fn load_u32_sequence(target: GPR, value: u32) -> [u32; 2] {
    [Assembler::make_lui(target, (value >> 16) as u16), Assembler::make_ori(target, target, value as u16)]
}

fn map_kuseg_page(vaddr: u32, paddr: usize, coherency: u8) {
    unsafe {
        cop0::clear_tlb();
        cop0::set_context_64(0);
        cop0::set_xcontext_64(0);
        cop0::write_tlb(
            9,
            0,
            make_entry_lo(true, true, true, coherency, (paddr >> 12) as u32),
            make_entry_lo(true, true, true, coherency, (paddr >> 12) as u32),
            make_entry_hi(0, u27::extract_u64(vaddr as u64, 13), u2::new(0)),
        );
    }
}

fn setup_program(program: &[u32], coherency: u8) -> Result<(UncachedHeapMemory<u32>, u32), String> {
    if program.len() >= PAGE_WORDS {
        return Err("Program too large for mapped page".into());
    }
    let mut backing = UncachedHeapMemory::<u32>::new_with_align(PAGE_WORDS, 4096);
    for (index, instruction) in program.iter().enumerate() {
        backing.write(index, *instruction);
    }
    map_kuseg_page(KUSEG_EXEC_BASE, backing.start_phyiscal(), coherency);
    unsafe {
        cop0::dcache_index_writeback_invalidate_range(KUSEG_EXEC_BASE as usize, program.len() * 4 + 32);
        cop0::icache_index_invalidate_range(KUSEG_EXEC_BASE as usize, program.len() * 4 + 32);
        cop0::sync();
    }
    Ok((backing, KUSEG_EXEC_BASE))
}

pub(crate) fn run_mode_program(
    mode: StatusKSU,
    reverse_endian: bool,
    mode_64bit: bool,
    entry: u32,
    expected_exception: CauseException,
    skip_instructions: u64,
) -> Result<ExceptionContext, String> {
    let mut user_status = Status::DEFAULT
        .with_ksu(mode)
        .with_reverse_endian(reverse_endian)
        .with_exl(true)
        .with_erl(false);
    if mode_64bit {
        user_status = user_status.with_kx(true).with_sx(true).with_ux(true);
    }
    let kernel_status = Status::DEFAULT.with_exl(true).raw_value();
    let kernel_return_address = return_via_s0_stub as u32 as i32 as i64 as u64;
    let result = expect_exception(expected_exception, skip_instructions, || {
        set_exception_return_override(kernel_return_address, kernel_status);
        unsafe {
            asm!(
                ".set noat",
                ".set noreorder",
                "or $15, $31, $0",
                "jal 2f",
                "nop",
                "1:",
                "or $31, $15, $0",
                "b 3f",
                "nop",
                "2:",
                "or $16, $31, $0",
                "mtc0 {status}, $12",
                "nop",
                "nop",
                "mtc0 {entry}, $14",
                "nop",
                "nop",
                "eret",
                "3:",
                status = in(reg) user_status.raw_value(),
                entry = in(reg) entry as u32,
                out("$15") _,
                out("$16") _,
            );
        }
        clear_exception_return_override();
        Ok(())
    });

    clear_exception_return_override();
    result
}

type SegmentCase = (&'static str, StatusKSU, bool, u64, CauseException);

fn expected_ksu_bits(mode: StatusKSU) -> u32 {
    match mode {
        StatusKSU::Kernel => 0x00,
        StatusKSU::Supervisor => 0x08,
        StatusKSU::User => 0x10,
    }
}

fn run_segment_case(mode: StatusKSU, mode_64bit: bool, address: u64, expected: CauseException)
    -> Result<ExceptionContext, String> {
    let mut program = Vec::new();
    if mode_64bit {
        program.extend_from_slice(&load_u64_sequence(GPR::T0, GPR::T1, address));
    } else {
        program.extend_from_slice(&load_u32_sequence(GPR::T0, address as u32));
    }
    program.push(Assembler::make_lb(GPR::T1, 0, GPR::T0));
    program.push(Assembler::make_syscall(0x29f));
    let (_backing, entry) = setup_program(&program, 0)?;
    run_mode_program(mode, false, mode_64bit, entry, expected, 1)
}

pub struct SegmentMapDataDriven;

impl Test for SegmentMapDataDriven {
    fn name(&self) -> &str { "Privilege: memory accesses" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let cases: [SegmentCase; 45] = [
            ("k32_kuseg_map",   StatusKSU::Kernel,     false, 0x0000_0000_0000_1000, CauseException::TLBL),
            ("k32_kseg0_c",     StatusKSU::Kernel,     false, 0xffff_ffff_8000_1000, CauseException::Sys),
            ("k32_kseg1_d",     StatusKSU::Kernel,     false, 0xffff_ffff_a000_1000, CauseException::Sys),
            ("k32_ksseg_map",   StatusKSU::Kernel,     false, 0xffff_ffff_c000_1000, CauseException::TLBL),
            ("k32_kseg3_map",   StatusKSU::Kernel,     false, 0xffff_ffff_e000_1000, CauseException::TLBL),
            ("s32_suseg_map",   StatusKSU::Supervisor, false, 0x0000_0000_0000_1000, CauseException::TLBL),
            ("s32_low_unused",  StatusKSU::Supervisor, false, 0xffff_ffff_9000_1000, CauseException::AdEL),
            ("s32_sseg_map",    StatusKSU::Supervisor, false, 0xffff_ffff_c000_1000, CauseException::TLBL),
            ("s32_top_unused",  StatusKSU::Supervisor, false, 0xffff_ffff_e000_1000, CauseException::AdEL),
            ("u32_useg_map",    StatusKSU::User,       false, 0x0000_0000_0000_1000, CauseException::TLBL),
            ("u32_unused",      StatusKSU::User,       false, 0xffff_ffff_9000_1000, CauseException::AdEL),
            ("k64_xkuseg_map",  StatusKSU::Kernel,     true,  0x0000_0000_0000_1000, CauseException::TLBL),
            ("k64_xkuseg_gap",  StatusKSU::Kernel,     true,  0x0000_0100_0000_0000, CauseException::AdEL),
            ("k64_xksseg_map",  StatusKSU::Kernel,     true,  0x4000_0000_0000_1000, CauseException::TLBL),
            ("k64_xksseg_gap",  StatusKSU::Kernel,     true,  0x4000_0100_0000_0000, CauseException::AdEL),
            ("k64_xkphys0_c32", StatusKSU::Kernel,     true,  0x8000_0000_0000_1000, CauseException::Sys),
            ("k64_xkphys0_gap", StatusKSU::Kernel,     true,  0x8000_0001_0000_0000, CauseException::AdEL),
            ("k64_xkphys1_c32", StatusKSU::Kernel,     true,  0x8800_0000_0000_1000, CauseException::Sys),
            ("k64_xkphys1_gap", StatusKSU::Kernel,     true,  0x8800_0001_0000_0000, CauseException::AdEL),
            ("k64_xkphys2_d32", StatusKSU::Kernel,     true,  0x9000_0000_0000_1000, CauseException::Sys),
            ("k64_xkphys2_gap", StatusKSU::Kernel,     true,  0x9000_0001_0000_0000, CauseException::AdEL),
            ("k64_xkphys3_c32", StatusKSU::Kernel,     true,  0x9800_0000_0000_1000, CauseException::Sys),
            ("k64_xkphys3_gap", StatusKSU::Kernel,     true,  0x9800_0001_0000_0000, CauseException::AdEL),
            ("k64_xkphys4_c32", StatusKSU::Kernel,     true,  0xa000_0000_0000_1000, CauseException::Sys),
            ("k64_xkphys4_gap", StatusKSU::Kernel,     true,  0xa000_0001_0000_0000, CauseException::AdEL),
            ("k64_xkphys5_c32", StatusKSU::Kernel,     true,  0xa800_0000_0000_1000, CauseException::Sys),
            ("k64_xkphys5_gap", StatusKSU::Kernel,     true,  0xa800_0001_0000_0000, CauseException::AdEL),
            ("k64_xkphys6_c32", StatusKSU::Kernel,     true,  0xb000_0000_0000_1000, CauseException::Sys),
            ("k64_xkphys6_gap", StatusKSU::Kernel,     true,  0xb000_0001_0000_0000, CauseException::AdEL),
            ("k64_xkphys7_c32", StatusKSU::Kernel,     true,  0xb800_0000_0000_1000, CauseException::Sys),
            ("k64_xkphys7_gap", StatusKSU::Kernel,     true,  0xb800_0001_0000_0000, CauseException::AdEL),
            ("k64_xkseg_map",   StatusKSU::Kernel,     true,  0xc000_0000_0000_1000, CauseException::TLBL),
            ("k64_xkseg_gap",   StatusKSU::Kernel,     true,  0xc000_0100_0000_0000, CauseException::AdEL),
            ("k64_ckseg0_c",    StatusKSU::Kernel,     true,  0xffff_ffff_8000_1000, CauseException::Sys),
            ("k64_ckseg1_d",    StatusKSU::Kernel,     true,  0xffff_ffff_a000_1000, CauseException::Sys),
            ("k64_ckseg2_map",  StatusKSU::Kernel,     true,  0xffff_ffff_c000_1000, CauseException::TLBL),
            ("k64_ckseg3_map",  StatusKSU::Kernel,     true,  0xffff_ffff_e000_1000, CauseException::TLBL),
            ("s64_xsuseg_map",  StatusKSU::Supervisor, true,  0x0000_0000_0000_1000, CauseException::TLBL),
            ("s64_xsuseg_gap",  StatusKSU::Supervisor, true,  0x0000_0100_0000_0000, CauseException::AdEL),
            ("s64_xsseg_map",   StatusKSU::Supervisor, true,  0x4000_0000_0000_1000, CauseException::TLBL),
            ("s64_xsseg_gap",   StatusKSU::Supervisor, true,  0x4000_0100_0000_0000, CauseException::AdEL),
            ("s64_csseg_map",   StatusKSU::Supervisor, true,  0xffff_ffff_c000_1000, CauseException::TLBL),
            ("s64_top_unused",  StatusKSU::Supervisor, true,  0xffff_ffff_e000_1000, CauseException::AdEL),
            ("u64_xuseg_map",   StatusKSU::User,       true,  0x0000_0000_0000_1000, CauseException::TLBL),
            ("u64_xuseg_gap",   StatusKSU::User,       true,  0x0000_0100_0000_0000, CauseException::AdEL),
        ];
        for (name, mode, mode_64bit, address, expected) in cases {
            let context = run_segment_case(mode, mode_64bit, address, expected)
                .map_err(|e| format!("{}: {}", name, e))?;
            soft_assert_eq2(context.cause.exception(), Ok(expected), || format!("{} cause", name))?;
            soft_assert_eq2(context.status & 0x18, expected_ksu_bits(mode), || format!("{} ksu", name))?;
        }
        Ok(())
    }
}

pub struct EnterUserModeAndReturnKernel;

impl Test for EnterUserModeAndReturnKernel {
    fn name(&self) -> &str { "Privilege: kernel->user via ERET and trap return" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let (_backing, entry) = setup_program(&[Assembler::make_syscall(0x31f)], 0)?;
        let context = run_mode_program(StatusKSU::User, false, false, entry, CauseException::Sys, 1)?;
        soft_assert_eq(context.status & 0x18, 0x10, "KSU at user-mode exception")?;
        soft_assert_eq(context.status & 0x02, 0x02, "EXL at user-mode exception")?;
        soft_assert_eq(context.cause.exception(), Ok(CauseException::Sys), "Cause at user-mode exception")?;
        Ok(())
    }
}

pub struct EnterSupervisorModeAndReturnKernel;

impl Test for EnterSupervisorModeAndReturnKernel {
    fn name(&self) -> &str { "Privilege: kernel->supervisor via ERET and trap return" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let (_backing, entry) = setup_program(&[Assembler::make_syscall(0x11f)], 0)?;
        let context = run_mode_program(StatusKSU::Supervisor, false, false, entry, CauseException::Sys, 1)?;
        soft_assert_eq(context.status & 0x18, 0x08, "KSU at supervisor-mode exception")?;
        soft_assert_eq(context.status & 0x02, 0x02, "EXL at supervisor-mode exception")?;
        soft_assert_eq(
            context.cause.exception(),
            Ok(CauseException::Sys),
            "Cause at supervisor-mode exception",
        )?;
        Ok(())
    }
}

pub struct UserModeCop0Unusable;

impl Test for UserModeCop0Unusable {
    fn name(&self) -> &str { "Privilege: COP0 unusable in user mode" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let (_backing, entry) = setup_program(&[Assembler::make_mfc0(GPR::T0, u5::new(12))], 0)?;
        let context = run_mode_program(StatusKSU::User, false, false, entry, CauseException::CopUnusable, 1)?;
        soft_assert_eq(context.status & 0x18, 0x10, "KSU on CopUnusable from user mode")?;
        soft_assert_eq(
            context.cause.exception(),
            Ok(CauseException::CopUnusable),
            "Cause on CopUnusable from user mode",
        )?;
        soft_assert_eq(
            context.cause.coprocessor_error(),
            u2::new(0),
            "CE on CopUnusable from user mode",
        )?;
        Ok(())
    }
}

pub struct SupervisorModeCop0Unusable;

impl Test for SupervisorModeCop0Unusable {
    fn name(&self) -> &str { "Privilege: COP0 unusable in supervisor mode" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let (_backing, entry) = setup_program(&[Assembler::make_mfc0(GPR::T0, u5::new(12))], 0)?;
        let context = run_mode_program(StatusKSU::Supervisor, false, false, entry, CauseException::CopUnusable, 1)?;
        soft_assert_eq(context.status & 0x18, 0x08, "KSU on CopUnusable from supervisor mode")?;
        soft_assert_eq(
            context.cause.exception(),
            Ok(CauseException::CopUnusable),
            "Cause on CopUnusable from supervisor mode",
        )?;
        soft_assert_eq(
            context.cause.coprocessor_error(),
            u2::new(0),
            "CE on CopUnusable from supervisor mode",
        )?;
        Ok(())
    }
}
