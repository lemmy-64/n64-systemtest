use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::arch::asm;

use crate::cop0;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

/// Tests the behavior of the COP0 Random register after writing to Wired and executing any other instructions.
/// 
/// When writing any value to Wired (which hardware will mask to a value of 0-63 inclusive), the
/// Random register is automatically set to 31. For each instruction after, the Random register is
/// decremented by 1, and follows the following behavior (pseudo-code):
/// 
/// ```
/// instruction_cycle() {
///     if random == wired {
///         random = 31;
///     } else {
///         random = (random - 1) & 63;
///     }
/// }
/// ```
pub struct RandomDecrement;

impl Test for RandomDecrement {
    fn name(&self) -> &str { "Random (decrement)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        
        fn simulate(cycles: u32, wired: u32) -> u32 {
            let mut random = 31;
            for _ in 0..cycles {
                if random == wired {
                    random = 31;
                } else {
                    random = (random - 1) & 63;
                }
            }
            
            random
        }
        
        // Note that when mfc0 is used after mtc0, extra cycles are required. That combined with
        // the nature of the test, assembly is used to ensure timing accuracy.
        fn perform(wired: u32) -> Result<(), String> {
            let after1: u32;
            let after16: u32;
            let after31: u32;
            let after100: u32;
            
            unsafe {
                asm!("
                    mtc0 {gpr_in}, $6
                    nop
                    nop
                    mfc0 {gpr_after1}, $1
                    
                    nop; nop; nop; nop; nop; nop; nop; nop; nop; nop;
                    nop; nop; nop; nop;
                    mfc0 {gpr_after16}, $1
                    
                    nop; nop; nop; nop; nop; nop; nop; nop; nop; nop;
                    nop; nop; nop; nop;
                    mfc0 {gpr_after31}, $1
                    
                    nop; nop; nop; nop; nop; nop; nop; nop; nop; nop;
                    nop; nop; nop; nop; nop; nop; nop; nop; nop; nop;
                    nop; nop; nop; nop; nop; nop; nop; nop; nop; nop;
                    nop; nop; nop; nop; nop; nop; nop; nop; nop; nop;
                    nop; nop; nop; nop; nop; nop; nop; nop; nop; nop;
                    nop; nop; nop; nop; nop; nop; nop; nop; nop; nop;
                    nop; nop; nop; nop; nop;
                    nop; nop; nop;
                    mfc0 {gpr_after100}, $1
                ",
                gpr_in = in(reg) wired,
                gpr_after1 = out(reg) after1,
                gpr_after16 = out(reg) after16,
                gpr_after31 = out(reg) after31,
                gpr_after100 = out(reg) after100,
            )}
            
            soft_assert_eq(after1, simulate(1, wired), &format!("Random, 1 instruction after setting Wired = {}", wired))?;
            soft_assert_eq(after16, simulate(16, wired), &format!("Random, 16 instructions after setting Wired = {}", wired))?;
            soft_assert_eq(after31, simulate(31, wired), &format!("Random, 31 instructions after setting Wired = {}", wired))?;
            soft_assert_eq(after100, simulate(100, wired), &format!("Random, 100 instructions after setting Wired = {}", wired))?;
            
            Ok(())
        }
        
        // Values above 63 are technically valid, but hardware will ignore any numbers larger than 63.
        // There is separate test for register masking.
        for i in 0..=63 {
            perform(i)?;
        }
        
        Ok(())
    }
}

/// Tests if read/write masking is correct for the COP0 Random register.
/// 
/// This register is read-only. Writes are ignored. In order to test this, we need to know what value
/// Random is supposed to contain after attempting a write. This requires writing to Wired, and the
/// use of assembly code to ensure instruction-timing accuracy.
/// 
/// This test relies on similar behavior as [RandomDecrement], so if decrement fails, this likely will too.
pub struct RandomMasking;

impl Test for RandomMasking {
    fn name(&self) -> &str { "Random (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let readback: u32;
        unsafe {
            asm!("
                mtc0 {gpr_wired}, $6
                nop
                nop
                mtc0 {gpr_test}, $1
                nop
                nop
                mfc0 {gpr_readback}, $1
            ",
            gpr_wired = in(reg) 0u32,
            gpr_test = in(reg) 0xFFFFFFFFu32,
            gpr_readback = out(reg) readback,
        )}
        soft_assert_eq(readback, 27, "Random was written as 0xFFFFFFFF, Wired written as 0, expecting Random write to be ignored")?;
        
        Ok(())
    }
}

