use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use arbitrary_int::{u2, u27};

use crate::assembler::{Assembler, GPR};
use crate::cop0::{self, make_entry_hi, make_entry_lo, Status};
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;
use crate::uncached_memory::UncachedHeapMemory;

const PAGE: usize = 4096;
const PAIR: usize = 8192;
const WORDS_12K: usize = (12 * 1024) / 4;

fn uncached_index(va_byte_off: usize) -> usize {
    if va_byte_off < PAGE {
        va_byte_off / 4
    } else {
        2048 + (va_byte_off - PAGE) / 4
    }
}

fn write_u32(buf: &mut UncachedHeapMemory<u32>, va_byte_off: usize, w: u32) {
    buf.write(uncached_index(va_byte_off), w);
}

fn flush_icache_around(vaddr: u64, use_64: bool) {
    let mut a = vaddr & !0xFu64;
    let end = vaddr + PAIR as u64;
    while a < end {
        if use_64 {
            unsafe {
                cop0::cache64::<1, 0>(a);
                cop0::cache64::<0, 0>(a);
            }
        } else {
            unsafe {
                cop0::cache::<1, 0>(a as usize);
                cop0::cache::<0, 0>(a as usize);
            }
        }
        a += 16;
    }
    unsafe { cop0::sync(); }
}

fn install_split_tlb_result(buf: &mut UncachedHeapMemory<u32>, vaddr: u64, use_64: bool) -> Result<(), String> {
    let pfn0 = (buf.start_phyiscal() >> 12) as u32;
    let pfn1 = ((buf.start_phyiscal() + 2 * PAGE) >> 12) as u32;
    if pfn0 + 2 != pfn1 {
        return Err(format!("expected third 4K PFN to be first+2, got pfn0={} pfn1={}", pfn0, pfn1));
    }
    let r = u2::extract_u64(vaddr, 62);
    let vpn = u27::extract_u64(vaddr, 13);
    unsafe {
        cop0::clear_tlb();
        cop0::set_context_64(0);
        cop0::set_xcontext_64(0);
        cop0::write_tlb(
            11,
            0,
            make_entry_lo(true, true, true, 2, pfn0),
            make_entry_lo(true, true, true, 2, pfn1),
            make_entry_hi(0, vpn, r),
        );
    }
    flush_icache_around(vaddr, use_64);
    Ok(())
}

fn jalr_u32(entry: u32) -> u32 {
    let mut v0: u32;
    unsafe {
        asm!("jalr $4, $3", in("$3") entry, out("$4") _, out("$2") v0);
    }
    v0
}

fn jalr_u64(entry: u64) -> u32 {
    let mut v0: u32;
    unsafe {
        asm!(
            "ld $2, 0($5)",
            "jalr $4, $2",
            "nop",
            in("$5") &entry,
            out("$2") v0,
            out("$4") _,
        );
    }
    v0
}

fn run_linear(vaddr: u64, use_64: bool) -> Result<(), String> {
    let mut buf = UncachedHeapMemory::<u32>::new_with_align(WORDS_12K, PAGE);
    install_split_tlb_result(&mut buf, vaddr, use_64)?;
    write_u32(&mut buf, 4088, Assembler::make_addiu(GPR::V0, GPR::R0, 0));
    write_u32(&mut buf, 4092, Assembler::make_addiu(GPR::V0, GPR::V0, 1));
    write_u32(&mut buf, 4096, Assembler::make_addiu(GPR::V0, GPR::V0, 2));
    write_u32(&mut buf, 4100, Assembler::make_addiu(GPR::V0, GPR::V0, 4));
    write_u32(&mut buf, 4104, Assembler::make_addiu(GPR::V0, GPR::V0, 8));
    write_u32(&mut buf, 4108, Assembler::make_jr(GPR::A0));
    write_u32(&mut buf, 4112, Assembler::make_nop());
    flush_icache_around(vaddr, use_64);
    let r = if use_64 {
        jalr_u64(vaddr + 4088)
    } else {
        jalr_u32((vaddr + 4088) as u32)
    };
    soft_assert_eq(r, 15, "linear cross-page v0")?;
    Ok(())
}

