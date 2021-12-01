use alloc::boxed::Box;
use crate::tests::{Level, Test};
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_less};

pub struct StartupTest {}

impl Test for StartupTest {
    fn name(&self) -> &str { "StartupTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        soft_assert_less(crate::cop0::wired(), 64, "Initial COP0 Wired")?; // Usually 0, but also seen 33 after soft reset. Don't check precise value
        soft_assert_less(crate::cop0::index() & 0x7FFFFFFF, 64, "Initial COP0 Index")?; // Usually 63, but also sometimes 0. Don't check precise value. Sometimes, highest bit is set

        // PageMask can be anything at startup - don't test it
        //soft_assert_eq(crate::cop0::pagemask() & 0xFFF0_0FFF, 0x0100_0000, "Initial COP0 PageMask")?;
        soft_assert_eq(crate::cop0::config(), 0x7006E463, "Initial COP0 Config")?;

        // Context can be anything at startup - don't test it
        //soft_assert_eq(crate::cop0::context_64() & 0xFFFEFFFF, 0x007E_FFF0, "Initial COP0 Context")?;

        // XContext can be anything at startup - don't test it
        // soft_assert_eq(crate::cop0::xcontext_64(), 0x0000_0001_FFFF_FFF0, "Initial COP0 XContext")?;

        soft_assert_eq(crate::cop0::previd(), 0xB22, "COP0 PrevID")?;

        // BadVAddr is not reset during a reset, but it has a known value after turning on
        //soft_assert_eq(crate::cop0::badvaddr(), 0xFFFFFFFF_FFFFFFFF, "COP0 BadVAddr")?;

        // ExceptPC/ErrorEPC are usually 0xFFFFFFFF_FFFFFFFF after first turn on, but can be different after reset
        //soft_assert_eq(crate::cop0::exceptpc(), 0xFFFFFFFF_FFFFFFFF, "COP0 ExceptPC")?;
        //soft_assert_eq(crate::cop0::errorepc(), 0xFFFFFFFF_FFFFFFFF, "COP0 ErrorEPC")?;

        // Status is initialized in main(), so this test isn't too valuable
        soft_assert_eq(crate::cop0::status(), 0x240000E0, "COP0 Status")?;
        soft_assert_eq(crate::cop0::status_64(), 0x240000E0, "COP0 Status (DMFC0)")?;

        Ok(())
    }
}
