use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_neq};

fn bgezal_basic<const VALUE: u64>(expected_jump: bool) -> Result<(), String> {
    let low: u32 = VALUE as u32;
    let high: u32 = (VALUE >> 32) as u32;
    let mut result: u32;
    let mut ra_result: u32;
    unsafe {
        asm!("
            .set noat
            .set noreorder

            DADDIU $25, $31, 0  // Stash RA in $25
            LUI $31, 0          // Clear RA

            // Merge low and high into $4
            DSLL32 $4, $6, 0
            DSRL32 $4, $4, 0

            DSLL32 $3, $7, 0
            OR $4, $4, $3

            LUI $3, 0x0000

            BGEZAL $4, 1f
            ORI $3, $3, 1

            ORI $3, $3, 2
1:          ORI $3, $3, 4
            DADDIU $5, $31, 0
            DADDIU $31, $25, 0  // Restore original RA
        ", in("$6") low, in("$7") high,
            out("$3") result, out("$4") _, out("$5") ra_result)
    }

    soft_assert_eq(result, if expected_jump { 5 } else { 7 }, "BGEZAL should have jumped")?;
    soft_assert_neq(ra_result, 0, "Return address should have been set")?;
    soft_assert_eq(unsafe { *(ra_result as *const u32) }, 0x34630002, "Return address does not point to correct instruction")?;

    Ok(())
}


pub struct BGEZALTaken {}

impl Test for BGEZALTaken {
    fn name(&self) -> &str { "BGEZAL: Taken" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        bgezal_basic::<0>(true)?;
        bgezal_basic::<1>(true)?;
        bgezal_basic::<15>(true)?;
        bgezal_basic::<0xFFFF0000>(true)?;
        bgezal_basic::<0x0000FFFFFFFF0000>(true)?;

        Ok(())
    }
}

pub struct BGEZALNotTaken {}

impl Test for BGEZALNotTaken {
    fn name(&self) -> &str { "BGEZAL: Not taken" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        bgezal_basic::<0xFFFFFFFF_FFFFFFFF>(false)?;
        bgezal_basic::<0xFFFFFFFF_0FFFFFFF>(false)?;
        bgezal_basic::<0xFFFFFFFF_00000000>(false)?;

        Ok(())
    }
}

pub struct BGEZALThatChangesItsOwnCondition {}

impl Test for BGEZALThatChangesItsOwnCondition {
    fn name(&self) -> &str { "BGEZAL: Changes its own condition" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        let mut ra_result: u32;
        unsafe {
            asm!("
            .set noat
            .set noreorder

            DADDIU $25, $31, 0  // Stash RA in $25

            ORI $31, $0, 1          // Make RA > 0

            LUI $3, 0x0000

            BGEZAL $31, 1f
            ORI $3, $3, 1

            ORI $3, $3, 2
1:          ORI $3, $3, 4
            DADDIU $5, $31, 0
            DADDIU $31, $25, 0  // Restore original RA
        ", out("$3") result, out("$4") _, out("$5") ra_result)
        }

        soft_assert_eq(result, 5, "BGEZAL should have jumped")?;
        soft_assert_neq(ra_result, 0, "Return address should have been set")?;
        soft_assert_eq(unsafe { *(ra_result as *const u32) }, 0x34630002, "Return address does not point to correct instruction")?;

        Ok(())
    }
}

fn bgezall_basic<const VALUE: u64>(expected_jump: bool) -> Result<(), String> {
    let low: u32 = VALUE as u32;
    let high: u32 = (VALUE >> 32) as u32;
    let mut result: u32;
    let mut ra_result: u32;
    unsafe {
        asm!("
            .set noat
            .set noreorder

            DADDIU $25, $31, 0  // Stash RA in $25
            LUI $31, 0          // Clear RA

            // Merge low and high into $4
            DSLL32 $4, $6, 0
            DSRL32 $4, $4, 0

            DSLL32 $3, $7, 0
            OR $4, $4, $3

            LUI $3, 0x0000

            BGEZALL $4, 1f
            ORI $3, $3, 1

            ORI $3, $3, 2
1:          ORI $3, $3, 4
            DADDIU $5, $31, 0
            DADDIU $31, $25, 0  // Restore original RA
        ", in("$6") low, in("$7") high,
        out("$3") result, out("$4") _, out("$5") ra_result)
    }

    soft_assert_eq(result, if expected_jump { 5 } else { 6 }, "BGEZALL should have jumped")?;
    soft_assert_neq(ra_result, 0, "Return address should have been set")?;
    soft_assert_eq(unsafe { *(ra_result as *const u32) }, 0x34630002, "Return address does not point to correct instruction")?;

    Ok(())
}


pub struct BGEZALLTaken {}

impl Test for BGEZALLTaken {
    fn name(&self) -> &str { "BGEZALL: Taken" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        bgezall_basic::<0>(true)?;
        bgezall_basic::<1>(true)?;
        bgezall_basic::<15>(true)?;
        bgezall_basic::<0xFFFF0000>(true)?;
        bgezall_basic::<0x0000FFFFFFFF0000>(true)?;

        Ok(())
    }
}

pub struct BGEZALLNotTaken {}

impl Test for BGEZALLNotTaken {
    fn name(&self) -> &str { "BGEZALL: Not taken" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        bgezall_basic::<0xFFFFFFFF_FFFFFFFF>(false)?;
        bgezall_basic::<0xFFFFFFFF_0FFFFFFF>(false)?;
        bgezall_basic::<0xFFFFFFFF_00000000>(false)?;

        Ok(())
    }
}

pub struct BGEZALWithinDelay {}

impl Test for BGEZALWithinDelay {
    fn name(&self) -> &str { "BGEZAL: Within delay slot of J" }

