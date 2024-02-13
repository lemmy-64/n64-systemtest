use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use crate::memory_map::MemoryMap;
use crate::pi::{Pi, PiStatusRead, PiStatusWrite};
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;
use crate::uncached_memory::UncachedHeapMemory;

const DATA: [u64; 4] = [0x01234567_89ABCDEF, 0x21436587_99BADCFE, 0xA9887766_55443322, 0x32445566_29384756];

pub struct CartAddressMasking {}

impl Test for CartAddressMasking {
    fn name(&self) -> &str { "cart_memory: cart address (masking)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for x in [0, 0xFFFFFFFF, 0xFEDCBA97] {
            Pi::set_cart_address(x);
            soft_assert_eq(Pi::cart_address(), x & 0xFFFF_FFFE, "Cart address not masked correctly")?;
            Pi::set_dram_address(x);
            soft_assert_eq(Pi::dram_address(), x & 0xFF_FFFE, "Dram address not masked correctly")?;
        }
        Ok(())
    }
}

pub struct PIDMA {}

impl Test for PIDMA {
    fn name(&self) -> &str { "cart_memory: DMA CART -> DMEM" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // Clear Pi, in case there's an error or interrupt pending
        Pi::set_status(PiStatusWrite::new().with_reset(true).with_clear_interrupt(true));
        soft_assert_eq(Pi::status(), PiStatusRead::new(), "Pi Status before dma")?;

        let cart_addr = MemoryMap::physical_cart_address(&DATA[0] as *const u64 as *const u32) as u32;
        let mut target = UncachedHeapMemory::<u32>::new_with_init_value(100, 0);
        let dram_address = target.start_phyiscal() as u32;
        Pi::set_cart_address(cart_addr);
        Pi::set_dram_address(dram_address);
        Pi::set_write_length(15);
        while Pi::status().dma_busy() {}
        soft_assert_eq(target.read(0), 0x01234567, "First word")?;
        soft_assert_eq(Pi::cart_address() - cart_addr, 16, "cart address after dma (difference to written value)")?;
        soft_assert_eq(Pi::dram_address() - dram_address, 16, "dram address after dma (difference to written value)")?;
        soft_assert_eq(Pi::status(), PiStatusRead::new().with_interrupt(true), "Pi Status after dma")?;
        Ok(())
    }
}
