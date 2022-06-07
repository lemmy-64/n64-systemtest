use crate::math::bits::Bitmasks32;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum CycleType {
    SingleCycle = 0,
    DualCycle = 1,
    Copy = 2,
    Fill = 3,
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum CoverageMode {
    Clamp = 0,
    Wrap = 1,
    Zap = 2,
    Save = 3,
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Format {
    RGBA = 0,
    YUV = 1,
    CI = 2,
    IA = 3,
    I = 4
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum PixelSize {
    Bits4 = 0,
    Bits8 = 1,
    Bits16 = 2,
    Bits32 = 3,
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum PM {
    CombineColor = 0,
    MemoryColor = 1,
    BlendColor = 2,
    FogColor = 3
}

#[repr(u8)]
#[allow(dead_code)]
#[derive()]
#[derive(Copy, Clone)]
pub enum A {
    CombineAlpha = 0,
    FogAlpha = 1,
    ShadeAlpha = 2,
    Zero = 3,
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum B {
    InverseA = 0,
    MemoryAlpha = 1,
    One = 2,
    Zero = 3
}

pub struct Othermode {
    raw_value: u64
}

impl Othermode {
    pub const fn new() -> Self { Self { raw_value: 0 } }

    pub const fn raw_value(&self) -> u64 { self.raw_value }

    pub const fn with_cycle_type(&self, value: CycleType) -> Self {
        const SHIFT: u32 = 52;
        const MASK: u64 = !((Bitmasks32::M2 as u64) << SHIFT);
        Self {
            raw_value: self.raw_value & MASK | ((value as u64) << SHIFT),
        }
    }

    pub const fn with_coverage_mode(&self, value: CoverageMode) -> Self {
        const SHIFT: u32 = 8;
        const MASK: u64 = !((Bitmasks32::M2 as u64) << SHIFT);
        Self {
            raw_value: self.raw_value & MASK | ((value as u64) << SHIFT),
        }
    }

    const fn with_blender<const EXTRA_SHIFT: usize>(&self, value: Blender) -> Self {
        let mask: u64 = !(0b11_0011_0011_0011u64 << (16 + EXTRA_SHIFT));
        let value = (((value.b as u64) << 16) | ((value.m as u64) << 20) | ((value.a as u64) << 24) | ((value.p as u64) << 28)) << EXTRA_SHIFT;
        Self {
            raw_value: self.raw_value & mask | value,
        }
    }

    pub const fn with_blender_0(&self, value: Blender) -> Self { self.with_blender::<2>(value) }

    #[allow(dead_code)]
    pub const fn with_blender_1(&self, value: Blender) -> Self { self.with_blender::<0>(value) }
}

pub struct Blender {
    a: A,
    p: PM,
    b: B,
    m: PM,
}

impl Blender {
    pub fn new(a: A, p: PM, b: B, m: PM) -> Self { Self { a, p, b, m } }
}