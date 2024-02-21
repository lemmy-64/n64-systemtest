use alloc::format;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use core::cmp::max;
use crate::memory_map::MemoryMap;
use crate::pi::{Pi, PiStatusRead, PiStatusWrite};
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq2};
use crate::uncached_memory::UncachedHeapMemory;

const RDRAM_PAGE_SIZE: usize = 2048;

const DATA: [u64; 4] = [0x01234567_89ABCDEF, 0x21436587_99BADCFE, 0xA9887766_55443322, 0x32445566_29384756];

// This should really be an array of u8, but we need to use u16 to gurantee 2-byte alignment
const COUNTER16: [u16; 512] = {
    let mut result = [0u16; 512];
    let mut i = 0u16;
    while i < 512 {
        result[(i/2) as usize] = (i & 255) * 256 + ((i & 255)+1);
        i += 2;
    }
    result
};

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
    fn name(&self) -> &str { "cart_memory: DMA CART -> RDRAM" }

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

fn test_pidma_transfer(rdram_misalign: isize, mut page_offset: isize, cart_addr: u32, requested_size: isize, expected: &[u8], cart_transfer_size: isize, rdram_transfer_size: isize) -> Result<(), String> {
    // Clear Pi, in case there's an error or interrupt pending
    Pi::set_status(PiStatusWrite::new().with_reset(true).with_clear_interrupt(true));
    soft_assert_eq(Pi::status(), PiStatusRead::new(), "PI Status before dma")?;

    let mut target = UncachedHeapMemory::<u8>::new_with_align(4096, RDRAM_PAGE_SIZE);
    for i in 0..4096 {
        target.write(i, 0xaa);
    }
    
    page_offset = max(page_offset, 16);
    let dram_addr = target.start_phyiscal() as u32 + rdram_misalign as u32 + page_offset as u32;

    Pi::set_cart_address(cart_addr);
    Pi::set_dram_address(dram_addr);
    Pi::set_write_length(requested_size as u32 - 1);
    while Pi::status().dma_busy() {}

    let actual_size = expected.len() as isize;
    for i in -16..0 {
        soft_assert_eq2(target.read((i+rdram_misalign+page_offset) as usize), 0xaa, || format!("Prefix byte at offset {}", i))?;
    }
    for i in 0..actual_size {
        soft_assert_eq2(target.read((i+rdram_misalign+page_offset) as usize), expected[i as usize], || format!("Byte at offset {}", i))?;
    }
    for i in actual_size..actual_size+16 {
        soft_assert_eq2(target.read((i+rdram_misalign+page_offset) as usize), 0xaa, || format!("Suffix byte at offset {}", i))?;
    }

    soft_assert_eq2((Pi::cart_address() - cart_addr) as isize, cart_transfer_size, || format!("cart address increment after DMA"))?;
    soft_assert_eq2((Pi::dram_address() - dram_addr) as isize, rdram_transfer_size,  || format!("dram address increment after DMA"))?;
    soft_assert_eq(Pi::status(), PiStatusRead::new().with_interrupt(true), "PI Status after dma")?;

    Ok(())
}


pub struct PIDMASmallSize {}

impl Test for PIDMASmallSize {
    fn name(&self) -> &str { "cart_memory: DMA CART -> RDRAM (small sizes)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { vec!{
        Box::new(1u32), Box::new(2u32), Box::new(3u32), Box::new(4u32),
        Box::new(5u32), Box::new(7u32), Box::new(124u32), Box::new(125u32),
        Box::new(126u32), Box::new(127u32), Box::new(128u32), Box::new(129u32),
        Box::new(131u32), Box::new(133u32),
    } }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        let requested_size = *value.downcast_ref::<u32>().unwrap() as isize;
        let actual_size = if requested_size >= 126 { (requested_size + 1) & !1 } else { requested_size };
        let cart_addr = MemoryMap::physical_cart_address(&COUNTER16[0] as *const u16 as *const u32) as u32;
        let cart_transfer_size = (actual_size + 1) & !1;
        let rdram_transfer_size = (actual_size + 7) & !7;
        let expected: Vec<u8> = (0..actual_size).map(|x| (x&255) as u8).collect();
        test_pidma_transfer(0, 16, cart_addr, requested_size, &expected, cart_transfer_size, rdram_transfer_size)?;
        Ok(())
    }   

}


pub struct PIDMAMisaligned {}

impl Test for PIDMAMisaligned {
    fn name(&self) -> &str { "cart_memory: DMA CART -> RDRAM (misaligned)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { vec!{
        Box::new((6u32, 1u32)), Box::new((6u32, 2u32)), Box::new((6u32, 6u32)),
        Box::new((6u32, 7u32)), Box::new((6u32, 8u32)), Box::new((6u32, 16u32)),
        Box::new((6u32, 119u32)), Box::new((6u32, 120u32)), Box::new((6u32, 121u32)),
        Box::new((6u32, 122u32)), Box::new((6u32, 128u32)), Box::new((6u32, 284u32)),
    } }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        let (rdram_misalign, size) = *value.downcast_ref::<(u32,u32)>().unwrap();
        let cart_addr = MemoryMap::physical_cart_address(&COUNTER16[0] as *const u16 as *const u32) as u32;
        let requested_size = size as isize;
        let actual_rom_size = if requested_size >= (126 - rdram_misalign as isize) { (requested_size + 1) & !1 } else { requested_size };
        let actual_size = max(0, if actual_rom_size < 128-rdram_misalign as isize { actual_rom_size - rdram_misalign as isize } else { actual_rom_size });
        let cart_transfer_size = (actual_rom_size + 1) & !1;
        let rdram_transfer_size = ((actual_size + rdram_misalign as isize + 7) & !7) - rdram_misalign as isize;
        let expected: Vec<u8> = (0..actual_size).map(|x| {
            if x >= 128-2*rdram_misalign as isize && x < 128-rdram_misalign as isize { 0xaa } else { (x&255) as u8 }
        }).collect();
        test_pidma_transfer(rdram_misalign as isize, 16, cart_addr, requested_size, &expected, cart_transfer_size, rdram_transfer_size)?;
        Ok(())
    }

}

