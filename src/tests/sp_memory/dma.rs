use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::Any;

use crate::MemoryMap;
use crate::rsp::rsp::RSP;
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq2;

// DMA:
// - RDRAM address and SPMEM address are aligned on 8 byte boundaries (the lower 3 bits are ignoed)
// - Length: To the written value 1 is added and then it is rounded up to the next multiple of 8 (e.g. 0..7 ==> 8 bytes, 8..15 => 16 bytes)
// - A DMA goes either into IMEM or DMEM. If it overflows, it will overflow within that memory but never overlap into the other one

fn dma_test<const N: usize>(source_index: usize, spmem_index: u32, length: u32, expected_start_offset: usize, expected_sp_address_after_dma: u32, expected: [[u16; 8]; N]) -> Result<(), String> {
    // Create some test data. Use uncached memory to ensure the DMA engine can see it
    // without us having to flush any caches first
    let mut source_data: [[u16; 8]; 4] = [[0u16; 8]; 4];
    let source_data_uncached = MemoryMap::uncached_mut(source_data.as_mut_ptr());
    unsafe {
        source_data_uncached.add(0).write_volatile([0x0123, 0x4567, 0x89AB, 0xCDEF, 0xFEDC, 0x89BA, 0x7654, 0x3210]);
        source_data_uncached.add(1).write_volatile([0x1212, 0x3434, 0x4545, 0x5656, 0x6767, 0x7878, 0x8989, 0x9A9A]);
        source_data_uncached.add(2).write_volatile([0xA11A, 0xB11B, 0xC11C, 0xD11D, 0xE11E, 0xF11F, 0xF00F, 0xE00E]);
        source_data_uncached.add(3).write_volatile([0xD00D, 0xC00C, 0xB00B, 0xA00A, 0x9009, 0x8008, 0x7007, 0x6006]);
    }

    // Clear SPMEM
    for i in 0..(N * 4) {
        SPMEM::write(expected_start_offset + i * 4, 0xBADDECAF);
    }

    // DMA simple
    let source_ptr = unsafe { (source_data_uncached as *mut u8).add(source_index) };
    RSP::start_dma_cpu_to_sp(source_ptr, spmem_index, length);
    RSP::wait_until_dma_completed();

    // Ensure the data arrived as expected
    for i in 0..N {
        soft_assert_eq2(SPMEM::read_vector16_from_dmem_or_imem(expected_start_offset + i * 0x10), expected[i], || format!("SPMEM[0x{:x}] after DMA", expected_start_offset + i * 0x10))?;
    }

    soft_assert_eq2(RSP::sp_address(), expected_sp_address_after_dma, || "SP-Address after DMA".to_string())?;

    Ok(())
}

pub struct SPDMA0_8_7 {}

impl Test for SPDMA0_8_7 {
    fn name(&self) -> &str { "spmem: DMA RDRAM -> DMEM (all aligned)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dma_test(0, 8, 7, 0, 0x10,[
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0x0123, 0x4567, 0x89AB, 0xCDEF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF]])?;
        Ok(())
    }
}

pub struct SPDMA0_12_7D {}

impl Test for SPDMA0_12_7D {
    fn name(&self) -> &str { "spmem: DMA RDRAM -> DMEM (SP offset unaligned)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dma_test(0, 12, 7, 0, 0x10,[
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0x0123, 0x4567, 0x89AB, 0xCDEF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF]])?;
        Ok(())
    }
}

pub struct SPDMA0_12_7I {}

impl Test for SPDMA0_12_7I {
    fn name(&self) -> &str { "spmem: DMA RDRAM -> IMEM (SP offset unaligned)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dma_test(0, 0x100B, 7, 0x1000, 0x1010,[
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0x0123, 0x4567, 0x89AB, 0xCDEF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF]])?;
        Ok(())
    }
}

pub struct SPDMA4_8_7 {}

impl Test for SPDMA4_8_7 {
    fn name(&self) -> &str { "spmem: DMA RDRAM -> DMEM (RAM offset unaligned)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dma_test(4, 8, 7, 0, 0x10,[
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0x0123, 0x4567, 0x89AB, 0xCDEF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF]])?;
        Ok(())
    }
}

