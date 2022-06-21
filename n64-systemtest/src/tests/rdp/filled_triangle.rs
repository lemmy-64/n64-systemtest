use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::mem::size_of;
use oorandom::Rand32;

use crate::graphics::color::{Color, RGBA8888};
use crate::rdp::fixedpoint::{I12_2, I16_16, U10_2};
use crate::rdp::modes::{A, B, Blender, CoverageMode, CycleType, Format, Othermode, PixelSize, PM};
use crate::rdp::rdp::RDP;
use crate::rdp::rdp_assembler::{RDPAssembler, RDPRectangle, TriangleBase};
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq_2d_array;
use crate::uncached_memory::UncachedHeapMemory;

fn render_on_cpu<T: Color + Copy + Clone + From<RGBA8888>, const WIDTH: usize, const HEIGHT: usize>(base: &TriangleBase, scissor: &RDPRectangle, color32: RGBA8888, coverage_mode: CoverageMode) -> [[T; WIDTH]; HEIGHT] {
    let color: T = color32.into();
    let mut result: [[T; WIDTH]; HEIGHT] = [[T::BLACK; WIDTH]; HEIGHT];
    let mut subpixel_coverage: [[u8; WIDTH]; HEIGHT] = [[0; WIDTH]; HEIGHT];

    let is_right_major = base.is_right_major();

    let yl: i32 = base.yl().raw_value();
    let ym: i32 = base.ym().raw_value();
    let yh: i32 = base.yh().raw_value();

    // TODO: Is 64 bit precision correct here or should it be i32 only?
    let xl: i64 = (base.xl().raw_value() as i64) << 2;
    let xm: i64 = (base.xm().raw_value() as i64) << 2;
    let xh: i64 = (base.xh().raw_value() as i64) << 2;

    let dl: i64 = base.dl().raw_value() as i64;
    let dm: i64 = base.dm().raw_value() as i64;
    let dh: i64 = base.dh().raw_value() as i64;

    let mut major_x = xh;
    let major_inc = dh;
    let mut y = yh;

    let sections = [(ym, xm, dm), (yl, xl, dl)];
    for (y_target, mut minor_x, minor_inc) in sections {
        while y < y_target {
            // Top scissor is "just" pixel accurate
            if (y >> 2) >= (((scissor.top().raw_value() as i32) + 3) >> 2) {
                if y >= scissor.bottom().raw_value() as i32 {
                    break;
                }

                let (left, right) = if is_right_major { (major_x, minor_x) } else { (minor_x, major_x) };

                if right >= left {
                    let subpixel_left = left >> 16;
                    let subpixel_right = (right - (2 << 2)) >> 16;
                    for x in subpixel_left..=subpixel_right {
                        // Left scissor is "just" pixel accurate
                        if (x >> 2) < (((scissor.left().raw_value() as i64) + 3) >> 2) {
                            continue;
                        }

                        // Right scissor is subpixel accurate
                        if x >= scissor.right().raw_value() as i64 {
                            break;
                        }
                        let y_pixel = y as usize >> 2;
                        let x_pixel = x as usize >> 2;
                        if y_pixel < HEIGHT && x_pixel < WIDTH {
                            subpixel_coverage[y_pixel][x_pixel] += 1;
                        }
                    }
                }
            }

            let next_y = y + 1;
            let next_major_x = major_x + major_inc;
            let next_minor_x = minor_x + minor_inc;

            minor_x = next_minor_x;
            major_x = next_major_x;
            y = next_y;
        }
    }

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let coverage = subpixel_coverage[y][x];
            if coverage != 0 {
                let alpha = match coverage_mode {
                    CoverageMode::Clamp => match coverage {
                        4 => 0x20,
                        7 => 0x40,
                        8 => 0x60,
                        9 => 0x60,
                        10 => 0x60,
                        12 => 0xA0,
                        16 => 0xE0,
                        _ => coverage,  // placeholder until all are filled out
                    },
                    CoverageMode::Zap => 0xE0,
                    CoverageMode::Wrap => 0xE0,
                    CoverageMode::Save => subpixel_coverage[y][x],
                };
                result[y as usize][x as usize] = color.with_alpha(alpha);
            }
        }
    }

    result
}