pub struct PIDMAMisalignedCrossPage {}

impl Test for PIDMAMisalignedCrossPage {
    fn name(&self) -> &str { "cart_memory: DMA CART -> RDRAM (misaligned, cross page)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { vec!{
        Box::new((6u32, 1u32)), Box::new((6u32, 2u32)), Box::new((6u32, 6u32)),
        Box::new((6u32, 7u32)), Box::new((6u32, 8u32)), Box::new((6u32, 16u32)),
        Box::new((6u32, 50u32)), Box::new((6u32, 51u32)), Box::new((6u32, 52u32)),
        Box::new((6u32, 53u32)), Box::new((6u32, 66u32)), Box::new((6u32, 284u32)),
    } }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        let (rdram_misalign, size) = *value.downcast_ref::<(u32,u32)>().unwrap();
        let cart_addr = MemoryMap::physical_cart_address(&COUNTER16[0] as *const u16 as *const u32) as u32;
        let requested_size = size as isize;
        let actual_rom_size = if requested_size >= (64-2 - rdram_misalign as isize) { (requested_size + 1) & !1 } else { requested_size };
        let actual_size = max(0, if actual_rom_size < 64-rdram_misalign as isize { actual_rom_size - rdram_misalign as isize } else { actual_rom_size });
        let cart_transfer_size = (actual_rom_size + 1) & !1;
        let rdram_transfer_size = ((actual_size + rdram_misalign as isize + 7) & !7) - rdram_misalign as isize;
        let expected: Vec<u8> = (0..actual_size).map(|x| {
            if x >= 64-2*rdram_misalign as isize && x < 64-rdram_misalign as isize { 0xaa } else { (x&255) as u8 }
        }).collect();
        test_pidma_transfer(rdram_misalign as isize, 2048-64, cart_addr, requested_size, &expected, cart_transfer_size, rdram_transfer_size)?;
        Ok(())
    }

}


pub struct PIDMAMisalignedEndOfPage {}

impl Test for PIDMAMisalignedEndOfPage {
    fn name(&self) -> &str { "cart_memory: DMA CART -> RDRAM (misaligned, end of page)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { vec!{
        Box::new((2u32, 1u32)), Box::new((2u32, 2u32)), Box::new((2u32, 4u32)), Box::new((2u32, 6u32)),
        Box::new((2u32, 7u32)), Box::new((2u32, 8u32)), Box::new((2u32, 16u32)),
        Box::new((2u32, 131u32)), Box::new((2u32, 132u32)), Box::new((2u32, 133u32)),
        Box::new((2u32, 224u32)),

        Box::new((6u32, 1u32)), Box::new((6u32, 2u32)), Box::new((6u32, 4u32)), Box::new((6u32, 6u32)),
        Box::new((6u32, 7u32)), Box::new((6u32, 8u32)), Box::new((6u32, 16u32)),
        Box::new((6u32, 123u32)), Box::new((6u32, 124u32)), Box::new((6u32, 125u32)), Box::new((6u32, 126u32)),
        Box::new((6u32, 224u32)),
    } }

    fn run(&self, value: &Box<dyn Any>) -> Result<(), String> {
        let (rdram_misalign, size) = *value.downcast_ref::<(u32,u32)>().unwrap();
        let cart_addr = MemoryMap::physical_cart_address(&COUNTER16[0] as *const u16 as *const u32) as u32;
        let requested_size = size as isize;
        let actual_rom_size = if requested_size >= (8 - rdram_misalign as isize) { (requested_size + 1) & !1 } else { requested_size };
        let mut actual_size = max(0, if actual_rom_size < 8-rdram_misalign as isize { actual_rom_size - rdram_misalign as isize } else { actual_rom_size });
        if rdram_misalign == 2 && actual_size >= 134 as isize {
            actual_size += 2;
        }
        if rdram_misalign == 6 && actual_size >= 124 as isize {
            actual_size += 6;
        }
        let cart_transfer_size = (actual_rom_size + 1) & !1;
        let rdram_transfer_size = ((actual_size + rdram_misalign as isize + 7) & !7) - rdram_misalign as isize;
        let expected: Vec<u8> = match rdram_misalign {
            2 => (0..actual_size).map(|x| { match x {
                4..=5 => 0xaa,
                132..=133 => 0xaa,
                134..=511 => ((x-2)&255) as u8,
                _ => (x&255) as u8
            }}).collect(),
            6 => (0..actual_size).map(|x| { match x {
                0..=1 => 0xaa,
                124..=129 => 0xaa,
                130..=511 => ((x-6)&255) as u8,
                _ => (x&255) as u8
            }}).collect(),
            _ => panic!("Unexpected misalignment"),
        };
        test_pidma_transfer(rdram_misalign as isize, 2048-8, cart_addr, requested_size, &expected, cart_transfer_size, rdram_transfer_size)?;
        Ok(())
    }

}
