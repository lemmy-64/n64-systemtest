use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_neq};

pub struct JALRSimple {}

impl Test for JALRSimple {
    fn name(&self) -> &str { "JALR: Simple" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        let mut ra_result: i32;
        unsafe {
            asm!("
                .set noat
                .set noreorder

                ADDIU $3, $0, 0

                // Stash RA in $25
                DADDIU $25, $31, 0

                // We need to figure out the current PC in a relocatable way. We can use JAL for that
                JAL 1f
                NOP

                // RA will now point to here
                ADDIU $2, $31, 16
                JALR $4, $2
                ORI $3, $3, 1  // delay slot
                ORI $3, $3, 2  // this is jumped over. it is also the return address of JALR
                ORI $3, $3, 4  // this is the jump target

                // Restore original return address
                J 2f
                NOP

1:
                JR $31
                NOP

2:
                // Restore original RA
                DADDIU $31, $25, 0

            ", out("$2") _, out("$3") result, out("$4") ra_result, out("$25") _)
        }

        soft_assert_eq(result, 5, "Delay slot or jump target wasn't executed correctly")?;
        soft_assert_eq(unsafe { *(ra_result as *const u32) }, 0x34630002, "Return address does not point to correct instruction")?;



        Ok(())
    }
}

pub struct JALRWithSameRegister {}

impl Test for JALRWithSameRegister {
    fn name(&self) -> &str { "JALR: With same register" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        let mut ra_result: i32;
        unsafe {
            asm!("
                .set noat
                .set noreorder

                ADDIU $3, $0, 0

                // Stash RA in $25
                DADDIU $25, $31, 0

                // We need to figure out the current PC in a relocatable way. We can use JAL for that
                JAL 1f
                NOP

                // RA will now point to here
                ADDIU $2, $31, 16
                JALR $2, $2
                ORI $3, $3, 1  // delay slot
                ORI $3, $3, 2  // this is the return address
                ORI $3, $3, 4  // this is the jump target

                // Restore original return address
                J 2f
                NOP

1:
                JR $31
                NOP

2:
                DADDIU $31, $25, 0

            ", out("$2") ra_result, out("$3") result, out("$25") _)
        }

        if (result & 2) != 0 {
            Err("Return address should have been set after JUMP")?
        }
        soft_assert_eq(result, 5, "Delay slot or jump target wasn't executed correctly")?;
        soft_assert_eq(unsafe { *(ra_result as *const u32) }, 0x34630002, "Return address does not point to correct instruction")?;

        Ok(())
    }
}

pub struct JALRDelayRAVisibility {}

impl Test for JALRDelayRAVisibility {
    fn name(&self) -> &str { "JALR: Read RA within delay slot" }

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

                JAL 1f    // Use JAL to find the current PC
                NOP
1:              // RA will now point to here
                ADDIU $31, $31, 20
                JALR $31
                ADDIU $2, $31, 0  // delay slot
                NOP
                NOP
0:              ADDIU $3, $31, 0  // target

                DADDIU $31, $25, 0  // Restore original RA
            ", out("$2") ra_delay, out("$3") ra_after, out("$25") ra_before)
        }

        soft_assert_neq(ra_after, ra_before, "JALR didn't update RA")?;
        soft_assert_eq(ra_delay, ra_after, "New RA value should be visible within delay slot")?;

        Ok(())
    }
}


pub struct JRWithRegisterChangeInDelaySlot {}

impl Test for JRWithRegisterChangeInDelaySlot {
    fn name(&self) -> &str { "JR: Register change in delay slot" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // What happens if the target address of a JALR is changed with its delay slot?
        let mut result: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder

                LUI $3, 0

                // Stash RA in $25
                DADDIU $25, $31, 0

                JAL 1f    // Use JAL to find the current PC
                NOP
1:              // RA will now point to here
                ADDIU $2, $31, 16
                JR $2
                ADDIU $2, $2, 4  // modify within the delay instruction

                ORI $3, $3, 1  // delay slot
                ORI $3, $3, 2  // this is the original target
                ORI $3, $3, 4  // this is the target after the delay slot changed it

                // Restore original return address
                J 2f
                NOP

2:
                DADDIU $31, $25, 0

            ", out("$2") _, out("$3") result, out("$25") _)
        }

