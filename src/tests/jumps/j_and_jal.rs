use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_neq};

pub struct JWithinDelay {}

impl Test for JWithinDelay {
    fn name(&self) -> &str { "J: Within delay slot of another J" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // This test answers the age old question: What happens to a J within a delay slot of another J?
        // (Answer: It is ignored)
        let mut result: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                LUI $3, 0x0000
                J 1f
                J 2f
                J 4f
                J 8f
                NOP

1:              ORI $3, $3, 1
2:              ORI $3, $3, 2
4:              ORI $3, $3, 4
8:              ORI $3, $3, 8
            ", out("$3") result)
        }

        soft_assert_eq(result, 0xF, "J within a delay slot should be ignored")?;

        Ok(())
    }
}

pub struct JALWithinDelay {}

impl Test for JALWithinDelay {
    fn name(&self) -> &str { "JAL: Within delay slot of J" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // JWithinDelay showed that the first J wins over the second. Is this also true for the
        // side-effect of RA-setting that JAL does?
        // Answer: Yes, but only kind-of. The delay slot is set to the target of the jump (+4)
        let mut result: u32;
        let mut ra_was_set: u32;
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
                JAL 2f
                JAL 4f
                JAL 8f
                NOP
                NOP
                NOP
                NOP
                NOP

1:              ORI $3, $3, 1
                NOP
                NOP
2:              ORI $3, $3, 2
                NOP
                NOP
4:              ORI $3, $3, 4
                NOP
                NOP
8:              ORI $3, $3, 8

                DSUB $4, $31, $25  // Calculate difference between RA and previous RA. If non-0, the JAL in delay set it

                DADDIU $31, $25, 0  // Restore original RA
            ", out("$3") result, out("$4") ra_was_set, out("$25") _)
        }

        soft_assert_eq(result, 0xF, "J within a delay slot should not take over target jump address")?;
        soft_assert_eq(ra_was_set, 44, "JAL in delay slot writes target address+4 of original jump into delay slot")?;

        Ok(())
    }
}

pub struct JALDelayRAVisibility {}

impl Test for JALDelayRAVisibility {
    fn name(&self) -> &str { "JAL: Read RA within delay slot" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // If the delay slot reads RA, will it see the new or the old value?
        // Answer: The new one
        let mut ra_delay: u32;
        let mut ra_after: u32;
        let mut ra_before: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                DADDIU $25, $31, 0  // Stash RA in $25

                JAL 0f
                ADDIU $2, $31, 0  // delay slot
0:              ADDIU $3, $31, 0  // target

                DADDIU $31, $25, 0  // Restore original RA
            ", out("$2") ra_delay, out("$3") ra_after, out("$25") ra_before)
        }

        soft_assert_neq(ra_after, ra_before, "JAL didn't update RA")?;
        soft_assert_eq(ra_delay, ra_after, "New RA value should be visible within delay slot")?;

        Ok(())
    }
}

pub struct JALWithinDelayOfJALR {}

impl Test for JALWithinDelayOfJALR {
    fn name(&self) -> &str { "JAL: Within delay slot of JALR" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // This one is tricky for recompilers: RA within delay has to set $31 to a value
        // that is not known at compile time
        let mut result: u32;
        let mut original_ra: u32;
        let mut ra_after_jalr: u32;
        let mut ra_after_jal: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                DADDIU $25, $31, 0  // Stash RA in $25
                LUI $3, 0x0000      // Clear result

                JAL 0f              // Do a normal JAL to figure out the current PC
                NOP
0:              DADDIU $25, $31, 0  // Stash RA in $25
                DADDIU $24, $25, 36

                JALR $4, $24
                JAL 2f
                NOP
                NOP
                NOP
                NOP
                NOP

1:              ORI $3, $3, 1
                NOP
                NOP
                NOP
2:              ORI $3, $3, 2
                NOP
                NOP
                NOP

                DADDIU $5, $31, 0

                DADDIU $31, $25, 0  // Restore original RA
            ", out("$3") result, out("$4") ra_after_jalr, out("$5") ra_after_jal, out("$24") _, out("$25") original_ra)
        }

        soft_assert_eq(result, 0x3, "JAL within a delay slot should not take over target jump address")?;
        soft_assert_eq(ra_after_jalr - original_ra, 16, "JALR writes address+4 into delay slot")?;
        soft_assert_eq(ra_after_jal - original_ra, 40, "JAL in delay slot writes target address+4 of original jump into delay slot")?;

        Ok(())
    }
}

