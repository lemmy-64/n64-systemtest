use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::{MemoryMap, VIDEO};
use crate::graphics::color::Color;
use crate::graphics::color::RGBA1555;
use crate::graphics::cursor::Cursor;
use crate::graphics::font::Font;
use crate::graphics::system_font::FONT_GENEVA_9;
use crate::math::vector::Vector;
use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{CP0Register, E, Element, GPR, RSPAssembler, VR};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq2;

fn zero_extend_accum48(v: i64) -> u64 { ((v as u64) << 16) >> 16 }

fn sign_extend_accum48(v: u64) -> i64 { ((v as i64) << 16) >> 16 }

#[inline(always)]
fn run_stress_test<F: Fn(&mut RSPAssembler) -> (), F2: Fn(u16, u16, u64) -> (u16, u64)>(name: &str, assembly_maker: F, cpu_computer: F2) -> Result<(), String> {
    // This runs every combination of vector values on the rsp, batched in STEPS_PER_RSP values at a time.
    // The RSP will always run one item ahead of the CPU to allow for some parallelism.
    // When the RSP is done, it will dma its results back into rsp_out_data (at index 0 for even and index 1 for odd jobs).
    // For non-accumulating instructions (e.g. VMULF) we can catch every single bit combination.
    // For accumulating instructions (e.g. VMACF), the accumulator will be reset at each batch but otherwise incremented;
    // while that doesn't catch all possible bit combinations, it should give us pretty good coverage.
    // An initial version of this ran for 8 hours, but it has seen quite some optimizations to
    // get this down to 1h20min (for VMULF). Unfortunately, these came at the expense of some readability.

    // In general, the current bottleneck is on the CPU side - if we were just running the RSP without
    // comparing on the CPU, we could run in 33min. So if we could move some things over to the RSP (e.g.
    // reordering rsp-accumulator into 64 bit words instead of 3*u16, we'd be saving some more work on the
    // CPU side. However, most of the work is probably multiplication anyway, so we shouldn't be expecting too much.

    // An alternative approach would be to not compare on the CPU at all but merely to record results in e.g.
    // a hash and run it later. However, that would make the test a lot less maintainable.

    const STEPS_PER_RSP: usize = 32;
    let mut rsp_out_data: [[[Vector; 4]; STEPS_PER_RSP]; 2] = Default::default();

    // Assemble RSP program
    let mut assembler = RSPAssembler::new(0);

    // Clear accumulator - it would be nice to just keep it running but it seems to be lost on BREAK
    assembler.write_vxor(VR::V0, VR::V0, VR::V0, Element::All);
    assembler.write_vmudn(VR::V0, VR::V0, VR::V0, Element::All);

    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V2, E::_0, 0x020, GPR::R0);

    // GRPs:
    // - S0: Target address to write output to
    // - S1: Decrementing loop counter
    // - S2: Current second vector register value (load on start and incremented by one in each iteration)

    assembler.write_li(GPR::S0, 0x40);
    assembler.write_li(GPR::S1, STEPS_PER_RSP as u32);
    assembler.write_lhu(GPR::S2, GPR::R0, 0x10);

    // Duplicate value and write through four SW
    assembler.write_sll(GPR::A0, GPR::S2, 16);
    assembler.write_or(GPR::A0, GPR::A0, GPR::S2);
    for i in 0..4 {
        assembler.write_sw(GPR::A0, GPR::R0, 0x10 + (i << 2));
    }

    let loop_beginning = assembler.get_jump_target();
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);

    assembly_maker(&mut assembler);

    assembler.write_vsar_any_index(VR::V4, VR::V0, VR::V0, E::_8);
    assembler.write_vsar_any_index(VR::V5, VR::V0, VR::V0, E::_9);
    assembler.write_vsar_any_index(VR::V6, VR::V0, VR::V0, E::_10);

    assembler.write_sqv(VR::V3, E::_0, 0x00, GPR::S0);
    assembler.write_sqv(VR::V4, E::_0, 0x10, GPR::S0);
    assembler.write_sqv(VR::V5, E::_0, 0x20, GPR::S0);
    assembler.write_sqv(VR::V6, E::_0, 0x30, GPR::S0);

    // Increment second vector register and write back. we can't use VADD as that clears the accumulator
    assembler.write_addiu(GPR::S2, GPR::S2, 1);
    assembler.write_sll(GPR::A0, GPR::S2, 16);
    assembler.write_or(GPR::A0, GPR::A0, GPR::S2);
    for i in 0..4 {
        assembler.write_sw(GPR::A0, GPR::R0, 0x10 + (i << 2));
    }
    assembler.write_addiu(GPR::S1, GPR::S1, -1);
    assembler.write_bgtz_backwards(GPR::S1, &loop_beginning);
    assembler.write_addiu(GPR::S0, GPR::S0, 0x40);  // delay slot

    // Initiate DMA back to CPU
    assembler.write_ori(GPR::A1, GPR::R0, 0x40);
    assembler.write_mtc0(CP0Register::SPAddress, GPR::A1);
    assembler.write_lw(GPR::A1, GPR::R0, 0x20);
    //assembler.write_li(GPR::A1, (&mut rsp_out_data).as_mut_ptr() as u32);
    assembler.write_mtc0(CP0Register::DRAMAddress, GPR::A1);
    assembler.write_li(GPR::A1, (STEPS_PER_RSP * 4 * 16) as u32 - 1);
    assembler.write_mtc0(CP0Register::WriteLength, GPR::A1);

    assembler.write_break();

    let font = Font::from_data(&FONT_GENEVA_9).unwrap();
    let mut cursor = Cursor::new_with_font(&font, RGBA1555::BLACK);
    for a_base in (0..=0xFFFF).step_by(8) {
        {
            let v = VIDEO.lock();
            {
                let mut lock = v.framebuffers().backbuffer().lock();
                let buffer = lock.as_mut().unwrap();
                buffer.clear_with_color(RGBA1555::WHITE);

                cursor.x = 16;
                cursor.y = 16;
                cursor.draw_text(buffer, format!("Stress testing {}. {:3.2}% complete", name, a_base as f32 / 65536.0f32 * 100.0f32).as_str());
            }
            v.swap_buffers();
        }

        let input1 = [a_base, a_base + 1, a_base + 2, a_base + 3, a_base + 4, a_base + 5, a_base + 6, a_base + 7];
        SPMEM::write_vector16_into_dmem(0x00, &input1);
        // The RSP code will duplicate this along lanes and increment itself
        SPMEM::write(0x10, 0);

        // Give the RSP a headstart of 1
        let mut rsp_out_current_index = 0;
        SPMEM::write(0x20, (&mut rsp_out_data[rsp_out_current_index]).as_mut_ptr() as u32);
        RSP::start_running(0);
        for b_base in (0..=0xFFFF).step_by(STEPS_PER_RSP) {
            // Wait until RSP is finished and get its results
            RSP::wait_until_rsp_is_halted_and_dma_completed();

            // Have the RSP run the next batch so that we can run the compare the current one
            let rsp_out_next_index = (rsp_out_current_index + 1) & 1;
            if (b_base as usize + STEPS_PER_RSP) <= 0xFFFF {
                SPMEM::write(0x20, (&mut rsp_out_data[rsp_out_next_index]).as_mut_ptr() as u32);
                RSP::start_running(0);
            }

            // The current accumulator of the simulated rsp. The upper 16 bits of each 64 value must be empty
            let mut cpu_accumulator: [u64; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

            // Compute on the CPU and compare results
            for b_offset in 0..STEPS_PER_RSP {
                let rsp_result = unsafe { MemoryMap::uncached(&rsp_out_data[rsp_out_current_index][b_offset][0]).read_volatile() };
                let rsp_acc_high = unsafe { MemoryMap::uncached(&rsp_out_data[rsp_out_current_index][b_offset][1]).read_volatile() };
                let rsp_acc_mid = unsafe { MemoryMap::uncached(&rsp_out_data[rsp_out_current_index][b_offset][2]).read_volatile() };
                let rsp_acc_low = unsafe { MemoryMap::uncached(&rsp_out_data[rsp_out_current_index][b_offset][3]).read_volatile() };

                let b = b_base + b_offset as u16;
                for i in 0..8 {
                    let (result_val, result_accum) = cpu_computer(a_base + i as u16, b, cpu_accumulator[i]);
                    cpu_accumulator[i] = result_accum;

                    let rsp_accum = ((rsp_acc_high.get16(i) as u64) << 32) | ((rsp_acc_mid.get16(i) as u64) << 16) | (rsp_acc_low.get16(i) as u64);
                    if (rsp_accum != result_accum) || (rsp_result.get16(i) != result_val) {
                        soft_assert_eq2(rsp_result.get16(i), result_val, || format!("Result vector for inputs 0x{:x} and 0x{:x}", (a_base as usize) as i16, b))?;
                        soft_assert_eq2(rsp_accum, result_accum, || format!("Result accumulator for inputs 0x{:x} and 0x{:x}", (a_base as usize) as i16, b))?;
                    }
                }
            }

            rsp_out_current_index = rsp_out_next_index;
        }
    }

    Ok(())
}