        soft_assert_eq(result, 6, "Jump target modification within delay slot should be ignored")?;

        Ok(())
    }
}

pub struct JALRWithRegisterChangeInDelaySlot {}

impl Test for JALRWithRegisterChangeInDelaySlot {
    fn name(&self) -> &str { "JALR: With register change in delay slot" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // What happens if the target address of a JALR is changed with its delay slot?
        let mut result: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder

                LUI $3, 0

                // Stash RA in $25
                DADDIU $25, $31, 0

                JAL 1f    // Use JAL to find the current PC
                NOP
1:              // RA will now point to here
                ADDIU $2, $31, 16
                JALR $2
                ADDIU $2, $2, 4  // modify within the delay instruction

                ORI $3, $3, 1  // delay slot
                ORI $3, $3, 2  // this is the original target
                ORI $3, $3, 4  // this is the target after the delay slot changed it

                // Restore original return address
                J 2f
                NOP

2:
                DADDIU $31, $25, 0

            ", out("$2") _, out("$3") result, out("$25") _)
        }

        soft_assert_eq(result, 6, "Jump target modification within delay slot should be ignored")?;

        Ok(())
    }
}

pub struct JRWithinDelayOfJALR {}

impl Test for JRWithinDelayOfJALR {
    fn name(&self) -> &str { "JR: Within delay slot of JALR" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        let mut original_ra: u32;
        let mut ra_after_jalr: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder
                DADDIU $25, $31, 0  // Stash RA in $25
                LUI $3, 0x0000      // Clear result

                JAL 0f              // Do a normal JAL to figure out the current PC
                NOP
0:              DADDIU $24, $31, 0
                DADDIU $23, $31, 40
                DADDIU $22, $31, 56

                JALR $4, $23
                JR $22          // within delay slot
                NOP
                NOP
                NOP
                NOP
                NOP

                ORI $3, $3, 1    // first JALR target
                NOP
                NOP
                NOP
                ORI $3, $3, 2    // second JALR target
                NOP
                NOP
                NOP
                DADDIU $31, $25, 0  // Restore original RA
            ", out("$3") result, out("$4") ra_after_jalr, out("$22") _, out("$23") _, out("$24") original_ra, out("$25") _)
        }

        soft_assert_eq(result, 0x3, "JALR within a delay slot should not take over target jump address")?;
        soft_assert_eq(ra_after_jalr - original_ra, 20, "JALR writes address+4 into delay slot")?;

        Ok(())
    }
}

pub struct JALRWithinDelayOfJALR {}

impl Test for JALRWithinDelayOfJALR {
    fn name(&self) -> &str { "JALR: Within delay slot of JALR" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        let mut original_ra: u32;
        let mut ra_after_jalr_1: u32;
        let mut ra_after_jalr_2: u32;
        unsafe {
            asm!("
                .set noat
                .set noreorder

                DADDIU $25, $31, 0  // Stash RA in $25
                LUI $3, 0x0000      // Clear result

                JAL 0f              // Do a normal JAL to figure out the current PC
                NOP
0:              DADDIU $24, $31, 0
                DADDIU $23, $31, 40
                DADDIU $22, $31, 56

                JALR $4, $23
                JALR $5, $22        // within delay slot
                NOP
                NOP
                NOP
                NOP
                NOP

                ORI $3, $3, 1    // first JALR target
                NOP
                NOP
                NOP
                ORI $3, $3, 2    // second JALR target
                NOP
                NOP
                NOP
                DADDIU $31, $25, 0  // Restore original RA
            ", out("$3") result, out("$4") ra_after_jalr_1, out("$5") ra_after_jalr_2, out("$22") _, out("$23") _, out("$24") original_ra, out("$25") _)
        }

        soft_assert_eq(result, 0x3, "JALR within a delay slot should not take over target jump address")?;
        soft_assert_eq(ra_after_jalr_1 - original_ra, 20, "JALR writes address+4 into delay slot")?;
        soft_assert_eq(ra_after_jalr_2 - original_ra, 44, "JALR in delay slot writes target address+4 of original jump into delay slot")?;

        Ok(())
    }
}