/// Tests mfc0 behavior in relation to Random being set by hardware when Wired is written to.
/// 
/// Due to CPU hazards, when the mfc0 instruction is used after mtc0, there is supposed to be two
/// instructions inbetween them. However, it is technically possible to use mtc0 earlier than
/// recommended. Compilers and assemblers typically will deal with this automatically but not always.
/// 
/// If a read from Random occurs immediately after a write to Wired, the read value will be as if
/// the write to Wired was replaced with any other instruction (e.g. nop).
/// 
/// If the read from Random occurs 1 instruction later, then the value will be 31.
pub struct RandomReadEarly;

impl Test for RandomReadEarly {
    fn name(&self) -> &str { "Random (read early)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // test immediate readback
        let start: u32;
        let readback: u32;
        unsafe {
            asm!("
                mtc0 {gpr_init_wired}, $6
                nop;
                nop; nop; nop; nop; nop;
                nop; nop; nop; nop; nop;
                
                mfc0 {gpr_start}, $1
                nop
                nop
                mtc0 {gpr_wired}, $6
                mfc0 {gpr_readback}, $1
                nop
                nop
            ",
            gpr_init_wired = in(reg) 20u32,
            gpr_start = out(reg) start,
            gpr_wired = in(reg) 5u32, // value here shouldn't matter
            gpr_readback = out(reg) readback,
        )}
        
        soft_assert_eq(start, 21, "Wired set to 20, Random decremented 10 times")?;
        soft_assert_eq(readback, 29, "Random expected to wrap at previously set Wired bound")?;
        
        // test delayed readback by 1 instruction cycle
        let start: u32;
        let readback: u32;
        unsafe {
            asm!("
                mtc0 {gpr_init_wired}, $6
                nop;
                nop; nop; nop; nop; nop;
                nop; nop; nop; nop; nop;
                
                mfc0 {gpr_start}, $1
                nop
                nop
                mtc0 {gpr_wired}, $6
                nop
                mfc0 {gpr_readback}, $1
                nop
                nop
            ",
            gpr_init_wired = in(reg) 20u32,
            gpr_start = out(reg) start,
            gpr_wired = in(reg) 5u32, // value here shouldn't matter
            gpr_readback = out(reg) readback,
        )}
        
        soft_assert_eq(start, 21, "Wired set to 20, Random decremented 10 times")?;
        soft_assert_eq(readback, 31, "Random expected to reset")?;
        
        Ok(())
    }
}

pub struct ContextMasking {}

impl Test for ContextMasking {
    fn name(&self) -> &str { "Context (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::context_64();
        for value in [0, 1, 5, 15, 30, 31, 32, 63, 64, 64, 1020, 63102, 0x0F000000, 0xFFFF0002, 0xFFFFFFFF, 0xF2345678_0000000, 0xFFFFFFFF_FFFFFFFF, 0] {
            unsafe { cop0::set_context_64(value); }
            let expected = (value & 0xFFFFFFFF_FF800000) | (previous & 0x7FFFFF);
            let readback = cop0::context_64();
            soft_assert_eq(readback, expected, format!("Context was written as {:x}", value).as_str())?;
        }
        Ok(())
    }
}

pub struct ContextMixedBitWriting {}

impl Test for ContextMixedBitWriting {
    fn name(&self) -> &str { "Context (sign extension)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::context_64();

        unsafe { cop0::set_context_64(0x12345678_00000000); }
        let expected1 = 0x12345678_00000000 | (previous & 0x7FFFFF);
        soft_assert_eq(cop0::context_64(), expected1, format!("Context after writing 64 bit").as_str())?;

        unsafe { cop0::set_context_32(0x8B000000); }
        let expected2 = 0xFFFFFFFF_8B000000 | (previous & 0x7FFFFF);
        soft_assert_eq(cop0::context_64(), expected2, format!("Writing Context (32 bit) should sign extend").as_str())?;

        unsafe { cop0::set_context_32(0x7B000000); }
        let expected3 = 0x7B000000 | (previous & 0x7FFFFF);
        soft_assert_eq(cop0::context_64(), expected3, format!("Writing Context (32 bit) should sign extend").as_str())?;

        Ok(())
    }
}

/// Tests if read/write masking is correct for the COP0 Wired register.
pub struct WiredMasking;

impl Test for WiredMasking {
    fn name(&self) -> &str { "Wired (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        unsafe { cop0::set_wired(0xFFFFFFFF); }
        soft_assert_eq(cop0::wired(), 63, "Wired was written as 0xFFFFFFFF")?;
        
        Ok(())
    }
}