pub struct VMULF {}

impl Test for VMULF {
    fn name(&self) -> &str { "RSP VMULF (Stress test)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_stress_test("VMULF", |assembler| {
            assembler.write_vmulf(VR::V3, VR::V1, VR::V0, Element::All);
        }, |a, b, _accum| {
            let product = (a as i16 as i32) * (b as i16 as i32);
            let temp = ((product as i64) << 1) + 0x8000;
            let result_val = if (temp > 0) && ((temp & !0x7FFFFFFF) != 0) { 0x7FFF } else { (temp >> 16) as u16 };

            (result_val, zero_extend_accum48(temp))
        })?;
        Ok(())
    }
}

pub struct VMUDH {}

impl Test for VMUDH {
    fn name(&self) -> &str { "RSP VMUDH (Stress test)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_stress_test("VMUDH", |assembler| {
            assembler.write_vmudh(VR::V3, VR::V1, VR::V0, Element::All);
        }, |a, b, _accum| {
            let product = (a as i16 as i32) * (b as i16 as i32);

            let result_val =
                if product >= 0 {
                    if (product & !0x7FFF) != 0 { 0x7FFFu16 } else { product as u16 }
                } else {
                    if (!product & !0x7FFF) != 0 { 0x8000u16 } else { product as u16 }
                } as u16;

            (result_val, (product as u32 as u64) << 16)
        })?;
        Ok(())
    }
}

