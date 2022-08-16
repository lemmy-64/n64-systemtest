use bitbybit::{bitenum, bitfield};

#[bitenum(u2, exhaustive: true)]
#[allow(dead_code)]
pub enum CycleType {
    SingleCycle = 0,
    DualCycle = 1,
    Copy = 2,
    Fill = 3,
}

#[bitenum(u2, exhaustive: true)]
#[allow(dead_code)]
pub enum CoverageMode {
    Clamp = 0,
    Wrap = 1,
    Zap = 2,
    Save = 3,
}

#[bitenum(u3, exhaustive: false)]
#[allow(dead_code)]
pub enum Format {
    RGBA = 0,
    YUV = 1,
    CI = 2,
    IA = 3,
    I = 4
}

#[bitenum(u2, exhaustive: true)]
#[allow(dead_code)]
pub enum PixelSize {
    Bits4 = 0,
    Bits8 = 1,
    Bits16 = 2,
    Bits32 = 3,
}

#[bitenum(u2, exhaustive: true)]
#[allow(dead_code)]
pub enum PM {
    CombineColor = 0,
    MemoryColor = 1,
    BlendColor = 2,
    FogColor = 3
}

#[bitenum(u2, exhaustive: true)]
#[allow(dead_code)]
pub enum A {
    CombineAlpha = 0,
    FogAlpha = 1,
    ShadeAlpha = 2,
    Zero = 3,
}

#[bitenum(u2, exhaustive: true)]
#[allow(dead_code)]
pub enum B {
    InverseA = 0,
    MemoryAlpha = 1,
    One = 2,
    Zero = 3
}

#[bitfield(u64, default: 0)]
pub struct Othermode {
    #[bits(52..=53, rw)]
    cycle_type: CycleType,

    #[bits(30..=31, rw)]
    blender_0p: PM,

    #[bits(28..=29, rw)]
    blender_1p: PM,

    #[bits(26..=27, rw)]
    blender_0a: A,

    #[bits(24..=25, rw)]
    blender_1a: A,

    #[bits(22..=23, rw)]
    blender_0m: PM,

    #[bits(20..=21, rw)]
    blender_1m: PM,

    #[bits(18..=19, rw)]
    blender_0b: B,

    #[bits(16..=17, rw)]
    blender_1b: B,

    #[bits(8..=9, rw)]
    coverage_mode: CoverageMode,

}

impl Othermode {
    pub const fn with_blender_0(&self, value: Blender) -> Self {
        self
            .with_blender_0p(value.p)
            .with_blender_0a(value.a)
            .with_blender_0m(value.m)
            .with_blender_0b(value.b)
    }

    #[allow(dead_code)]
    pub const fn with_blender_1(&self, value: Blender) -> Self {
        self
            .with_blender_1p(value.p)
            .with_blender_1a(value.a)
            .with_blender_1m(value.m)
            .with_blender_1b(value.b)
    }
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