fn render_on_rdp<T: Color + Copy + Clone, const WIDTH: usize, const HEIGHT: usize>(triangle: &TriangleBase, scissor: &RDPRectangle, color: RGBA8888, coverage_mode: CoverageMode) -> [[T; WIDTH]; HEIGHT] {
    let mut framebuffer = UncachedHeapMemory::<T>::new_with_init_value(WIDTH * HEIGHT, T::BLACK);

    let mut assembler = RDPAssembler::new();

    let image_size = match size_of::<T>() {
        2 => PixelSize::Bits16,
        4 => PixelSize::Bits32,
        _ => panic!("Unhandled color format"),
    };
    assembler.set_framebuffer_image(Format::RGBA, image_size, WIDTH - 1, &mut framebuffer);
    assembler.set_scissor(scissor);

    // Clear everything
    let clear_rect = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_u32(WIDTH as u32 - 1), U10_2::from_u32(HEIGHT as u32 - 1));
    assembler.set_othermode(Othermode::new()
        .with_cycle_type(CycleType::Fill));
    assembler.set_fillcolor32(RGBA8888::BLACK);
    assembler.filled_rectangle(&clear_rect);
    assembler.sync_pipe();

    // Draw triangle
    assembler.set_framebuffer_image(Format::RGBA, image_size, WIDTH - 1, &mut framebuffer);
    assembler.set_othermode(Othermode::new()
        .with_cycle_type(CycleType::SingleCycle)
        .with_coverage_mode(coverage_mode)
        .with_blender_0(Blender::new(A::CombineAlpha, PM::BlendColor, B::Zero, PM::MemoryColor)));
    assembler.set_blendcolor(color);
    assembler.filled_triangle(triangle);
    assembler.sync_pipe();
    assembler.sync_full();

    RDP::run_and_wait(&mut assembler);

    // Copy into non-uncached array.
    let mut result: [[T; WIDTH]; HEIGHT] = [[T::BLACK; WIDTH]; HEIGHT];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            result[y][x] = framebuffer.read((y * WIDTH + x) as usize);
        }
    }

    result
}

pub struct FilledTriangle1CycleDegenerateRect {}

impl Test for FilledTriangle1CycleDegenerateRect {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (degenerate as rectangle)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let scissor = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_usize(8), U10_2::from_usize(8));
        let color = RGBA8888::RED.with_alpha(255);
        let coverage_mode = CoverageMode::Zap;
        let triangle = TriangleBase::new(
            true, 0, 0,
            I12_2::from_i32(5),
            I12_2::from_i32(3),
            I12_2::from_i32(1),
            I16_16::new_with_masked_value(3 << 16),
            I16_16::new_with_masked_value(5 << 16),
            I16_16::from_i32(1),
            I16_16::from_i32(0),
            I16_16::from_i32(0),
            I16_16::from_i32(0),
        );

        let actual = render_on_rdp::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);
        let expected = render_on_cpu::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);

        soft_assert_eq_2d_array(actual, expected, || format!("Rendered pixels"))?;

        Ok(())
    }
}

/// A basic left-major triangle
pub struct FilledTriangle1CycleRightMajorFlatTop {}

impl Test for FilledTriangle1CycleRightMajorFlatTop {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (right major with flat top)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let scissor = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_usize(8), U10_2::from_usize(8));
        let color = RGBA8888::RED;
        let coverage_mode = CoverageMode::Zap;
        let triangle = TriangleBase::new(
            true, 0, 0,
            I12_2::from_i32(5),
            I12_2::from_i32(1),
            I12_2::from_i32(1),
            I16_16::from_i32(5),
            I16_16::from_i32(5),
            I16_16::from_i32(1),
            I16_16::from_i32(-1),
            I16_16::from_i32(0),
            I16_16::from_i32(0),
        );

        let actual = render_on_rdp::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);
        let expected = render_on_cpu::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);

        soft_assert_eq_2d_array(actual, expected, || format!("Rendered pixels"))?;

        Ok(())
    }
}

/// A basic left-major triangle
pub struct FilledTriangle1CycleRightMajor {}

impl Test for FilledTriangle1CycleRightMajor {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (right major)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // TODO: is anything ever shown here?
        let scissor = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_usize(8), U10_2::from_usize(8));
        let color = RGBA8888::RED;
        let coverage_mode = CoverageMode::Zap;
        let triangle = TriangleBase::new(
            true, 0, 0,
            I12_2::from_i32(5),
            I12_2::from_i32(1),
            I12_2::from_i32(1),
            I16_16::from_i32(4),
            I16_16::from_i32(4),
            I16_16::from_i32(6),
            I16_16::from_i32(0),
            I16_16::from_i32(-1),
            I16_16::from_i32(0),
        );

        let actual = render_on_rdp::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);
        let expected = render_on_cpu::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);

        soft_assert_eq_2d_array(actual, expected, || format!("Rendered pixels"))?;

        Ok(())
    }
}

/// Triangle that is being cut by a scissor rect
/// Subpixel accuracy is ignored on the left
pub struct FilledTriangle1CycleScissorLeft {}