fn run_beq_delay(vaddr: u64, use_64: bool) -> Result<(), String> {
    let mut buf = UncachedHeapMemory::<u32>::new_with_align(WORDS_12K, PAGE);
    install_split_tlb_result(&mut buf, vaddr, use_64)?;
    write_u32(&mut buf, 4088, Assembler::make_addiu(GPR::V0, GPR::R0, 0));
    write_u32(&mut buf, 4092, Assembler::make_beq(GPR::R0, GPR::R0, 1));
    write_u32(&mut buf, 4096, Assembler::make_addiu(GPR::V0, GPR::V0, 0x100));
    write_u32(&mut buf, 4100, Assembler::make_addiu(GPR::V0, GPR::V0, 0x200));
    write_u32(&mut buf, 4104, Assembler::make_jr(GPR::A0));
    write_u32(&mut buf, 4108, Assembler::make_nop());
    flush_icache_around(vaddr, use_64);
    let r = if use_64 {
        jalr_u64(vaddr + 4088)
    } else {
        jalr_u32((vaddr + 4088) as u32)
    };
    soft_assert_eq(r, 0x300, "BEQ delay cross-page v0")?;
    Ok(())
}

fn run_jr_delay(vaddr: u64, use_64: bool, jr_target: u64) -> Result<(), String> {
    let mut buf = UncachedHeapMemory::<u32>::new_with_align(WORDS_12K, PAGE);
    install_split_tlb_result(&mut buf, vaddr, use_64)?;
    let tgt = vaddr + 4100;
    let tgt32 = tgt as u32;
    let hi = (tgt32 >> 16) as u16;
    let lo = tgt32 as u16;
    write_u32(&mut buf, 4084, Assembler::make_lui(GPR::T0, hi));
    write_u32(&mut buf, 4088, Assembler::make_ori(GPR::T0, GPR::T0, lo));
    write_u32(&mut buf, 4092, Assembler::make_jr(GPR::T0));
    write_u32(&mut buf, 4096, Assembler::make_nop());
    write_u32(&mut buf, 4100, Assembler::make_addiu(GPR::V0, GPR::R0, 0x55));
    write_u32(&mut buf, 4104, Assembler::make_jr(GPR::A0));
    write_u32(&mut buf, 4108, Assembler::make_nop());
    flush_icache_around(vaddr, use_64);
    let r = if use_64 {
        let mut scratch = [0u64; 1];
        scratch[0] = jr_target;
        let entry_pc = vaddr + 4092;
        let mut v0: u32;
        unsafe {
            asm!(
                "ld $8, 0($12)",
                "ld $25, 0($5)",
                "jalr $4, $25",
                "nop",
                in("$12") scratch.as_ptr(),
                in("$5") &entry_pc,
                lateout("$2") v0,
                out("$4") _,
            );
        }
        v0
    } else {
        jalr_u32((vaddr + 4084) as u32)
    };
    soft_assert_eq(r, 0x55, "JR delay cross-page v0")?;
    Ok(())
}

fn run_branch_in_branch_inner_in_outer_delay(vaddr: u64, use_64: bool) -> Result<(), String> {
    let mut buf = UncachedHeapMemory::<u32>::new_with_align(WORDS_12K, PAGE);
    install_split_tlb_result(&mut buf, vaddr, use_64)?;
    write_u32(&mut buf, 4088, Assembler::make_addiu(GPR::V0, GPR::R0, 0));
    write_u32(&mut buf, 4092, Assembler::make_beq(GPR::R0, GPR::R0, 1));
    write_u32(&mut buf, 4096, Assembler::make_beq(GPR::R0, GPR::R0, 1));
    write_u32(&mut buf, 4100, Assembler::make_addiu(GPR::V0, GPR::V0, 2));
    write_u32(&mut buf, 4104, Assembler::make_addiu(GPR::V0, GPR::V0, 20));
    write_u32(&mut buf, 4108, Assembler::make_jr(GPR::A0));
    write_u32(&mut buf, 4112, Assembler::make_nop());
    flush_icache_around(vaddr, use_64);
    let r = if use_64 {
        jalr_u64(vaddr + 4088)
    } else {
        jalr_u32((vaddr + 4088) as u32)
    };
    soft_assert_eq(r, 22, "branch-in-branch inner in outer delay v0")?;
    Ok(())
}

