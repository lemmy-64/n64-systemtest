use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

pub struct JALRSimple {}

impl Test for JALRSimple {
    fn name(&self) -> &str { "JALR (simple)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        let mut ra_result: i32;
        unsafe {
            asm!("
                .set noat
                .set noreorder
TNE $0, $0

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
                DADDIU $31, $25, 0

            ", out("$2") _, out("$3") result, out("$4") ra_result)
        }

        soft_assert_eq(result, 5, "Delay slot or jump target wasn't executed correctly")?;
        soft_assert_eq(unsafe { *(ra_result as *const u32) }, 0x34630002, "Return address does not point to correct instruction")?;



        Ok(())
    }
}

pub struct JALRWithSameRegister {}

impl Test for JALRWithSameRegister {
    fn name(&self) -> &str { "JALR (with same register)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut result: u32;
        let mut ra_result: i32;
        unsafe {
            asm!("
                .set noat
                .set noreorder
TNE $0, $0

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

            ", out("$2") ra_result, out("$3") result)
        }

        if (result & 2) != 0 {
            Err("Return address should have been set after JUMP")?
        }
        soft_assert_eq(result, 5, "Delay slot or jump target wasn't executed correctly")?;
        soft_assert_eq(unsafe { *(ra_result as *const u32) }, 0x34630002, "Return address does not point to correct instruction")?;

        Ok(())
    }
}