pub struct SPDMA0_8_11D {}

impl Test for SPDMA0_8_11D {
    fn name(&self) -> &str { "spmem: DMA RDRAM -> DMEM (length = 11)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dma_test(0, 8, 11, 0, 0x18,[
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0x0123, 0x4567, 0x89AB, 0xCDEF],
            [0xFEDC, 0x89BA, 0x7654, 0x3210, 0xBADD, 0xECAF, 0xBADD, 0xECAF]])?;
        Ok(())
    }
}

pub struct SPDMA0_8_11I {}

impl Test for SPDMA0_8_11I {
    fn name(&self) -> &str { "spmem: DMA RDRAM -> IMEM (length = 11)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dma_test(0, 0x1008, 11, 0x1000, 0x1018,[
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0x0123, 0x4567, 0x89AB, 0xCDEF],
            [0xFEDC, 0x89BA, 0x7654, 0x3210, 0xBADD, 0xECAF, 0xBADD, 0xECAF]])?;
        Ok(())
    }
}

pub struct SPDMAIntoDMEMUntilEnd {}

impl Test for SPDMAIntoDMEMUntilEnd {
    fn name(&self) -> &str { "spmem: DMA RDRAM -> DMEM (until the end)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // expected will read from both DMEM and IMEM. Ensure that nothing spilled into IMEM
        dma_test(0, 0xFF0, 15, 0xFE0, 0x0,[
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0x0123, 0x4567, 0x89AB, 0xCDEF, 0xFEDC, 0x89BA, 0x7654, 0x3210],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF]])?;

        Ok(())
    }
}

pub struct SPDMAIntoDMEMWithOverflow {}

impl Test for SPDMAIntoDMEMWithOverflow {
    fn name(&self) -> &str { "spmem: DMA RDRAM -> DMEM (overflow)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // expected will read from both DMEM and IMEM. Ensure that nothing spilled into IMEM
        dma_test(0, 0xFF0, 31, 0xFE0, 0x10,[
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0x0123, 0x4567, 0x89AB, 0xCDEF, 0xFEDC, 0x89BA, 0x7654, 0x3210],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF]])?;

        // But also ensure that DMEM properly overflowed
        soft_assert_eq2(SPMEM::read_vector16_from_dmem(0), [0x1212, 0x3434, 0x4545, 0x5656, 0x6767, 0x7878, 0x8989, 0x9A9A], || "SPMEM[0x0] after DMA".to_string())?;
        Ok(())
    }
}

pub struct SPDMAIntoIMEMUntilEnd {}

impl Test for SPDMAIntoIMEMUntilEnd {
    fn name(&self) -> &str { "spmem: DMA RDRAM -> IMEM (until the end)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // expected will read from both DMEM and IMEM. Ensure that nothing spilled into IMEM
        dma_test(0, 0x1FF0, 15, 0x1FE0, 0x1000,[
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0x0123, 0x4567, 0x89AB, 0xCDEF, 0xFEDC, 0x89BA, 0x7654, 0x3210],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF]])?;

        Ok(())
    }
}

pub struct SPDMAIntoIMEMWithOverflow {}

impl Test for SPDMAIntoIMEMWithOverflow {
    fn name(&self) -> &str { "spmem: DMA RDRAM -> IMEM (overflow)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // expected will read from both IMEM and DMEM. Ensure that nothing spilled into DMEM
        dma_test(0, 0x1FF0, 31, 0x1FE0, 0x1010,[
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0x0123, 0x4567, 0x89AB, 0xCDEF, 0xFEDC, 0x89BA, 0x7654, 0x3210],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF],
            [0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF, 0xBADD, 0xECAF]])?;

        // But also ensure that IMEM properly overflowed
        soft_assert_eq2(SPMEM::read_vector16_from_dmem_or_imem(0x1000), [0x1212, 0x3434, 0x4545, 0x5656, 0x6767, 0x7878, 0x8989, 0x9A9A], || "SPMEM[0x1000] after DMA".to_string())?;
        Ok(())
    }
}