fn run_branch_in_branch_inner_on_next_page(vaddr: u64, use_64: bool) -> Result<(), String> {
    let mut buf = UncachedHeapMemory::<u32>::new_with_align(WORDS_12K, PAGE);
    install_split_tlb_result(&mut buf, vaddr, use_64)?;
    write_u32(&mut buf, 4088, Assembler::make_addiu(GPR::V0, GPR::R0, 0));
    write_u32(&mut buf, 4092, Assembler::make_beq(GPR::R0, GPR::R0, 1));
    write_u32(&mut buf, 4096, Assembler::make_nop());
    write_u32(&mut buf, 4100, Assembler::make_beq(GPR::R0, GPR::R0, 1));
    write_u32(&mut buf, 4104, Assembler::make_addiu(GPR::V0, GPR::V0, 2));
    write_u32(&mut buf, 4108, Assembler::make_addiu(GPR::V0, GPR::V0, 20));
    write_u32(&mut buf, 4112, Assembler::make_jr(GPR::A0));
    write_u32(&mut buf, 4116, Assembler::make_nop());
    flush_icache_around(vaddr, use_64);
    let r = if use_64 {
        jalr_u64(vaddr + 4088)
    } else {
        jalr_u32((vaddr + 4088) as u32)
    };
    soft_assert_eq(r, 22, "branch-in-branch inner on next page v0")?;
    Ok(())
}

pub struct CrossPageExecLinear32 {}

impl Test for CrossPageExecLinear32 {
    fn name(&self) -> &str { "TLB: linear icache across split 4K PFN (32-bit VA)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_linear(0x0DEA_0000u64, false)
    }
}

pub struct CrossPageExecLinear64 {}

impl Test for CrossPageExecLinear64 {
    fn name(&self) -> &str { "TLB: linear icache across split 4K PFN (64-bit VA)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }
        let r = run_linear(0xC000_00AB_C0EA_0000u64, true);
        unsafe { cop0::set_status(Status::DEFAULT); }
        r
    }
}

pub struct CrossPageExecBeqDelay32 {}

impl Test for CrossPageExecBeqDelay32 {
    fn name(&self) -> &str { "TLB: BEQ last in 4K, delay on next PFN (32-bit VA)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_beq_delay(0x0DEA_0000u64, false)
    }
}

pub struct CrossPageExecBeqDelay64 {}

impl Test for CrossPageExecBeqDelay64 {
    fn name(&self) -> &str { "TLB: BEQ last in 4K, delay on next PFN (64-bit VA)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }
        let r = run_beq_delay(0xC000_00AB_C0EA_0000u64, true);
        unsafe { cop0::set_status(Status::DEFAULT); }
        r
    }
}

pub struct CrossPageExecJrDelay32 {}

impl Test for CrossPageExecJrDelay32 {
    fn name(&self) -> &str { "TLB: JR last in 4K, delay on next PFN (32-bit VA)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_jr_delay(0x0DEA_0000u64, false, 0)
    }
}

pub struct CrossPageExecJrDelay64 {}

impl Test for CrossPageExecJrDelay64 {
    fn name(&self) -> &str { "TLB: JR last in 4K, delay on next PFN (64-bit VA)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }
        let v = 0xC000_00AB_C0EA_0000u64;
        let r = run_jr_delay(v, true, v + 4100);
        unsafe { cop0::set_status(Status::DEFAULT); }
        r
    }
}

pub struct CrossPageExecBranchInBranch32 {}

impl Test for CrossPageExecBranchInBranch32 {
    fn name(&self) -> &str { "TLB: branch in branch, inner in outer delay (32-bit VA)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_branch_in_branch_inner_in_outer_delay(0x0DEA_0000u64, false)
    }
}

pub struct CrossPageExecBranchInBranch64 {}

impl Test for CrossPageExecBranchInBranch64 {
    fn name(&self) -> &str { "TLB: branch in branch, inner in outer delay (64-bit VA)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }
        let r = run_branch_in_branch_inner_in_outer_delay(0xC000_00AB_C0EA_0000u64, true);
        unsafe { cop0::set_status(Status::DEFAULT); }
        r
    }
}

pub struct CrossPageExecBranchInBranchInnerPage32 {}

impl Test for CrossPageExecBranchInBranchInnerPage32 {
    fn name(&self) -> &str { "TLB: branch in branch, inner BEQ on next PFN (32-bit VA)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_branch_in_branch_inner_on_next_page(0x0DEA_0000u64, false)
    }
}

pub struct CrossPageExecBranchInBranchInnerPage64 {}

impl Test for CrossPageExecBranchInBranchInnerPage64 {
    fn name(&self) -> &str { "TLB: branch in branch, inner BEQ on next PFN (64-bit VA)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }
        let r = run_branch_in_branch_inner_on_next_page(0xC000_00AB_C0EA_0000u64, true);
        unsafe { cop0::set_status(Status::DEFAULT); }
        r
    }
}