pub struct VMUDM {}

impl Test for VMUDM {
    fn name(&self) -> &str { "RSP VMUDM (Stress test)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_stress_test("VMUDM", |assembler| {
            assembler.write_vmudm(VR::V3, VR::V1, VR::V0, Element::All);
        }, |a, b, _accum| {
            let product = (a as i16 as i32) * (b as u16 as i32);

            ((product >> 16) as u16, zero_extend_accum48(product as i64))
        })?;
        Ok(())
    }
}

pub struct VMUDN {}

impl Test for VMUDN {
    fn name(&self) -> &str { "RSP VMUDN (Stress test)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_stress_test("VMUDN", |assembler| {
            assembler.write_vmudn(VR::V3, VR::V1, VR::V0, Element::All);
        }, |a, b, _accum| {
            let product = (a as u16 as i32) * (b as i16 as i32);
            let result_val = product as u16;

            (result_val, zero_extend_accum48(product as i64))
        })?;
        Ok(())
    }
}

pub struct VMACF {}

impl Test for VMACF {
    fn name(&self) -> &str { "RSP VMACF (Stress test)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_stress_test("VMACF", |assembler| {
            assembler.write_vmacf(VR::V3, VR::V1, VR::V0, Element::All);
        }, |a, b, accum| {
            let product = (a as i16 as i32) * (b as i16 as i32);

            let new_accum = ((product as i64) << 1) + sign_extend_accum48(accum);
            let temp_shifted32 = (new_accum >> 16) as i32;
            let result_val =
                if temp_shifted32 >= 0 {
                    if (temp_shifted32 & !0x7FFF) != 0 { 0x7FFFu16 } else { temp_shifted32 as u16 }
                } else {
                    if (!temp_shifted32 & !0x7FFF) != 0 { 0x8000u16 } else { temp_shifted32 as u16 }
                } as u16;

            (result_val, zero_extend_accum48(new_accum))
        })?;
        Ok(())
    }
}

pub struct VMADH {}

impl Test for VMADH {
    fn name(&self) -> &str { "RSP VMADH (Stress test)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_stress_test("VMADH", |assembler| {
            assembler.write_vmadh(VR::V3, VR::V1, VR::V0, Element::All);
        }, |a, b, accum| {
            let product = (a as i16 as i32) * (b as i16 as i32);

            let new_accum = ((product as i64) << 16) + sign_extend_accum48(accum);
            let temp_shifted32 = (new_accum >> 16) as i32;
            let result_val =
                if temp_shifted32 >= 0 {
                    if (temp_shifted32 & !0x7FFF) != 0 { 0x7FFFu16 } else { temp_shifted32 as u16 }
                } else {
                    if (!temp_shifted32 & !0x7FFF) != 0 { 0x8000u16 } else { temp_shifted32 as u16 }
                } as u16;

            (result_val, zero_extend_accum48(new_accum))
        })?;
        Ok(())
    }
}

pub struct VMADM {}

impl Test for VMADM {
    fn name(&self) -> &str { "RSP VMADM (Stress test)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_stress_test("VMADM", |assembler| {
            assembler.write_vmadm(VR::V3, VR::V1, VR::V0, Element::All);
        }, |a, b, accum| {
            let product = (a as i16 as i32) * (b as u16 as i32);
            let new_accum = (product as i64) + sign_extend_accum48(accum);
            let product_shifted = new_accum >> 16;

            let result_val =
                if product_shifted >= 0 {
                    if (product_shifted & !0x7FFF) != 0 { 0x7FFFu16 } else { product_shifted as u16 }
                } else {
                    if (!product_shifted & !0x7FFF) != 0 { 0x8000u16 } else { product_shifted as u16 }
                } as u16;

            (result_val, zero_extend_accum48(new_accum))
        })?;
        Ok(())
    }
}

pub struct VMADN {}

impl Test for VMADN {
    fn name(&self) -> &str { "RSP VMADN (Stress test)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_stress_test("VMADN", |assembler| {
            assembler.write_vmadn(VR::V3, VR::V1, VR::V0, Element::All);
        }, |a, b, accum| {
            let product = (a as u16 as i32) * (b as i16 as i32);

            let new_accum = (product as i64) + sign_extend_accum48(accum);
            let result_val =
                if new_accum >= 0 {
                    if (new_accum & !0x7FFFFFFF) != 0 { 0xffff } else { new_accum as u16 }
                } else {
                    if (!new_accum & !0x7FFFFFFF) != 0 { 0 } else { new_accum as u16 }
                } as u16;

            (result_val, zero_extend_accum48(new_accum))
        })?;
        Ok(())
    }
}