pub struct XContextMasking {}

impl Test for XContextMasking {
    fn name(&self) -> &str { "XContext (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::xcontext_64();
        for value in [0xF1234567_89ABCDEF, 0xFFFFFFFF_FFFFFFFF, 0x00000000_FFFFFFFF, 0x00000001_FFFFFFFF] {
            unsafe { cop0::set_xcontext_64(value); }
            let expected = (value & 0xFFFFFFFE_00000000) | (previous & 0x00000001_FFFFFFFF);
            let readback = cop0::xcontext_64();
            soft_assert_eq(readback, expected, format!("XContext was written as {:x}. But as it is readonly it shouldn't change", value).as_str())?;
        }
        Ok(())
    }
}

pub struct XContextMaskingMixed {}

impl Test for XContextMaskingMixed {
    fn name(&self) -> &str { "XContext (masking, mixed)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::xcontext_64();

        unsafe { cop0::set_xcontext_64(0x12345678_00000000); }
        let expected1 = 0x12345678_00000000 | (previous & 0x00000001_FFFFFFFF);
        soft_assert_eq(cop0::xcontext_64(), expected1, format!("XContext after writing 64 bit").as_str())?;

        unsafe { cop0::set_xcontext_32(0x8B000000); }
        let expected2 = 0xFFFFFFFE_00000000 | (previous & 0x00000001_FFFFFFFF);
        soft_assert_eq(cop0::xcontext_64(), expected2, format!("Writing XContext (32 bit) should sign extend").as_str())?;

        unsafe { cop0::set_xcontext_32(0x7B000000); }
        let expected3 = previous & 0x00000001_FFFFFFFF;
        soft_assert_eq(cop0::xcontext_64(), expected3, format!("Writing XContext (32 bit) should sign extend").as_str())?;
        Ok(())
    }
}

pub struct BadVAddrReadOnly {}

impl Test for BadVAddrReadOnly {
    fn name(&self) -> &str { "BadVAddr (readonly)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::badvaddr();
        for value in [0xF1234567_89ABCDEF, 0x00000000_00000000, 0x00000000_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF] {
            unsafe { cop0::set_badvaddr(value); }
            let expected = previous;
            let readback = cop0::badvaddr();
            soft_assert_eq(readback, expected, format!("BadVAddr was written as {:x}. But as it is readonly it shouldn't change", value).as_str())?;
        }
        Ok(())
    }
}

pub struct ExceptPCNoMasking {}

impl Test for ExceptPCNoMasking {
    fn name(&self) -> &str { "ExceptPC (no masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0xF7654321_89ABCDEF, 0x00000000_00000000, 0x00000000_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF] {
            unsafe { cop0::set_exceptpc(value); }
            let expected = value;
            let readback = cop0::exceptpc();
            soft_assert_eq(readback, expected, format!("ExceptPC was written as 0x{:x}. It shouldn't be masked", value).as_str())?;
        }
        Ok(())
    }
}

pub struct ErrorEPCNoMasking {}

impl Test for ErrorEPCNoMasking {
    fn name(&self) -> &str { "ErrorEPC (no masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0xF7654321_89ABCDEF, 0x00000000_00000000, 0x00000000_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF, 0x12345678_AABBCCDD] {
            unsafe { cop0::set_errorepc(value); }
            let expected = value;
            let readback = cop0::errorepc();
            soft_assert_eq(readback, expected, format!("ErrorEPC was written as 0x{:x}. It shouldn't be masked", value).as_str())?;
        }
        Ok(())
    }
}

pub struct LLAddrIs32Bit {}

impl Test for LLAddrIs32Bit {
    fn name(&self) -> &str { "LLAddr (32 bit)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for value in [0xF7654321_89ABCDEF, 0x00000000_00000000, 0x00000000_FFFFFFFF, 0xFFFFFFFF_FFFFFFFF] {
            unsafe { cop0::set_lladdr(value); }
            let expected = value & 0xFFFFFFFF;
            let readback = cop0::lladdr();
            soft_assert_eq(readback, expected, format!("LLAddr was written as 0x{:x}. Only its lower 32 bit should change", value).as_str())?;
        }
        Ok(())
    }
}

pub struct StatusIs32Bit {}

impl Test for StatusIs32Bit {
    fn name(&self) -> &str { "Status (upper 32 bit ignored)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::status_64();
        for value in [0xFFFFFFFF_00000000, 0x00000000_00000000] {
            let write_value = (value & 0xFFFFFFFF_00000000) | (previous & 0xFFFFFFFF);
            unsafe { cop0::set_status_64(write_value); }
            let expected = previous;
            let readback = cop0::status_64();
            soft_assert_eq(readback, expected, format!("Status was written as 0x{:x}. It's upper 32 bit should be constant", write_value).as_str())?;
        }
        Ok(())
    }
}