    fn level(&self) -> Level { Level::TooWeird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        let mut ra_offset: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder

                DADDIU $25, $31, 0  // Stash RA in $25
                LUI $3, 0x0000      // Clear result

                JAL 0f              // Do a normal JAL to figure out the current PC
                NOP
0:              DADDIU $24, $31, 0

                J 1f
                BGEZAL $3, 2f
                NOP
                NOP
                NOP
                NOP
                NOP

1:              ORI $3, $3, 1
                ORI $3, $3, 2
                ORI $3, $3, 4
2:              ORI $3, $3, 8
                ORI $3, $3, 16
                ORI $3, $3, 32
                ORI $3, $3, 64
                ORI $3, $3, 128
                ORI $3, $3, 256
                ORI $3, $3, 512

                DSUB $4, $31, $24
                DADDIU $31, $25, 0  // Restore original RA
            ", out("$3") result, out("$4") ra_offset, out("$24") _, out("$25") _)
        }

        soft_assert_eq(result, 769, "BGEZAL within a delay slot should add its offset to the branch target address")?;
        soft_assert_eq(ra_offset, 36, "BGEZAL return address is incorrect")?;

        Ok(())
    }
}

pub struct BGEZALWithinDelayOfBEQ {}

impl Test for BGEZALWithinDelayOfBEQ {
    fn name(&self) -> &str { "BGEZAL: Within delay slot of BEQ" }

    fn level(&self) -> Level { Level::TooWeird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        let mut ra_offset: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder

                DADDIU $25, $31, 0  // Stash RA in $25
                LUI $3, 0x0000      // Clear result

                JAL 0f              // Do a normal JAL to figure out the current PC
                NOP
0:              DADDIU $24, $31, 0

                BEQ $3, $0, 1f
                BGEZAL $3, 2f
                NOP
                NOP
                NOP
                NOP
                NOP

1:              ORI $3, $3, 1
                ORI $3, $3, 2
                ORI $3, $3, 4
2:              ORI $3, $3, 8
                ORI $3, $3, 16
                ORI $3, $3, 32
                ORI $3, $3, 64
                ORI $3, $3, 128
                ORI $3, $3, 256
                ORI $3, $3, 512

                DSUB $4, $31, $24
                DADDIU $31, $25, 0  // Restore original RA
            ", out("$3") result, out("$4") ra_offset, out("$24") _, out("$25") _)
        }

        soft_assert_eq(result, 769, "BGEZAL within a delay slot should add its offset to the branch target address")?;
        soft_assert_eq(ra_offset, 36, "BGEZAL return address is incorrect")?;

        Ok(())
    }
}

pub struct BGEZALNotTakenWithinDelay {}