impl Test for FilledTriangle1CycleScissorLeft {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (scissor left)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for left_scissor_raw_value in 0..12 {
            let left_scissor = U10_2::new_with_masked_value(left_scissor_raw_value);
            let scissor = RDPRectangle::new(left_scissor, U10_2::from_u32(0), U10_2::from_usize(8), U10_2::from_usize(8));
            let color = RGBA8888::RED;
            let coverage_mode = CoverageMode::Zap;
            let triangle = TriangleBase::new(
                true, 0, 0,
                I12_2::from_i32(5),
                I12_2::from_i32(1),
                I12_2::from_i32(1),
                I16_16::from_i32(5),
                I16_16::from_i32(5),
                I16_16::from_i32(1),
                I16_16::from_i32(-1),
                I16_16::from_i32(0),
                I16_16::from_i32(0),
            );

            let actual = render_on_rdp::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);
            let expected = render_on_cpu::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);

            soft_assert_eq_2d_array(actual, expected, || format!("Rendered pixels with left scissor={:?}", left_scissor))?;
        }

        Ok(())
    }
}

/// Triangle that is being cut by a scissor rect (with subpixel precision)
pub struct FilledTriangle1CycleScissorTop {}

impl Test for FilledTriangle1CycleScissorTop {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (scissor top)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for top_scissor_raw_value in 0..12 {
            let top_scissor = U10_2::new_with_masked_value(top_scissor_raw_value);
            let scissor = RDPRectangle::new(U10_2::from_u32(0), top_scissor, U10_2::from_usize(8), U10_2::from_usize(8));
            let color = RGBA8888::RED;
            let coverage_mode = CoverageMode::Zap;
            let triangle = TriangleBase::new(
                true, 0, 0,
                I12_2::from_i32(5),
                I12_2::from_i32(0),
                I12_2::from_i32(0),
                I16_16::from_i32(5),
                I16_16::from_i32(5),
                I16_16::from_i32(1),
                I16_16::from_i32(-1),
                I16_16::from_i32(0),
                I16_16::from_i32(0),
            );

            let actual = render_on_rdp::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);
            let expected = render_on_cpu::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);

            soft_assert_eq_2d_array(actual, expected, || format!("Rendered pixels with top scissor={:?}", top_scissor))?;
        }

        Ok(())
    }
}

fn test_right_scissor(step_by: usize) -> Result<(), String> {
    for right_scissor_raw_value in (0..32).step_by(step_by) {
        let right_scissor = U10_2::new_with_masked_value(right_scissor_raw_value);
        let scissor = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), right_scissor, U10_2::from_usize(8));
        let color = RGBA8888::RED;
        let coverage_mode = CoverageMode::Zap;
        let triangle = TriangleBase::new(
            true, 0, 0,
            I12_2::from_i32(5),
            I12_2::from_i32(1),
            I12_2::from_i32(1),
            I16_16::from_i32(5),
            I16_16::from_i32(5),
            I16_16::from_i32(1),
            I16_16::from_i32(-1),
            I16_16::from_i32(0),
            I16_16::from_i32(0),
        );

        let actual = render_on_rdp::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);
        let expected = render_on_cpu::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);

        soft_assert_eq_2d_array(actual, expected, || format!("Rendered pixels with right scissor={:?}", right_scissor))?;
    }
    Ok(())
}

/// Triangle that is being cut by a scissor rect (with subpixel precision)
pub struct FilledTriangle1CycleScissorRight {}

impl Test for FilledTriangle1CycleScissorRight {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (scissor right)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_right_scissor(4)
    }
}

/// Triangle that is being cut by a scissor rect (with subpixel precision)
pub struct FilledTriangle1CycleScissorRightSubPixelPrecision {}

impl Test for FilledTriangle1CycleScissorRightSubPixelPrecision {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (scissor right with subpixel precision)" }

    fn level(&self) -> Level { Level::RDPPrecise }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_right_scissor(1)
    }
}

fn test_bottom_scissor(step_by: usize) -> Result<(), String> {
    for bottom_scissor_raw_value in (0..32).step_by(step_by) {
        let bottom_scissor = U10_2::new_with_masked_value(bottom_scissor_raw_value);
        let scissor = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_usize(8), bottom_scissor);
        let color = RGBA8888::RED;
        let coverage_mode = CoverageMode::Zap;
        let triangle = TriangleBase::new(
            true, 0, 0,
            I12_2::from_i32(5),
            I12_2::from_i32(1),
            I12_2::from_i32(1),
            I16_16::from_i32(5),
            I16_16::from_i32(5),
            I16_16::from_i32(1),
            I16_16::from_i32(-1),
            I16_16::from_i32(0),
            I16_16::from_i32(0),
        );

        let actual = render_on_rdp::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);
        let expected = render_on_cpu::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);

        soft_assert_eq_2d_array(actual, expected, || format!("Rendered pixels with bottom scissor={:?}", bottom_scissor))?;
    }
    Ok(())
}