/// Tests if read/write masking is correct for the COP0 Status register.
pub struct StatusMasking;

impl Test for StatusMasking {
    fn name(&self) -> &str { "Status (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::status();
        unsafe { cop0::set_status(previous | 0x00080000) }
        let readback = cop0::status();
        
        soft_assert_eq(readback, previous & 0xFFF7FFFF, "Status bit-19 set. Expected readback bit-19 to be clear")?;
        
        Ok(())
    }
}

/// Tests if read/write masking is correct for the COP0 Config register.
pub struct ConfigMasking;

impl Test for ConfigMasking {
    fn name(&self) -> &str { "Config (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let previous = cop0::config();
        unsafe { cop0::set_config((previous & 0x7F00800F) | 0x80F91B90) }
        let readback = cop0::config();
        
        soft_assert_eq(readback, (previous & 0x7F00800F) | 0x00066460, "Config written with {:#010X}")?;
        
        Ok(())
    }
}

/// Tests write/read behavior for all unused COP0 registers, using an extra COP0 write.
/// 
/// Unused registers include number 7, 21, 22, 23, 24, 25, and 31. Writes to these registers exhibit
/// odd behavior. Reads after writes, will always return the same value. But if another write occurs
/// to another COP0 register, the readback from the first register, will be whatever was written to
/// the second.
pub struct UnusedRegistersExtraMtc0;

impl Test for UnusedRegistersExtraMtc0 {
    fn name(&self) -> &str { "Unused COP0 Registers (with extra mtc0)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        #[inline]
        fn write_read_cop0<const INDEX: u32>(value: u32, junk: u32) -> u32 {
            let readback: u32;
            unsafe {
                asm!("
                    mtc0 {gpr_write}, ${cop0_reg}
                    nop
                    nop
                    mtc0 {junk}, $11
                    nop
                    nop
                    mfc0 {gpr_read}, ${cop0_reg}
                    nop
                    nop
                ",
                gpr_write = in(reg) value,
                cop0_reg = const INDEX,
                junk = in(reg) junk,
                gpr_read = out(reg) readback,
                );
            }
            
            readback
        }
        
        macro_rules! perform_test {
            ($reg:expr, $value:expr, $junk:expr) => {
                let readback = write_read_cop0::<$reg>($value, $junk);
                soft_assert_eq(readback, $junk, &format!("Unused COP0 Reg{} written with {:#010X}, then any other COP0 register written with {:#010X} before readback", $reg, $value, $junk))?;
            }
        }
        
        // Test with different write and junk values to make sure emulator isn't cheating
        for write in [0x13171A1Eu32, 0xAAAAAAAA, 0xFEDCBA98, 0x12345678, 0xFFFFFFFF] {
            for junk in [0x8BADF00Du32, 0xDEADBEEF, 0xBADDCAFE] {
                perform_test!(7, write, junk);
                perform_test!(21, write, junk);
                perform_test!(22, write, junk);
                perform_test!(23, write, junk);
                perform_test!(24, write, junk);
                perform_test!(25, write, junk);
                perform_test!(31, write, junk);
            }
        }
        
        Ok(())
    }
}

/// Tests write/read behavior for all unused COP0 registers, using an extra unrelated instructions.
/// 
/// Unused registers include number 7, 21, 22, 23, 24, 25, and 31. Writes to these registers exhibit
/// odd behavior. Reads after writes, will always return the same value. But if another write occurs
/// to another COP0 register, the readback from the first register, will be whatever was written to
/// the second.
/// 
/// This test uses extra instructions between the write and readback, that simulate other typical
/// CPU operations. These do not affect the latched state from the COP0 write, thus the readback
/// should be the same value as what was written.
pub struct UnusedRegistersExtraUnrelated;

