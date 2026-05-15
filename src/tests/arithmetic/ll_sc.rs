use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::cop0::{self, Status};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_neq};
use crate::tests::{Level, Test};
use crate::uncached_memory::UncachedHeapMemory;
use crate::cop0::{make_entry_hi, make_entry_lo};
use arbitrary_int::{u2, u27};

pub struct LL {}

impl Test for LL {
    fn name(&self) -> &str { "LL (sign extension + LLAddr)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }

        let mut memory: u32 = 0x89AB_CDEF;
        let ptr = (&mut memory as *mut u32) as isize as u64;

        let mut ll_value: u64 = 0;
        unsafe {
            asm!("
                .set noat
                LD $3, 0($2)
                LL $4, 0($3)
                SD $4, 0($5)
            ", in("$2") &ptr, out("$3") _, out("$4") _, in("$5") &mut ll_value);
        }

        let expected_lladdr = ((ptr as usize & 0x1FFF_FFFF) >> 4) as u64;
        soft_assert_eq(ll_value, 0xFFFF_FFFF_89AB_CDEF, "LL value")?;
        soft_assert_eq(cop0::lladdr(), expected_lladdr, "LLAddr after LL")?;

        Ok(())
    }
}

pub struct SC {}

impl Test for SC {
    fn name(&self) -> &str { "SC (successful store conditional)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }

        let mut memory: u32 = 0x89AB_CDEF;
        let ptr = (&mut memory as *mut u32) as isize as u64;

        let mut ll_value: u64 = 0;
        let mut sc_status: u64 = 0;
        unsafe {
            asm!("
                .set noat
                LD $3, 0($2)
                LL $5, 0($3)
                LUI $4, 0x1357
                ORI $4, $4, 0x9BDF
                SC $4, 0($3)
                SD $5, 0($6)
                SD $4, 0($7)
            ", in("$2") &ptr, out("$3") _, out("$4") _, out("$5") _, in("$6") &mut ll_value, in("$7") &mut sc_status);
        }

        soft_assert_eq(ll_value, 0xFFFF_FFFF_89AB_CDEF, "LL value before SC")?;
        soft_assert_eq(sc_status, 1, "SC success flag")?;
        soft_assert_eq(memory, 0x1357_9BDF, "Memory after SC")?;

        Ok(())
    }
}

pub struct LLD {}

impl Test for LLD {
    fn name(&self) -> &str { "LLD (load linked doubleword)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }

        let mut memory: u64 = 0x89AB_CDEF_0123_4567;
        let ptr = (&mut memory as *mut u64) as isize as u64;

        let mut lld_value: u64 = 0;
        unsafe {
            asm!("
                .set noat
                LD $3, 0($2)
                LLD $4, 0($3)
                SD $4, 0($5)
            ", in("$2") &ptr, out("$3") _, out("$4") _, in("$5") &mut lld_value);
        }

        let expected_lladdr = ((ptr as usize & 0x1FFF_FFFF) >> 4) as u64;
        soft_assert_eq(lld_value, 0x89AB_CDEF_0123_4567, "LLD value")?;
        soft_assert_eq(cop0::lladdr(), expected_lladdr, "LLAddr after LLD")?;

        Ok(())
    }
}

pub struct SCD {}

impl Test for SCD {
    fn name(&self) -> &str { "SCD (successful store conditional doubleword)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }

        let mut memory: u64 = 0x89AB_CDEF_0123_4567;
        let ptr = (&mut memory as *mut u64) as isize as u64;

        let mut lld_value: u64 = 0;
        let mut scd_status: u64 = 0;
        let write_value: u64 = 0x1020_3040_5060_7080;
        unsafe {
            asm!("
                .set noat
                LD $3, 0($2)
                LLD $5, 0($3)
                LD $4, 0($8)
                SCD $4, 0($3)
                SD $5, 0($6)
                SD $4, 0($7)
            ", in("$2") &ptr, out("$3") _, out("$4") _, out("$5") _, in("$6") &mut lld_value, in("$7") &mut scd_status, in("$8") &write_value);
        }

        soft_assert_eq(lld_value, 0x89AB_CDEF_0123_4567, "LLD value before SCD")?;
        soft_assert_eq(scd_status, 1, "SCD success flag")?;
        soft_assert_eq(memory, 0x1020_3040_5060_7080, "Memory after SCD")?;

        Ok(())
    }
}

pub struct SCAfterERET {}

impl Test for SCAfterERET {
    fn name(&self) -> &str { "SC after ERET fails and keeps LLAddr" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }

        let mut memory: u32 = 0x89AB_CDEF;
        let ptr = (&mut memory as *mut u32) as isize as u64;

        let mut ll_value: u64 = 0;
        let mut sc_status: u64 = 0;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $3, 0($2)
                LL $4, 0($3)
                SD $4, 0($5)
                MFC0 $6, $12
                ORI $6, $6, 0X2
                MTC0 $6, $12
                NOP
                NOP
                LA $7, 1f
                DMTC0 $7, $14
                NOP
                NOP
                NOP
                ERET
            1:
                LUI $4, 0x1357
                ORI $4, $4, 0x9BDF
                SC $4, 0($3)
                SD $4, 0($8)
            ", in("$2") &ptr, in("$5") &mut ll_value, in("$8") &mut sc_status,
               out("$3") _, out("$4") _, out("$6") _, out("$7") _);
        }