/// Triangle that is being cut by a scissor rect
pub struct FilledTriangle1CycleScissorBottom {}

impl Test for FilledTriangle1CycleScissorBottom {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (scissor bottom)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_bottom_scissor(4)
    }
}

/// Triangle that is being cut by a scissor rect
pub struct FilledTriangle1CycleScissorBottomSubPixelPrecision {}

impl Test for FilledTriangle1CycleScissorBottomSubPixelPrecision {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (scissor bottom with subpixel precision)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        test_bottom_scissor(1)
    }
}

pub struct FilledTriangle1CycleNegativeYH {}

impl Test for FilledTriangle1CycleNegativeYH {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (with negative Y)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let scissor = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_usize(8), U10_2::from_usize(8));
        let color = RGBA8888::RED.with_alpha(255);
        let coverage_mode = CoverageMode::Zap;
        let triangle = TriangleBase::new(
            true, 0, 0,
            I12_2::from_i32(5),
            I12_2::from_i32(3),
            I12_2::from_i32(-2),
            I16_16::new_with_masked_value(3 << 16),
            I16_16::new_with_masked_value(5 << 16),
            I16_16::from_i32(1),
            I16_16::from_i32(0),
            I16_16::from_i32(0),
            I16_16::from_i32(0),
        );

        let actual = render_on_rdp::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);
        let expected = render_on_cpu::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);

        soft_assert_eq_2d_array(actual, expected, || format!("Rendered pixels"))?;

        Ok(())
    }
}

pub struct FilledTriangle1CycleNegativeXL {}

impl Test for FilledTriangle1CycleNegativeXL {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (negative X)" }

    fn level(&self) -> Level { Level::RDPBasic }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let scissor = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_usize(8), U10_2::from_usize(8));
        let color = RGBA8888::BLUE;
        let coverage_mode = CoverageMode::Zap;
        let triangle = TriangleBase::new(
            false, 0, 0,
            I12_2::from_i32(7),
            I12_2::from_i32(7),
            I12_2::from_i32(2),
            I16_16::from_i32(-1),
            I16_16::from_i32(-1),
            I16_16::from_i32(4),
            I16_16::from_i32(0),
            I16_16::from_i32(1),
            I16_16::from_i32(0),
        );

        let actual = render_on_rdp::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);
        let expected = render_on_cpu::<RGBA8888, 8, 8>(&triangle, &scissor, RGBA8888::BLUE, coverage_mode);

        soft_assert_eq_2d_array(actual, expected, || format!("Rendered pixels"))?;

        Ok(())
    }
}

pub struct FilledTriangle1CycleRandomized {}

impl Test for FilledTriangle1CycleRandomized {
    fn name(&self) -> &str { "RDP FilledTriangle 1 Cycle (randomized)" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut random = Rand32::new(0);
        for _ in 0..100 {
            let scissor = RDPRectangle::new(U10_2::from_u32(0), U10_2::from_u32(0), U10_2::from_usize(8), U10_2::from_usize(8));
            let color = RGBA8888::BLUE;
            let coverage_mode = CoverageMode::Zap;
            // For very large y range, weird effects happen that aren't understood. Don't include them in the random set yet
            let y_min = -2i32 << 2;
            let y_max = 16i32 << 2;
            let y_range = (y_max - y_min) as u32;

            let yh = I12_2::new_with_raw_value(y_min + (random.rand_range(0..y_range) as i32));
            let ym = I12_2::new_with_raw_value(y_min + (random.rand_range(0..y_range) as i32));
            let yl = I12_2::new_with_raw_value(y_min + (random.rand_range(0..y_range) as i32));

            let x_min = -2i32 << 16;
            let x_max = 16i32 << 16;
            let x_range = (x_max - x_min) as u32;

            let xh = I16_16::new_with_raw_value(x_min + (random.rand_range(0..x_range) as i32));
            let xm = I16_16::new_with_raw_value(x_min + (random.rand_range(0..x_range) as i32));
            let xl = I16_16::new_with_raw_value(x_min + (random.rand_range(0..x_range) as i32));
            let triangle = TriangleBase::new(
                (random.rand_u32() & 1) != 0, 0, 0,
                yl, ym, yh,
                xl, xm, xh,
                I16_16::from_i32(0),
                I16_16::from_i32(0),
                I16_16::from_i32(0),
            );

            let actual = render_on_rdp::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);
            let expected = render_on_cpu::<RGBA8888, 8, 8>(&triangle, &scissor, color, coverage_mode);

            soft_assert_eq_2d_array(actual, expected, || format!("Rendered pixels for {:?}", triangle))?;
        }

        Ok(())
    }
}