impl Test for UnusedRegistersExtraUnrelated {
    fn name(&self) -> &str { "Unused COP0 Registers (with extra unrelated instructions)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        #[inline]
        fn write_read_cop0<const INDEX: u32>(value: u32) -> u32 {
            let readback: u32;
            unsafe {
                asm!("
                    .set noat
                    mtc0 {gpr_write}, ${cop0_reg}
                    nop
                    nop
                    addiu $0, $0, 0xABCD
                    xor $0, $5, $9
                    mult $0, $0
                    nop
                    nop
                    mfc0 {gpr_read}, ${cop0_reg}
                    nop
                    nop
                ",
                gpr_write = in(reg) value,
                cop0_reg = const INDEX,
                gpr_read = out(reg) readback,
                );
            }
            
            readback
        }
        
        macro_rules! perform_test {
            ($reg:expr, $value:expr) => {
                let readback = write_read_cop0::<$reg>($value);
                soft_assert_eq(readback, $value, &format!("Unused COP0 Reg{} written with {:#010X}, then various unrelated CPU instructions (ADDIU, XOR, MULT) before readback. Expecting same value back.", $reg, $value))?;
            }
        }
        
        // Test with different write and junk values to make sure emulator isn't cheating
        for write in [0x13171A1Eu32, 0xAAAAAAAA, 0xFEDCBA98, 0x12345678, 0xFFFFFFFF] {
            perform_test!(7, write);
            perform_test!(21, write);
            perform_test!(22, write);
            perform_test!(23, write);
            perform_test!(24, write);
            perform_test!(25, write);
            perform_test!(31, write);
        }
        
        Ok(())
    }
}

/// Tests write/read behavior for all unused COP0 registers, by reading back shortly after writing.
/// 
/// Unused registers include number 7, 21, 22, 23, 24, 25, and 31. Writes to these registers exhibit
/// odd behavior. Reads after writes, will always return the same value. But if another write occurs
/// to another COP0 register, the readback from the first register, will be whatever was written to
/// the second.
/// 
/// This test doesn't write to any other register before reading back.
pub struct UnusedRegistersWriteRead;

impl Test for UnusedRegistersWriteRead {
    fn name(&self) -> &str { "Unused COP0 Registers (write/read)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        #[inline]
        fn write_read_cop0<const INDEX: u32>(value: u32) -> u32 {
            let readback: u32;
            unsafe {
                asm!("
                    mtc0 {gpr_write}, ${cop0_reg}
                    nop
                    nop
                    mfc0 {gpr_read}, ${cop0_reg}
                    nop
                    nop
                ",
                gpr_write = in(reg) value,
                cop0_reg = const INDEX,
                gpr_read = out(reg) readback,
                );
            }
            
            readback
        }
        
        macro_rules! perform_test {
            ($reg:expr, $value:expr) => {
                let readback = write_read_cop0::<$reg>($value);
                soft_assert_eq(readback, $value, &format!("Unused COP0 Reg{} written with {:#010X}, expecting same value back", $reg, $value))?;
            }
        }
        
        // Test with different write values to make sure emulator isn't cheating
        for write in [0x13171A1Eu32, 0xAAAAAAAA, 0xFEDCBA98, 0x12345678, 0xFFFFFFFF] {
            perform_test!(7, write);
            perform_test!(21, write);
            perform_test!(22, write);
            perform_test!(23, write);
            perform_test!(24, write);
            perform_test!(25, write);
            perform_test!(31, write);
        }
        
        Ok(())
    }
}

/// Tests if read/write masking is correct for the COP0 ParityError register.
pub struct ParityErrorMasking;

impl Test for ParityErrorMasking {
    fn name(&self) -> &str { "ParityError (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }
    
    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let readback: u32;
        unsafe {
            asm!("
                mtc0 {gpr_test}, $26
                nop
                nop
                mtc0 {junk}, $11
                nop
                nop
                mfc0 {gpr_readback}, $26
                nop
                nop
            ",
            gpr_test = in(reg) 0xFFFFFFFFu32,
            junk = in(reg) 0xAA55AA55u32,
            gpr_readback = out(reg) readback,
        )}
        soft_assert_eq(readback, 0xFF, "ParityError (26) was written as 0xFFFFFFFF")?;
        
        Ok(())
    }
}

/// Tests if read/write masking is correct for the COP0 CacheError register.
pub struct CacheErrorMasking;

impl Test for CacheErrorMasking {
    fn name(&self) -> &str { "CacheError (masking)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }
    
    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let readback: u32;
        unsafe {
            asm!("
                mtc0 {gpr_test}, $27
                nop
                nop
                mtc0 {junk}, $11
                nop
                nop
                mfc0 {gpr_readback}, $27
                nop
                nop
            ",
            gpr_test = in(reg) 0xFFFFFFFFu32,
            junk = in(reg) 0xAA55AA55u32,
            gpr_readback = out(reg) readback,
        )}
        soft_assert_eq(readback, 0, "CacheError (27) was written as 0xFFFFFFFF")?;
        
        Ok(())
    }
}
