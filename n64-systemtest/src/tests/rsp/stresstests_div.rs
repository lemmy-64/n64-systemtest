use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use num_traits::ToPrimitive;

use crate::VIDEO;
use crate::graphics::color::Color;
use crate::graphics::color::RGBA5551;
use crate::graphics::cursor::Cursor;
use crate::graphics::font::Font;
use crate::graphics::system_font::FONT_GENEVA_9;
use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{E, Element, GPR, RSPAssembler, VR};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::rsp::op_vmov_vrcp::{rcp, rsq};
use crate::tests::soft_asserts::soft_assert_eq2;

fn run_stress_test<FEmitter: Fn(&mut RSPAssembler, VR), FSimulator: Fn(u32) -> u32>(name: &str, emit: FEmitter, simulate: FSimulator) -> Result<(), String> {
    let mut assembler = RSPAssembler::new(0);
    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    emit(&mut assembler, VR::V0);
    assembler.write_sqv(VR::V0, E::_0, 0x100, GPR::R0);
    assembler.write_break();

    let font = Font::from_data(&FONT_GENEVA_9).unwrap();
    let mut cursor = Cursor::new_with_font(&font, RGBA5551::BLACK);
    for input_value in 0x0000_0000..=0xFFFF_FFFF {
        if (input_value & 0xFFFFF) == 0 {
            let v = VIDEO.lock();
            {
                let mut lock = v.framebuffers().backbuffer().lock();
                let buffer = lock.as_mut().unwrap();
                buffer.clear_with_color(RGBA5551::WHITE);

                cursor.x = 16;
                cursor.y = 16;
                cursor.draw_text(buffer, format!("Stress testing {}. {:3.2}% complete (at 0x{:x})", name, input_value as f32 / 0xFFFF_FFFFu32.to_f32().unwrap() * 100.0f32, input_value).as_str());
            }
            v.swap_buffers();
        }

        SPMEM::write(0x0, input_value);

        RSP::start_running(0);
        let expected_value = simulate(input_value);
        RSP::wait_until_rsp_is_halted();

        let result = SPMEM::read(0x104);
        soft_assert_eq2(result, expected_value, || format!("Result for value 0x{:x}", input_value))?;
    }

    Ok(())
}

pub struct VRCP32 { }

impl Test for VRCP32 {
    fn name(&self) -> &str { "RSP VRCPL/VRCPH (Stress test)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_stress_test(
            "VRCPL/VRCPH",
            &|assembler: &mut RSPAssembler, reg| {
                assembler.write_vrcph(reg, reg, VR::V2, Element::_0);
                assembler.write_vrcpl(reg, reg, VR::V3, Element::_1);
                assembler.write_vrcph(reg, reg, VR::V2, Element::_0);
            },
            rcp,
        )?;

        Ok(())
    }
}

pub struct VRSQ32 { }

impl Test for VRSQ32 {
    fn name(&self) -> &str { "RSP VRSQL/VRSQH (Stress test)" }

    fn level(&self) -> Level { Level::StressTest }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_stress_test(
            "VRSQL/VRSQH",
            &|assembler: &mut RSPAssembler, reg| {
                assembler.write_vrsqh(reg, reg, VR::V2, Element::_0);
                assembler.write_vrsql(reg, reg, VR::V3, Element::_1);
                assembler.write_vrsqh(reg, reg, VR::V2, Element::_0);
            },
            rsq,
        )?;

        Ok(())
    }
}

