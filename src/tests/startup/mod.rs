use spinning_top::Spinlock;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use crate::cop0::Status;
use crate::println;

use crate::rsp::rsp::RSP;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_less};

static COP0_STATUS_EVERDRIVE_BUG: Spinlock<bool> = Spinlock::new(false);

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

        // BadVAddr is not reset during a reset, but it has a known value after turning on
        //soft_assert_eq(crate::cop0::badvaddr(), 0xFFFFFFFF_FFFFFFFF, "COP0 BadVAddr")?;

        // ExceptPC/ErrorEPC are usually 0xFFFFFFFF_FFFFFFFF after first turn on, but can be different after reset
        //soft_assert_eq(crate::cop0::exceptpc(), 0xFFFFFFFF_FFFFFFFF, "COP0 ExceptPC")?;
        //soft_assert_eq(crate::cop0::errorepc(), 0xFFFFFFFF_FFFFFFFF, "COP0 ErrorEPC")?;

        // COP0 Status: This should be 0x3400_0000, and we should check for that. We can also allow
        // if soft_reset is true as that happens after the reset button.
        // The EverDrive has a bug however and sets the wrong value. If we detect that,
        // TearDownTest will report it
        let status = crate::cop0::status();
        soft_assert_eq(crate::cop0::status_64(), status.raw_value() as u64, "COP0 Status DMFC0 has to return same value as MFC0")?;
        const STATUS_EXPECTED: Status = Status::new().with_cop1usable(true).with_cop0usable(true).with_fpu64(true);
        const STATUS_EVERDRIVE64: Status = Status::new().with_cop1usable(true).with_fpu64(true).with_soft_reset(true).with_kx(true).with_sx(true).with_ux(true);

        soft_assert_eq(crate::cop0::status_64(), status.raw_value() as u64, "COP0 Status DMFC0 has to return same value as MFC0")?;
        if status.with_soft_reset(false) == STATUS_EXPECTED {
            // all good
        } else if status == STATUS_EVERDRIVE64 {
            // wrong, but print at the end
            *COP0_STATUS_EVERDRIVE_BUG.lock() = true;
        } else {
            soft_assert_eq(status, STATUS_EXPECTED, "COP0 Status at init")?;
        }

        // RSP Status
        soft_assert_eq(RSP::status(), 0x1, "RSP STATUS")?;
        soft_assert_eq(RSP::pc(), 0x0, "RSP PC")?;

        // COP1 control word
        // This doesn't have a fixed value. After a hardreset it is 0, but after a soft reset it is whatever it was before
        //soft_assert_eq(0x01000800, cfc1::<31>(), "COP1 FCSR")?;

        Ok(())
    }
}

pub struct TearDownTest {}

impl Test for TearDownTest {
    fn name(&self) -> &str { "TearDownTest" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // This tests COP0 status that was recorded all the way at the beginning

        if *COP0_STATUS_EVERDRIVE_BUG.lock() {
            println!("EverDrive64 bug detected. Init COP0.Status to 0x34000000");
        }

        Ok(())
    }
}