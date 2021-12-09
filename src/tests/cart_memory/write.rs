use alloc::boxed::Box;
use alloc::{format, vec};
use crate::tests::{Level, Test};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;
use crate::assembler::{Assembler, Opcode};
use crate::MemoryMap;
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_neq};

// Writing to CART:
// - Writing to any place in CART will make the next READ return that (even if addresses are different)
// - In case of multiple writes, only the first one matters
// - Writes smaller than 32 bit are extended with zeroes (e.g. SB 0x89 is extended to 0x8900_0000)
// - For 64 bit writes, only the upper 64 bit are used
// - When reading back values smaller than 32 bit, the upper bits are returned (e.g. 0x89 for 0x89abcdef)
// - The temp value disappears when either a) a value is read from cart or b) a short while has passed.
//   Reading from cart takes about 130-200 cycles, so it is quite possible that a actually causes b
// - Does any game exercise this? Actually yes! A Bug's Life does and locks up if this isn't done
//   correctly

const DATA: [u64; 2] = [0x0123456789ABCDEF, 0x2143658799BADCFE];

pub struct WriteAndReadback {}

impl Test for WriteAndReadback {
    fn name(&self) -> &str { "cart-writing: Write32, Read32 (same location)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p_cart = MemoryMap::uncached_cart_address(&DATA[0] as *const u64 as *const u32) as *mut u32;

        unsafe { p_cart.write_volatile(0xBADC0FFE) }
        unsafe { p_cart.write_volatile(0xDECAF) }
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0xBADC0FFE, "Reading first time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading second time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading third time from cart after writing")?;

        Ok(())
    }
}

pub struct WriteAndReadback2 {}

impl Test for WriteAndReadback2 {
    fn name(&self) -> &str { "cart-writing: Write32, Read32 (different location, nearby)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p_cart = MemoryMap::uncached_cart_address(&DATA[0] as *const u64 as *const u32) as *mut u32;

        unsafe { p_cart.add(1).write_volatile(0xBADC0FFE) }
        unsafe { p_cart.write_volatile(0xDECAF) }
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0xBADC0FFE, "Reading first time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading second time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading third time from cart after writing")?;

        Ok(())
    }
}

pub struct WriteAndReadback3 {}

impl Test for WriteAndReadback3 {
    fn name(&self) -> &str { "cart-writing: Write32, Read32 (beginning of rom)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p_cart = MemoryMap::uncached_cart_address(&DATA[0] as *const u64 as *const u32) as *mut u32;

        unsafe { (0xB000_0000usize as *mut u32).write_volatile(0xBADC0FFE) }
        unsafe { p_cart.write_volatile(0xDECAF) }
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0xBADC0FFE, "Reading first time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading second time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading third time from cart after writing")?;

        Ok(())
    }
}

pub struct WriteAndReadback4 {}

impl Test for WriteAndReadback4 {
    fn name(&self) -> &str { "cart-writing: Write32, Read32 (end of rom)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p_cart = MemoryMap::uncached_cart_address(&DATA[0] as *const u64 as *const u32) as *mut u32;

        unsafe { (0xBFBF_FFFCusize as *mut u32).write_volatile(0xBADC0FFE) }
        unsafe { p_cart.write_volatile(0xDECAF) }
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0xBADC0FFE, "Reading first time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading second time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading third time from cart after writing")?;

        Ok(())
    }
}

pub struct WriteAndReadback5 {}

impl Test for WriteAndReadback5 {
    fn name(&self) -> &str { "cart-writing: Write32 (outside of ROM), Read32" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p_cart = MemoryMap::uncached_cart_address(&DATA[0] as *const u64 as *const u32) as *mut u32;

        unsafe { (0xBFC0_0000usize as *mut u32).write_volatile(0xBADC0FFE) }
        unsafe { p_cart.write_volatile(0xDECAF) }
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0xDECAF, "Reading first time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading second time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading third time from cart after writing")?;

        Ok(())
    }
}

pub struct DecayAfterSomeClockCycles { }

impl DecayAfterSomeClockCycles {
    fn inner_test<const WAIT_LOOP_ITERATIONS: u32, const STORE_INSTRUCTION: u32>(expect_decay: bool) -> Result<(), String> {
        let data_ptr: u32 = MemoryMap::uncached_cart_address(&DATA[0] as *const u64) as u32;
        let result: u32;
        unsafe {
            asm!("
            LUI $4, 0xBADC
            ORI $4, $4, 0x0FFE
            .WORD {STORE_INSTRUCTION}

            // Wait a few cycles
            ORI $4, $0, {WAIT_LOOP_ITERATIONS}
2:
            ADDI $4, $4, -1
            BGTZ $4, 2b
            NOP
            // Read it again
            LW $3, 0($2)",
            STORE_INSTRUCTION = const STORE_INSTRUCTION, WAIT_LOOP_ITERATIONS = const WAIT_LOOP_ITERATIONS,
            in ("$2") data_ptr, out("$3") result, out("$4") _)
        }

        if expect_decay {
            soft_assert_eq(result, 0x01234567, format!("Expect that value is gone after {} loop iterations", WAIT_LOOP_ITERATIONS).as_str())?;
        } else {
            soft_assert_neq(result, 0x01234567, format!("Expect that value is still visible after just {} loop iterations", WAIT_LOOP_ITERATIONS).as_str())?;
        }

        Ok(())
    }