        soft_assert_eq(ll_value, 0xFFFF_FFFF_89AB_CDEF, "LL value before ERET")?;
        soft_assert_eq(sc_status, 0, "SC success flag after ERET")?;
        soft_assert_eq(memory, 0x89AB_CDEF, "Memory after SC following ERET")?;
        soft_assert_neq(cop0::lladdr(), 0, "LLAddr after ERET and SC must remain set")?;

        Ok(())
    }
}

pub struct SCDAfterERET {}

impl Test for SCDAfterERET {
    fn name(&self) -> &str { "SCD after ERET fails and keeps LLAddr" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::ADDRESSING_MODE_64_BIT); }

        let mut memory: u64 = 0x89AB_CDEF_0123_4567;
        let ptr = (&mut memory as *mut u64) as isize as u64;

        let mut lld_value: u64 = 0;
        let mut scd_status: u64 = 0;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LD $3, 0($2)
                LLD $4, 0($3)
                SD $4, 0($5)
                MFC0 $6, $12
                ORI $6, $6, 0x2
                MTC0 $6, $12
                NOP
                NOP
                LA $7, 1f
                DMTC0 $7, $14
                NOP
                NOP
                NOP
                ERET
            1:
                LUI $4, 0x1020
                ORI $4, $4, 0x3040
                DSLL32 $4, $4, 0
                ORI $4, $4, 0x5060
                DSLL $4, $4, 16
                ORI $4, $4, 0x7080
                SCD $4, 0($3)
                SD $4, 0($8)
            ", in("$2") &ptr, in("$5") &mut lld_value, in("$8") &mut scd_status,
               out("$3") _, out("$4") _, out("$6") _, out("$7") _);
        }

        soft_assert_eq(lld_value, 0x89AB_CDEF_0123_4567, "LLD value before ERET")?;
        soft_assert_eq(scd_status, 0, "SCD success flag after ERET")?;
        soft_assert_eq(memory, 0x89AB_CDEF_0123_4567, "Memory after SCD following ERET")?;
        soft_assert_neq(cop0::lladdr(), 0, "LLAddr after ERET and SCD must remain set")?;

        Ok(())
    }
}

pub struct SCAliasOnSamePhysicalViaTLB {}

impl Test for SCAliasOnSamePhysicalViaTLB {
    fn name(&self) -> &str { "LL/SC aliases via TLB use physical LLAddr" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_status(Status::DEFAULT); }

        let mut data = UncachedHeapMemory::<u32>::new_with_align(1024, 4096);
        data.write(0, 0x89AB_CDEF);
        let physical = data.start_phyiscal() as u64;
        let pfn = (physical >> 12) as u32;

        let ll_virtual: u32 = 0x0DEA_0000;
        let sc_virtual: u32 = 0x0BEE_0000;
        let expected_lladdr = (physical >> 4) & 0x1FF_FFFF;

        unsafe {
            cop0::clear_tlb();
            cop0::set_context_64(0);
            cop0::set_xcontext_64(0);
            cop0::write_tlb(
                10,
                0,
                make_entry_lo(true, true, true, 0, pfn),
                make_entry_lo(true, false, false, 0, 0),
                make_entry_hi(0, u27::new(ll_virtual >> 13), u2::new(0)),
            );
            cop0::write_tlb(
                11,
                0,
                make_entry_lo(true, true, true, 0, pfn),
                make_entry_lo(true, false, false, 0, 0),
                make_entry_hi(0, u27::new(sc_virtual >> 13), u2::new(0)),
            );
            cop0::set_entry_hi(0);
        }

        let mut ll_value: u32 = 0;
        let mut sc_status: u32 = 0;
        let mut readback_value: u32 = 0;
        unsafe {
            asm!("
                .set noat
                LW $3, 0($2)
                LW $4, 0($8)
                LL $5, 0($3)
                LUI $6, 0x1357
                ORI $6, $6, 0x9BDF
                SC $6, 0($4)
                SW $5, 0($9)
                SW $6, 0($10)
                LW $7, 0($3)
                SW $7, 0($11)
            ",
            in("$2") &ll_virtual, in("$8") &sc_virtual,
            out("$3") _, out("$4") _, out("$5") _, out("$6") _, out("$7") _,
            in("$9") &mut ll_value, in("$10") &mut sc_status, in("$11") &mut readback_value);
        }

        soft_assert_eq(ll_value, 0x89AB_CDEF, "LL value through first alias")?;
        soft_assert_eq(sc_status, 1, "SC status through second alias")?;
        soft_assert_eq(readback_value, 0x1357_9BDF, "Memory after SC through alias")?;
        soft_assert_eq(cop0::lladdr(), expected_lladdr, "LLAddr must track physical address")?;

        unsafe { cop0::clear_tlb(); }
        Ok(())
    }
}