impl Test for BGEZALNotTakenWithinDelay {
    fn name(&self) -> &str { "BGEZAL: Within delay slot of J, but not taken" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        let mut ra_offset: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder

                DADDIU $25, $31, 0  // Stash RA in $25
                LUI $3, 0x0000      // Clear result
                LUI $4, 0xFFFF

                JAL 0f              // Do a normal JAL to figure out the current PC
                NOP
0:              DADDIU $24, $31, 0

                J 1f
                BGEZAL $4, 2f
                NOP
                NOP
                NOP
                NOP
                NOP

1:              ORI $3, $3, 1
                ORI $3, $3, 2
                ORI $3, $3, 4
2:              ORI $3, $3, 8
                ORI $3, $3, 16
                ORI $3, $3, 32
                ORI $3, $3, 64
                ORI $3, $3, 128
                ORI $3, $3, 256
                ORI $3, $3, 512

                DSUB $4, $31, $24
                DADDIU $31, $25, 0  // Restore original RA
            ", out("$3") result, out("$4") ra_offset, out("$24") _, out("$25") _)
        }

        soft_assert_eq(result, 1023, "BGEZAL within a delay slot should add its offset to the branch target address")?;
        soft_assert_eq(ra_offset, 36, "BGEZAL return address is incorrect")?;

        Ok(())
    }
}

pub struct BEQWithinDelay {}

impl Test for BEQWithinDelay {
    fn name(&self) -> &str { "BEQ: Within delay slot of J" }

    fn level(&self) -> Level { Level::TooWeird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder

                DADDIU $25, $31, 0  // Stash RA in $25
                LUI $3, 0x0000      // Clear result

                JAL 0f              // Do a normal JAL to figure out the current PC
                NOP
0:              DADDIU $25, $31, 0  // Stash RA in $25

                J 1f
                BEQ $3, $0, 2f
                NOP
                NOP
                NOP
                NOP
                NOP

1:              ORI $3, $3, 1
                ORI $3, $3, 2
                ORI $3, $3, 4
2:              ORI $3, $3, 8
                ORI $3, $3, 16
                ORI $3, $3, 32
                ORI $3, $3, 64
                ORI $3, $3, 128
                ORI $3, $3, 256
                ORI $3, $3, 512

                DADDIU $31, $25, 0  // Restore original RA
            ", out("$3") result, out("$25") _)
        }

        soft_assert_eq(result, 769, "BEQ within a delay slot should add its offset to the branch target address")?;

        Ok(())
    }
}

pub struct BEQWithinDelayOfJR {}

impl Test for BEQWithinDelayOfJR {
    fn name(&self) -> &str { "BEQ: Within delay slot of JR" }

    fn level(&self) -> Level { Level::TooWeird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder
TNE $0, $0

                DADDIU $25, $31, 0  // Stash RA in $25
                LUI $3, 0x0000      // Clear result

                JAL 0f              // Do a normal JAL to figure out the current PC
                NOP
0:              DADDIU $4, $31, 36

                JR $4
                BEQ $3, $0, 2f
                NOP
                NOP
                NOP
                NOP
                NOP

1:              ORI $3, $3, 1
                ORI $3, $3, 2
                ORI $3, $3, 4
2:              ORI $3, $3, 8
                ORI $3, $3, 16
                ORI $3, $3, 32
                ORI $3, $3, 64
                ORI $3, $3, 128
                ORI $3, $3, 256
                ORI $3, $3, 512

                DADDIU $31, $25, 0  // Restore original RA
            ", out("$3") result, out("$4") _, out("$25") _)
        }

        soft_assert_eq(result, 514, "BEQ within a delay slot should add its offset to the branch target address")?;

        Ok(())
    }
}

pub struct BEQNotTakenWithinDelay {}

impl Test for BEQNotTakenWithinDelay {
    fn name(&self) -> &str { "BEQ: Within delay slot of J, but not taken" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder

                DADDIU $25, $31, 0  // Stash RA in $25
                LUI $3, 0x0000      // Clear result
                LUI $4, 0x0001

                J 1f
                BEQ $4, $0, 2f
                NOP
                NOP
                NOP
                NOP
                NOP

1:              ORI $3, $3, 1
                ORI $3, $3, 2
                ORI $3, $3, 4
2:              ORI $3, $3, 8
                ORI $3, $3, 16
                ORI $3, $3, 32
                ORI $3, $3, 64
                ORI $3, $3, 128
                ORI $3, $3, 256
                ORI $3, $3, 512
            ", out("$3") result, out("$4") _, out("$25") _)
        }

        soft_assert_eq(result, 1023, "BEQ within a delay slot should add its offset to the branch target address")?;

        Ok(())
    }
}