    fn test<const INSTRUCTION: u32, const LOWER_BOUND: u32, const UPPER_BOUND: u32>() -> Result<(), String> {
        for _ in 0..200 {
            // The cut-off is somewhere around 70 loop iterations, but it's not clear cut (depending
            // on how code is laid out in memory).
            Self::inner_test::<0, INSTRUCTION>(false)?;
            Self::inner_test::<LOWER_BOUND, INSTRUCTION>(false)?;
            Self::inner_test::<UPPER_BOUND, INSTRUCTION>(true)?;
        }
        Ok(())
    }
}

impl Test for DecayAfterSomeClockCycles {
    fn name(&self) -> &str { "cart-writing: Temp value decay" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> {
        vec! {
            Box::new(32u32),
            Box::new(16u32),
            Box::new(8u32),
            Box::new(64u32),
        }
    }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        // How long does the temp-value stay? A short while, but not very long
        match (*value).downcast_ref::<u32>() {
            Some(&64) => {
                const INSTRUCTION: u32 = Assembler::make_loadstore(Opcode::SD, 4, 0, 2);
                DecayAfterSomeClockCycles::test::<INSTRUCTION, 20, 100>()?;
            }
            Some(&32) => {
                const INSTRUCTION: u32 = Assembler::make_loadstore(Opcode::SW, 4, 0, 2);
                DecayAfterSomeClockCycles::test::<INSTRUCTION, 20, 100>()?;
            }
            Some(&16) => {
                const INSTRUCTION: u32 = Assembler::make_loadstore(Opcode::SH, 4, 0, 2);
                DecayAfterSomeClockCycles::test::<INSTRUCTION, 20, 100>()?;
            }
            Some(&8) => {
                const INSTRUCTION: u32 = Assembler::make_loadstore(Opcode::SB, 4, 0, 2);
                DecayAfterSomeClockCycles::test::<INSTRUCTION, 15, 70>()?;
            }
            _ => {
                return Err("Value is not valid".to_string())
            }
        }

        Ok(())
    }
}

pub struct Write32AndReadback8 {}

impl Test for Write32AndReadback8 {
    fn name(&self) -> &str { "cart-writing: Write32, Read8" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p_cart = MemoryMap::uncached_cart_address(&DATA[0] as *const u64 as *const u32) as *mut u32;

        unsafe { (0xB000_0000usize as *mut u32).write_volatile(0xBADC0FFE) }
        unsafe { p_cart.write_volatile(0xDECAF) }
        soft_assert_eq(unsafe { (p_cart as *mut u8).read_volatile() }, 0xBA, "Reading first time from cart after writing")?;
        soft_assert_eq(unsafe { (p_cart as *mut u8).read_volatile() }, 0x01, "Reading second time from cart after writing")?;

        Ok(())
    }
}

pub struct Write32AndReadback16 {}

impl Test for Write32AndReadback16 {
    fn name(&self) -> &str { "cart-writing: Write32, Read16" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p_cart = MemoryMap::uncached_cart_address(&DATA[0] as *const u64 as *const u32) as *mut u32;

        unsafe { (0xB000_0000usize as *mut u32).write_volatile(0xBADC0FFE) }
        unsafe { p_cart.write_volatile(0xDECAF) }
        soft_assert_eq(unsafe { (p_cart as *mut u16).read_volatile() }, 0xBADC, "Reading first time from cart after writing")?;
        soft_assert_eq(unsafe { (p_cart as *mut u16).read_volatile() }, 0x0123, "Reading second time from cart after writing")?;

        Ok(())
    }
}

pub struct Write8AndReadback32 {}

impl Test for Write8AndReadback32 {
    fn name(&self) -> &str { "cart-writing: Write8, Read32" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p_cart = MemoryMap::uncached_cart_address(&DATA[0] as *const u64 as *const u32) as *mut u32;

        unsafe { (0xB000_0000usize as *mut u8).write_volatile(0xBA) }
        unsafe { p_cart.write_volatile(0xDECAF) }
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0xBA000000, "Reading first time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading second time from cart after writing")?;

        Ok(())
    }
}

pub struct Write16AndReadback32 {}

impl Test for Write16AndReadback32 {
    fn name(&self) -> &str { "cart-writing: Write16, Read32" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p_cart = MemoryMap::uncached_cart_address(&DATA[0] as *const u64 as *const u32) as *mut u32;

        unsafe { (0xB000_0000usize as *mut u16).write_volatile(0xBADC) }
        unsafe { p_cart.write_volatile(0xDECAF) }
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0xBADC0000, "Reading first time from cart after writing")?;
        soft_assert_eq(unsafe { p_cart.read_volatile() }, 0x01234567, "Reading second time from cart after writing")?;

        Ok(())
    }
}

pub struct Write64AndReadback32 {}

impl Test for Write64AndReadback32 {
    fn name(&self) -> &str { "cart-writing: Write64, Read32" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let p_cart = MemoryMap::uncached_cart_address(&DATA[0] as *const u64 as *const u32) as *mut u32;

        unsafe { (0xB000_0000usize as *mut u64).write_volatile(0x98765432_1AF1231A) }
        unsafe { (p_cart as *mut u64).write_volatile(0x01010101_23232323) }
        soft_assert_eq(unsafe { (p_cart as *mut u32).read_volatile() }, 0x98765432, "Reading first time from cart after writing")?;
        soft_assert_eq(unsafe { (p_cart as *mut u32).read_volatile() }, 0x01234567, "Reading second time from cart after writing")?;

        Ok(())
    }
}
