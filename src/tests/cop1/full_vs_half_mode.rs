use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::arch::asm;
use core::any::Any;
use crate::cop0::{RegisterIndex, set_status, Status};
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::soft_assert_eq;

fn set_mode(full: bool) {
    unsafe { set_status(Status::DEFAULT.with_fpu64(full)); }
}

// The next four assembly wrappers probably aren't fully legal as they assume that
// the compiler leaves the FPU registers alone; I don't think there's such a guarantee but in practice
// it's fine.

fn mtc1<const REG: usize>(value: u32) {
    unsafe {
        asm!("
            .set noat
            .set noreorder
            mtc1 {value}, ${cop1Reg}
        ", value = in(reg) value, cop1Reg = const REG)}
}
fn mfc1<const REG: usize>() -> u32 {
    let result: u32;
    unsafe {
        asm!("
            .set noat
            .set noreorder
            mfc1 {value}, ${cop1Reg}
        ", value = out(reg) result, cop1Reg = const REG)
    }
    result
}
fn dmtc1<const REG: usize>(value: u64) {
    unsafe {
        asm!("
            .set noat
            .set noreorder
            dsll32 {lower}, {lower}, 0
            dsrl32 {lower}, {lower}, 0
            dsll32 {upper}, {upper}, 0
            or {lower}, {lower}, {upper}
            dmtc1 {lower}, ${cop1Reg}
        ", lower = inout(reg) (value as u32) => _, upper = inout(reg) ((value >> 32) as u32) => _, cop1Reg = const REG)
    }
}
fn dmfc1<const REG: usize>() -> u64 {
    let lower: u32;
    let upper: u32;
    unsafe {
        asm!("
            dmfc1 {lower}, ${cop1Reg}
            dsrl32 {upper}, {lower}, 0
            dsll32 {lower}, 0
            dsrl32 {lower}, 0
        ", lower = out(reg) lower, upper = out(reg) upper, cop1Reg = const REG)
    }
    ((upper as u64) << 32) | (lower as u64)
}

pub struct FullMode;

impl Test for FullMode {
    fn name(&self) -> &str { "Move To/From in Full Mode" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);

        // Read back as 64 bit values (this is just 1:1 what was written)
        soft_assert_eq(dmfc1::<0>(), 0x00001111_22223333, "DMFC1 after DMTC1 (0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1 after DMTC1 (1)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1 after DMTC1 (2)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1 after DMTC1 (3)")?;
        soft_assert_eq(dmfc1::<4>(), 0x00110011_22332233, "DMFC1 after DMTC1 (4)")?;
        soft_assert_eq(dmfc1::<5>(), 0x44554455_66776677, "DMFC1 after DMTC1 (5)")?;
        soft_assert_eq(dmfc1::<6>(), 0x88998899_AABBAABB, "DMFC1 after DMTC1 (6)")?;
        soft_assert_eq(dmfc1::<7>(), 0xCCDDCCDD_EEFFEEFF, "DMFC1 after DMTC1 (7)")?;

        // Read back as 32 bit values in 64 bit mode (this is simply the lower 32 bit)
        soft_assert_eq(mfc1::<0>(), 0x22223333, "MFC1 after DMTC1 (0)")?;
        soft_assert_eq(mfc1::<1>(), 0x66667777, "MFC1 after DMTC1 (1)")?;
        soft_assert_eq(mfc1::<2>(), 0xAAAABBBB, "MFC1 after DMTC1 (2)")?;
        soft_assert_eq(mfc1::<3>(), 0xEEEEFFFF, "MFC1 after DMTC1 (3)")?;
        soft_assert_eq(mfc1::<4>(), 0x22332233, "MFC1 after DMTC1 (4)")?;
        soft_assert_eq(mfc1::<5>(), 0x66776677, "MFC1 after DMTC1 (5)")?;
        soft_assert_eq(mfc1::<6>(), 0xAABBAABB, "MFC1 after DMTC1 (6)")?;
        soft_assert_eq(mfc1::<7>(), 0xEEFFEEFF, "MFC1 after DMTC1 (7)")?;

        // Write 32 bit value in 64 bit mode and read back as 64 bit to see how they were written to
        // We only do 4 writes to ensure the others stay as they were
        mtc1::<0>(0x33332222);
        mtc1::<1>(0x77776666);
        mtc1::<2>(0xBBBBAAAA);
        mtc1::<3>(0xFFFFEEEE);
        soft_assert_eq(dmfc1::<0>(), 0x00001111_33332222, "DMFC1 after MTC1 (0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_77776666, "DMFC1 after MTC1 (1)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_BBBBAAAA, "DMFC1 after MTC1 (2)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_FFFFEEEE, "DMFC1 after MTC1 (3)")?;
        soft_assert_eq(dmfc1::<4>(), 0x00110011_22332233, "DMFC1 after DMTC1 (with MTC1 on different reg) (4)")?;
        soft_assert_eq(dmfc1::<5>(), 0x44554455_66776677, "DMFC1 after DMTC1 (with MTC1 on different reg) (5)")?;
        soft_assert_eq(dmfc1::<6>(), 0x88998899_AABBAABB, "DMFC1 after DMTC1 (with MTC1 on different reg) (6)")?;
        soft_assert_eq(dmfc1::<7>(), 0xCCDDCCDD_EEFFEEFF, "DMFC1 after DMTC1 (with MTC1 on different reg) (7)")?;

        Ok(())
    }
}

pub struct HalfMode;

impl Test for HalfMode {
    fn name(&self) -> &str { "Move To/From in Half Mode" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // For 64 bit instructions, the odd numbers should be interpreted like even numbers
        set_mode(false);

        // Write 64 bit - for the first 4 we skip the odds, then we don't
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);

        // Read back as 64 bit values (this is just 1:1 what was written)
        soft_assert_eq(dmfc1::<0>(), 0x00001111_22223333, "DMFC1 after DMTC1 (0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x00001111_22223333, "DMFC1 after DMTC1 (1)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1 after DMTC1 (2)")?;
        soft_assert_eq(dmfc1::<3>(), 0x88889999_AAAABBBB, "DMFC1 after DMTC1 (3)")?;
        soft_assert_eq(dmfc1::<4>(), 0x44554455_66776677, "DMFC1 after DMTC1 (4)")?;
        soft_assert_eq(dmfc1::<5>(), 0x44554455_66776677, "DMFC1 after DMTC1 (5)")?;
        soft_assert_eq(dmfc1::<6>(), 0xCCDDCCDD_EEFFEEFF, "DMFC1 after DMTC1 (6)")?;
        soft_assert_eq(dmfc1::<7>(), 0xCCDDCCDD_EEFFEEFF, "DMFC1 after DMTC1 (7)")?;

        // Read back as 32 bit values (this is simply the lower 32 bit)
        soft_assert_eq(mfc1::<0>(), 0x22223333, "MFC1 after DMTC1 (0)")?;
        soft_assert_eq(mfc1::<1>(), 0x00001111, "MFC1 after DMTC1 (1)")?;
        soft_assert_eq(mfc1::<2>(), 0xAAAABBBB, "MFC1 after DMTC1 (2)")?;
        soft_assert_eq(mfc1::<3>(), 0x88889999, "MFC1 after DMTC1 (3)")?;
        soft_assert_eq(mfc1::<4>(), 0x66776677, "MFC1 after DMTC1 (4)")?;
        soft_assert_eq(mfc1::<5>(), 0x44554455, "MFC1 after DMTC1 (5)")?;
        soft_assert_eq(mfc1::<6>(), 0xEEFFEEFF, "MFC1 after DMTC1 (6)")?;
        soft_assert_eq(mfc1::<7>(), 0xCCDDCCDD, "MFC1 after DMTC1 (7)")?;

        // Write 32 bit value and read back as 64 bit to see how they were written to
        // We only do 4 writes to ensure the others stay as they were
        mtc1::<0>(0x33332222);
        mtc1::<1>(0x77776666);
        mtc1::<2>(0xBBBBAAAA);
        mtc1::<3>(0xFFFFEEEE);
        soft_assert_eq(dmfc1::<0>(), 0x77776666_33332222, "DMFC1 after MTC1 (0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x77776666_33332222, "DMFC1 after MTC1 (1)")?;
        soft_assert_eq(dmfc1::<2>(), 0xFFFFEEEE_BBBBAAAA, "DMFC1 after MTC1 (2)")?;
        soft_assert_eq(dmfc1::<3>(), 0xFFFFEEEE_BBBBAAAA, "DMFC1 after MTC1 (3)")?;
        soft_assert_eq(dmfc1::<4>(), 0x44554455_66776677, "DMFC1 after DMTC1 (with MTC1 on different reg) (4)")?;
        soft_assert_eq(dmfc1::<5>(), 0x44554455_66776677, "DMFC1 after DMTC1 (with MTC1 on different reg) (5)")?;
        soft_assert_eq(dmfc1::<6>(), 0xCCDDCCDD_EEFFEEFF, "DMFC1 after DMTC1 (with MTC1 on different reg) (6)")?;
        soft_assert_eq(dmfc1::<7>(), 0xCCDDCCDD_EEFFEEFF, "DMFC1 after DMTC1 (with MTC1 on different reg) (7)")?;


        Ok(())
    }
}

pub struct MixedMode;

impl Test for MixedMode {
    fn name(&self) -> &str { "Move To/From in Mixed Mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);

        // Switch to 32 bit mode and read back
        set_mode(false);

        soft_assert_eq(dmfc1::<0>(), 0x00001111_22223333, "DMFC1 after DMTC1 and switch to 32 bit mode (0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x00001111_22223333, "DMFC1 after DMTC1 and switch to 32 bit mode (1)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1 after DMTC1 and switch to 32 bit mode (2)")?;
        soft_assert_eq(dmfc1::<3>(), 0x88889999_AAAABBBB, "DMFC1 after DMTC1 and switch to 32 bit mode (3)")?;

        soft_assert_eq(mfc1::<0>(), 0x22223333, "MFC1 after DMTC1 and switch to 32 bit mode (0)")?;
        soft_assert_eq(mfc1::<1>(), 0x00001111, "MFC1 after DMTC1 and switch to 32 bit mode (1)")?;
        soft_assert_eq(mfc1::<2>(), 0xAAAABBBB, "MFC1 after DMTC1 and switch to 32 bit mode (2)")?;
        soft_assert_eq(mfc1::<3>(), 0x88889999, "MFC1 after DMTC1 and switch to 32 bit mode (3)")?;

        // Write in 32 bit mode
        mtc1::<0>(0x33332222);
        mtc1::<1>(0x77776666);
        mtc1::<2>(0xBBBBAAAA);

        // Read back in 64 bit mode to see where things ended up (and to ensure that nothing was dropped)
        set_mode(true);
        soft_assert_eq(dmfc1::<0>(), 0x77776666_33332222, "DMFC1 (64 bit) after various writes (0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1 (64 bit) after various writes (1)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_BBBBAAAA, "DMFC1 (64 bit) after various writes (2)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1 (64 bit) after various writes (3)")?;
        soft_assert_eq(dmfc1::<4>(), 0x00110011_22332233, "DMFC1 (64 bit) after various writes (4)")?;
        soft_assert_eq(dmfc1::<5>(), 0x44554455_66776677, "DMFC1 (64 bit) after various writes (5)")?;
        soft_assert_eq(dmfc1::<6>(), 0x88998899_AABBAABB, "DMFC1 (64 bit) after various writes (6)")?;
        soft_assert_eq(dmfc1::<7>(), 0xCCDDCCDD_EEFFEEFF, "DMFC1 (64 bit) after various writes (7)")?;

        Ok(())
    }
}

/// 32 bit operations in 64 bit mode:
/// - LWC1 and MTC1 leave the upper 32 bit as-is
/// - MOV.S copies the upper 32 bits over (it might be identical to MOV.D)
/// - Everything else clears the upper 32 bits to 0
pub struct UpperBitsOf32BitOperationFull;

impl Test for UpperBitsOf32BitOperationFull {
    fn name(&self) -> &str { "Upper bits of 32 bit operation (full mode)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<11>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<12>(0x99988877_76665554);
        dmtc1::<13>(0x44333222_11100000);
        dmtc1::<14>(0x10101010_20202020);
        dmtc1::<15>(0x30303030_40404040);
        dmtc1::<16>(0x50505050_60606060);
        dmtc1::<17>(0x70707070_80808080);


        dmtc1::<24>(0x00000123_11223344);
        dmtc1::<25>(0x00001234_11223344);
        dmtc1::<27>(266f64.to_bits());
        dmtc1::<28>((-10f32).to_bits() as u64 | 0x1234567_00000000);
        dmtc1::<30>(16f32.to_bits() as u64 | 0x1234567_00000000);
        dmtc1::<31>((-16f32).to_bits() as u64 | 0x1234567_00000000);

        let a: f32 = 1234.5f32;

        unsafe {
            asm!("
                .set noat
                ADD.S $0, $28, $30
                SUB.S $1, $28, $30
                MUL.S $2, $28, $30
                DIV.S $3, $28, $30
                SQRT.S $4, $30
                ABS.S $5, $28
                NEG.S $6, $30
                MOV.S $7, $31

                CVT.S.D $8, $27
                CVT.S.L $9, $24
                CVT.S.W $10, $24

                CVT.W.S $11, $30
                ROUND.W.S $12, $30
                TRUNC.W.S $13, $30
                CEIL.W.S $14, $30
                FLOOR.W.S $15, $30

                LWC1 $16, 0({a})
                MTC1 {a}, $17
            ", a = in(reg) &a)
        }

        // Read back as 64 bit values
        soft_assert_eq(dmfc1::<0>(), 6f32.to_bits() as u64, "DMFC1 after ADD.S (0)")?;
        soft_assert_eq(dmfc1::<1>(), (-26f32).to_bits() as u64, "DMFC1 after SUB.S (1)")?;
        soft_assert_eq(dmfc1::<2>(), (-160f32).to_bits() as u64, "DMFC1 after MUL.S (2)")?;
        soft_assert_eq(dmfc1::<3>(), (-0.625f32).to_bits() as u64, "DMFC1 after DIV.S (3)")?;
        soft_assert_eq(dmfc1::<4>(), 4f32.to_bits() as u64, "DMFC1 after SQRT.S (4)")?;
        soft_assert_eq(dmfc1::<5>(), 10f32.to_bits() as u64, "DMFC1 after ABS.S (5)")?;
        soft_assert_eq(dmfc1::<6>(), (-16f32).to_bits() as u64, "DMFC1 after NEG.S (6)")?;
        soft_assert_eq(dmfc1::<7>(), 0x01234567_c1800000, "DMFC1 after MOV.S (7)")?;

        soft_assert_eq(dmfc1::<8>(), 266f32.to_bits() as u64, "DMFC1 after CVT.S.D (8)")?;
        soft_assert_eq(dmfc1::<9>(), (0x123_11223344u64 as f32).to_bits() as u64, "DMFC1 after CVT.S.L (9)")?;
        soft_assert_eq(dmfc1::<10>(), (0x11223344u32 as f32).to_bits() as u64, "DMFC1 after CVT.S.W (10)")?;

        soft_assert_eq(dmfc1::<11>(), 16u64, "DMFC1 after CVT.W.S (11)")?;
        soft_assert_eq(dmfc1::<12>(), 16u64, "DMFC1 after ROUND.W.S (12)")?;
        soft_assert_eq(dmfc1::<13>(), 16u64, "DMFC1 after TRUNC.W.S (13)")?;
        soft_assert_eq(dmfc1::<14>(), 16u64, "DMFC1 after CEIL.W.S (14)")?;
        soft_assert_eq(dmfc1::<15>(), 16u64, "DMFC1 after FLOOR.W.S (15)")?;

        soft_assert_eq(dmfc1::<16>(), 0x50505050_00000000u64 | (1234.5f32.to_bits() as u64), "DMFC1 after LWC1 (16)")?;
        soft_assert_eq(dmfc1::<17>(), 0x70707070_00000000u64 | (&a as *const f32 as u64), "DMFC1 after MTC1 (17)")?;

        Ok(())
    }
}

pub struct UpperBitsOf32BitOperationHalf;

impl Test for UpperBitsOf32BitOperationHalf {
    fn name(&self) -> &str { "Upper bits of 32 bit operation (half mode)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<11>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<12>(0x99988877_76665554);
        dmtc1::<13>(0x44333222_11100000);
        dmtc1::<14>(0x10101010_20202020);
        dmtc1::<15>(0x30303030_40404040);
        dmtc1::<16>(0x50505050_60606060);
        dmtc1::<17>(0x70707070_80808080);

        dmtc1::<24>(0x00000123_11223344);
        dmtc1::<25>(0x00000012_11223355);
        dmtc1::<26>(266f64.to_bits());
        dmtc1::<27>((-266f64).to_bits());
        dmtc1::<28>((-10f32).to_bits() as u64 | 0x1234567_00000000);
        dmtc1::<29>(10f32.to_bits() as u64 | 0x1234567_00000000);
        dmtc1::<30>(16f32.to_bits() as u64 | 0x1234567_00000000);
        dmtc1::<31>((-16f32).to_bits() as u64 | 0x1234567_00000000);

        set_mode(false);
        let mut result: [u64; 32] = [0u64; 32];
        const STATUS_INDEX: u32 = RegisterIndex::Status as u32;
        const FULL_MODE: u32 = Status::DEFAULT.raw_value();
        unsafe {
            asm!("
                .set noat
                ADD.S $0, $28, $30
                ADD.S $1, $29, $30
                ADD.S $2, $28, $31

                SUB.S $3, $28, $30
                SUB.S $4, $29, $30
                SUB.S $5, $28, $31

                MUL.S $6, $28, $30
                MUL.S $7, $29, $30
                MUL.S $8, $28, $31

                DIV.S $9, $28, $30
                DIV.S $10, $28, $30
                DIV.S $11, $28, $30

                SQRT.S $12, $30
                SQRT.S $13, $31

                ABS.S $14, $28
                ABS.S $15, $29

                NEG.S $16, $30
                NEG.S $17, $31

                MOV.S $18, $30
                MOV.S $19, $31

                NOP
                NOP
                mtc0 {full_reg}, ${STATUS_INDEX}
                NOP
                NOP
                SDC1 $0, (8*0)({result})
                SDC1 $1, (8*1)({result})
                SDC1 $2, (8*2)({result})
                SDC1 $3, (8*3)({result})
                SDC1 $4, (8*4)({result})
                SDC1 $5, (8*5)({result})
                SDC1 $6, (8*6)({result})
                SDC1 $7, (8*7)({result})
                SDC1 $8, (8*8)({result})
                SDC1 $9, (8*9)({result})
                SDC1 $10, (8*10)({result})
                SDC1 $11, (8*11)({result})
                SDC1 $12, (8*12)({result})
                SDC1 $13, (8*13)({result})
                SDC1 $14, (8*14)({result})
                SDC1 $15, (8*15)({result})
                SDC1 $16, (8*16)({result})
                SDC1 $17, (8*17)({result})
                SDC1 $18, (8*18)({result})
                SDC1 $19, (8*19)({result})
            ", result = in(reg) &mut result, full_reg = in(reg) FULL_MODE, STATUS_INDEX = const STATUS_INDEX)
        }
        set_mode(true);

        // Read back as 64 bit values
        soft_assert_eq(result[0], 6f32.to_bits() as u64, "Result after ADD.S (0)")?;
        soft_assert_eq(result[1], 6f32.to_bits() as u64, "Result after ADD.S (1)")?;
        soft_assert_eq(result[2], (-26f32).to_bits() as u64, "Result after ADD.S (2)")?;

        soft_assert_eq(result[3], (-26f32).to_bits() as u64, "Result after SUB.S (3)")?;
        soft_assert_eq(result[4], (-26f32).to_bits() as u64, "Result after SUB.S (4)")?;
        soft_assert_eq(result[5], 6f32.to_bits() as u64, "Result after SUB.S (5)")?;

        soft_assert_eq(result[6], (-160f32).to_bits() as u64, "Result after MUL.S (6)")?;
        soft_assert_eq(result[7],(-160f32).to_bits() as u64, "Result after MUL.S (7)")?;
        soft_assert_eq(result[8], 160f32.to_bits() as u64, "Result after MUL.S (8)")?;

        soft_assert_eq(result[9], (-0.625f32).to_bits() as u64, "Result after DIV.S (9)")?;
        soft_assert_eq(result[10], (-0.625f32).to_bits() as u64, "Result after DIV.S (10)")?;
        soft_assert_eq(result[11], (-0.625f32).to_bits() as u64, "Result after DIV.S (11)")?;

        soft_assert_eq(result[12], 4f32.to_bits() as u64, "Result after SQRT.S (12)")?;
        soft_assert_eq(result[13], 4f32.to_bits() as u64, "Result after SQRT.S (13)")?;

        soft_assert_eq(result[14], 10f32.to_bits() as u64, "Result after ABS.S (14)")?;
        soft_assert_eq(result[15], 10f32.to_bits() as u64, "Result after ABS.S (15)")?;

        soft_assert_eq(result[16], (-16f32).to_bits() as u64, "Result after NEG.S (16)")?;
        soft_assert_eq(result[17], (-16f32).to_bits() as u64, "Result after NEG.S (17)")?;

        soft_assert_eq(result[18], 0x01234567_41800000, "Result after MOV.S (18)")?;
        soft_assert_eq(result[19], 0x01234567_41800000, "Result after MOV.S (19)")?;

        Ok(())
    }
}

pub struct UpperBitsOf32BitConversionHalf;

impl Test for UpperBitsOf32BitConversionHalf {
    fn name(&self) -> &str { "Upper bits of 32 bit conversions (half mode)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<11>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<12>(0x99988877_76665554);
        dmtc1::<13>(0x44333222_11100000);
        dmtc1::<14>(0x10101010_20202020);
        dmtc1::<15>(0x30303030_40404040);
        dmtc1::<16>(0x50505050_60606060);
        dmtc1::<17>(0x70707070_80808080);

        dmtc1::<24>(0x00000123_11223344);
        dmtc1::<25>(0x00000012_21223355);
        dmtc1::<26>(266f64.to_bits());
        dmtc1::<27>((-266f64).to_bits());
        dmtc1::<28>((-10f32).to_bits() as u64 | 0x1234567_00000000);
        dmtc1::<29>(10f32.to_bits() as u64 | 0x1234567_00000000);
        dmtc1::<30>(16f32.to_bits() as u64 | 0x1234567_00000000);
        dmtc1::<31>((-16f32).to_bits() as u64 | 0x1234567_00000000);

        set_mode(false);
        let mut result: [u64; 32] = [0u64; 32];
        const STATUS_INDEX: u32 = RegisterIndex::Status as u32;
        const FULL_MODE: u32 = Status::DEFAULT.raw_value();
        unsafe {
            asm!("
                // Conversions (no matter the size of the input) ignore the lowest bit of the input
                CVT.S.D $0, $26
                CVT.S.D $1, $27

                CVT.S.L $2, $24
                CVT.S.L $3, $25

                CVT.S.W $4, $24
                CVT.S.W $5, $25

                CVT.W.S $6, $30
                CVT.W.S $7, $31

                ROUND.W.S $8, $30
                ROUND.W.S $9, $31

                TRUNC.W.S $10, $30
                TRUNC.W.S $11, $31

                CEIL.W.S $12, $30
                CEIL.W.S $13, $31

                FLOOR.W.S $14, $30
                FLOOR.W.S $15, $31

                CVT.L.S $16, $30
                CVT.L.S $17, $31
                ROUND.L.S $18, $31
                TRUNC.L.S $19, $31
                CEIL.L.S $20, $31
                FLOOR.L.S $21, $31

                NOP
                NOP
                mtc0 {full_reg}, ${STATUS_INDEX}
                NOP
                NOP
                SDC1 $0, (8*0)({result})
                SDC1 $1, (8*1)({result})
                SDC1 $2, (8*2)({result})
                SDC1 $3, (8*3)({result})
                SDC1 $4, (8*4)({result})
                SDC1 $5, (8*5)({result})
                SDC1 $6, (8*6)({result})
                SDC1 $7, (8*7)({result})
                SDC1 $8, (8*8)({result})
                SDC1 $9, (8*9)({result})
                SDC1 $10, (8*10)({result})
                SDC1 $11, (8*11)({result})
                SDC1 $12, (8*12)({result})
                SDC1 $13, (8*13)({result})
                SDC1 $14, (8*14)({result})
                SDC1 $15, (8*15)({result})
                SDC1 $16, (8*16)({result})
                SDC1 $17, (8*17)({result})
                SDC1 $18, (8*18)({result})
                SDC1 $19, (8*19)({result})
                SDC1 $20, (8*20)({result})
                SDC1 $21, (8*21)({result})
            ", result = in(reg) &mut result, full_reg = in(reg) FULL_MODE, STATUS_INDEX = const STATUS_INDEX)
        }
        set_mode(true);

        // Read back as 64 bit values
        soft_assert_eq(result[0], 266f32.to_bits() as u64, "Result after CVT.S.D (0)")?;
        soft_assert_eq(result[1], 266f32.to_bits() as u64, "Result after CVT.S.D (1)")?;

        soft_assert_eq(result[2], (0x123_11223344u64 as f32).to_bits() as u64, "Result after CVT.S.L (2)")?;
        soft_assert_eq(result[3], (0x123_11223344u64 as f32).to_bits() as u64, "Result after CVT.S.L (3)")?;
        soft_assert_eq(result[4], (0x11223344u32 as f32).to_bits() as u64, "Result after CVT.S.W (4)")?;
        soft_assert_eq(result[5], (0x11223344u32 as f32).to_bits() as u64, "Result after CVT.S.W (5)")?;

        soft_assert_eq(result[6], 16u64, "Result after CVT.W.S (6)")?;
        soft_assert_eq(result[7], 16u64, "Result after CVT.W.S (7)")?;

        soft_assert_eq(result[8], 16u64, "Result after ROUND.W.S (8)")?;
        soft_assert_eq(result[9], 16u64, "Result after ROUND.W.S (9)")?;

        soft_assert_eq(result[10], 16u64, "Result after TRUNC.W.S (10)")?;
        soft_assert_eq(result[11], 16u64, "Result after TRUNC.W.S (11)")?;

        soft_assert_eq(result[12], 16u64, "Result after CEIL.W.S (12)")?;
        soft_assert_eq(result[13], 16u64, "Result after CEIL.W.S (13)")?;

        soft_assert_eq(result[14], 16u64, "Result after FLOOR.W.S (14)")?;
        soft_assert_eq(result[15], 16u64, "Result after FLOOR.W.S (15)")?;

        soft_assert_eq(result[16], 16u64, "Result after CVT.L.S (16)")?;
        soft_assert_eq(result[17], 16u64, "Result after CVT.L.S (17)")?;
        soft_assert_eq(result[18], 16u64, "Result after ROUND.L.S (18)")?;
        soft_assert_eq(result[19], 16u64, "Result after TRUNC.L.S (19)")?;
        soft_assert_eq(result[20], 16u64, "Result after CEIL.L.S (20)")?;
        soft_assert_eq(result[21], 16u64, "Result after FLOOR.L.S (21)")?;

        Ok(())
    }
}

pub struct HalfMode64BitOperationsWithOddIndex;

impl Test for HalfMode64BitOperationsWithOddIndex {
    fn name(&self) -> &str { "64 bit with odd indices (half mode)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<11>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<12>(0x99988877_76665554);
        dmtc1::<13>(0x44333222_11100000);
        dmtc1::<14>(0x10101010_20202020);
        dmtc1::<15>(0x30303030_40404040);
        dmtc1::<16>(0x50505050_60606060);
        dmtc1::<17>(0x70707070_80808080);

        dmtc1::<28>((-10f64).to_bits());
        dmtc1::<29>(10f64.to_bits());
        dmtc1::<30>(16f64.to_bits());
        dmtc1::<31>((-16f64).to_bits());

        set_mode(false);
        let mut result: [u64; 32] = [0u64; 32];
        const STATUS_INDEX: u32 = RegisterIndex::Status as u32;
        const FULL_MODE: u32 = Status::DEFAULT.raw_value();
        unsafe {
            asm!("
                ADD.D $0, $28, $30
                ADD.D $1, $29, $30
                ADD.D $2, $28, $31

                SUB.D $3, $28, $30
                SUB.D $4, $29, $30
                SUB.D $5, $28, $31

                MUL.D $6, $28, $30
                MUL.D $7, $29, $30
                MUL.D $8, $28, $31

                DIV.D $9, $28, $30
                DIV.D $10, $28, $30
                DIV.D $11, $28, $30

                SQRT.D $12, $30
                SQRT.D $13, $31

                ABS.D $14, $28
                ABS.D $15, $29

                NEG.D $16, $30
                NEG.D $17, $31

                MOV.D $18, $30
                MOV.D $19, $31

                NOP
                NOP
                mtc0 {full_reg}, ${STATUS_INDEX}
                NOP
                NOP
                SDC1 $0, (8*0)({result})
                SDC1 $1, (8*1)({result})
                SDC1 $2, (8*2)({result})
                SDC1 $3, (8*3)({result})
                SDC1 $4, (8*4)({result})
                SDC1 $5, (8*5)({result})
                SDC1 $6, (8*6)({result})
                SDC1 $7, (8*7)({result})
                SDC1 $8, (8*8)({result})
                SDC1 $9, (8*9)({result})
                SDC1 $10, (8*10)({result})
                SDC1 $11, (8*11)({result})
                SDC1 $12, (8*12)({result})
                SDC1 $13, (8*13)({result})
                SDC1 $14, (8*14)({result})
                SDC1 $15, (8*15)({result})
                SDC1 $16, (8*16)({result})
                SDC1 $17, (8*17)({result})
                SDC1 $18, (8*18)({result})
                SDC1 $19, (8*19)({result})
            ", result = in(reg) &mut result, full_reg = in(reg) FULL_MODE, STATUS_INDEX = const STATUS_INDEX)
        }
        set_mode(true);

        // Read back as 64 bit values
        soft_assert_eq(result[0], 6f64.to_bits(), "Result after ADD.D (0)")?;
        soft_assert_eq(result[1], 6f64.to_bits(), "Result after ADD.D (1)")?;
        soft_assert_eq(result[2], (-26f64).to_bits(), "Result after ADD.D (2)")?;

        soft_assert_eq(result[3], (-26f64).to_bits(), "Result after SUB.D (3)")?;
        soft_assert_eq(result[4], (-26f64).to_bits(), "Result after SUB.D (4)")?;
        soft_assert_eq(result[5], 6f64.to_bits(), "Result after SUB.D (5)")?;

        soft_assert_eq(result[6], (-160f64).to_bits(), "Result after MUL.D (6)")?;
        soft_assert_eq(result[7],(-160f64).to_bits(), "Result after MUL.D (7)")?;
        soft_assert_eq(result[8], 160f64.to_bits(), "Result after MUL.D (8)")?;

        soft_assert_eq(result[9], (-0.625f64).to_bits(), "Result after DIV.D (9)")?;
        soft_assert_eq(result[10], (-0.625f64).to_bits(), "Result after DIV.D (10)")?;
        soft_assert_eq(result[11], (-0.625f64).to_bits(), "Result after DIV.D (11)")?;

        soft_assert_eq(result[12], 4f64.to_bits(), "Result after SQRT.D (12)")?;
        soft_assert_eq(result[13], 4f64.to_bits(), "Result after SQRT.D (13)")?;

        soft_assert_eq(result[14], 10f64.to_bits(), "Result after ABS.D (14)")?;
        soft_assert_eq(result[15], 10f64.to_bits(), "Result after ABS.D (15)")?;

        soft_assert_eq(result[16], (-16f64).to_bits(), "Result after NEG.D (16)")?;
        soft_assert_eq(result[17], (-16f64).to_bits(), "Result after NEG.D (17)")?;

        soft_assert_eq(result[18], 16f64.to_bits(), "Result after MOV.D (18)")?;
        soft_assert_eq(result[19], 16f64.to_bits(), "Result after MOV.D (19)")?;

        Ok(())
    }
}

pub struct HalfMode64BitConversionsWithOddIndex;

impl Test for HalfMode64BitConversionsWithOddIndex {
    fn name(&self) -> &str { "64 bit with odd indices conversions (half mode)" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<11>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<12>(0x99988877_76665554);
        dmtc1::<13>(0x44333222_11100000);
        dmtc1::<14>(0x10101010_20202020);
        dmtc1::<15>(0x30303030_40404040);
        dmtc1::<16>(0x50505050_60606060);
        dmtc1::<17>(0x70707070_80808080);

        dmtc1::<24>(0x00000123_11223344);
        dmtc1::<25>(0x00000012_21223355);
        dmtc1::<26>((266f32.to_bits() as u64) | 0x1234567_00000000);
        dmtc1::<27>(((-266f32).to_bits() as u64) | 0x1234567_00000000);
        dmtc1::<28>((-10f64).to_bits());
        dmtc1::<29>(10f64.to_bits());
        dmtc1::<30>(16f64.to_bits());
        dmtc1::<31>((-16f64).to_bits());

        set_mode(false);
        let mut result: [u64; 32] = [0u64; 32];
        const STATUS_INDEX: u32 = RegisterIndex::Status as u32;
        const FULL_MODE: u32 = Status::DEFAULT.raw_value();
        unsafe {
            asm!("
                // Conversions (no matter the size of the input) ignore the lowest bit of the input
                CVT.D.S $0, $26
                CVT.D.S $1, $27

                CVT.D.L $2, $24
                CVT.D.L $3, $25

                CVT.D.W $4, $24
                CVT.D.W $5, $25

                CVT.W.D $6, $30
                CVT.W.D $7, $31

                ROUND.W.D $8, $30
                ROUND.W.D $9, $31

                TRUNC.W.D $10, $30
                TRUNC.W.D $11, $31

                CEIL.W.D $12, $30
                CEIL.W.D $13, $31

                FLOOR.W.D $14, $30
                FLOOR.W.D $15, $31

                CVT.L.D $16, $30
                CVT.L.D $17, $31
                ROUND.L.D $18, $31
                TRUNC.L.D $19, $31
                CEIL.L.D $20, $31
                FLOOR.L.D $21, $31

                NOP
                NOP
                mtc0 {full_reg}, ${STATUS_INDEX}
                NOP
                NOP
                SDC1 $0, (8*0)({result})
                SDC1 $1, (8*1)({result})
                SDC1 $2, (8*2)({result})
                SDC1 $3, (8*3)({result})
                SDC1 $4, (8*4)({result})
                SDC1 $5, (8*5)({result})
                SDC1 $6, (8*6)({result})
                SDC1 $7, (8*7)({result})
                SDC1 $8, (8*8)({result})
                SDC1 $9, (8*9)({result})
                SDC1 $10, (8*10)({result})
                SDC1 $11, (8*11)({result})
                SDC1 $12, (8*12)({result})
                SDC1 $13, (8*13)({result})
                SDC1 $14, (8*14)({result})
                SDC1 $15, (8*15)({result})
                SDC1 $16, (8*16)({result})
                SDC1 $17, (8*17)({result})
                SDC1 $18, (8*18)({result})
                SDC1 $19, (8*19)({result})
                SDC1 $20, (8*20)({result})
                SDC1 $21, (8*21)({result})
            ", result = in(reg) &mut result, full_reg = in(reg) FULL_MODE, STATUS_INDEX = const STATUS_INDEX)
        }
        set_mode(true);

        // Read back as 64 bit values
        soft_assert_eq(result[0], 266f64.to_bits() as u64, "Result after CVT.D.S (0)")?;
        soft_assert_eq(result[1], 266f64.to_bits() as u64, "Result after CVT.D.S (1)")?;

        soft_assert_eq(result[2], (0x123_11223344u64 as f64).to_bits(), "Result after CVT.D.L (2)")?;
        soft_assert_eq(result[3], (0x123_11223344u64 as f64).to_bits(), "Result after CVT.D.L (3)")?;
        soft_assert_eq(result[4], (0x11223344u32 as f64).to_bits(), "Result after CVT.D.W (4)")?;
        soft_assert_eq(result[5], (0x11223344u32 as f64).to_bits(), "Result after CVT.D.W (5)")?;

        soft_assert_eq(result[6], 16u64, "Result after CVT.W.D (6)")?;
        soft_assert_eq(result[7], 16u64, "Result after CVT.W.D (7)")?;

        soft_assert_eq(result[8], 16u64, "Result after ROUND.D.S (8)")?;
        soft_assert_eq(result[9], 16u64, "Result after ROUND.D.S (9)")?;

        soft_assert_eq(result[10], 16u64, "Result after TRUNC.D.S (10)")?;
        soft_assert_eq(result[11], 16u64, "Result after TRUNC.D.S (11)")?;

        soft_assert_eq(result[12], 16u64, "Result after CEIL.D.S (12)")?;
        soft_assert_eq(result[13], 16u64, "Result after CEIL.D.S (13)")?;

        soft_assert_eq(result[14], 16u64, "Result after FLOOR.D.S (14)")?;
        soft_assert_eq(result[15], 16u64, "Result after FLOOR.D.S (15)")?;

        soft_assert_eq(result[16], 16u64, "Result after CVT.L.D (16)")?;
        soft_assert_eq(result[17], 16u64, "Result after CVT.L.D (17)")?;
        soft_assert_eq(result[18], 16u64, "Result after ROUND.L.D (18)")?;
        soft_assert_eq(result[19], 16u64, "Result after TRUNC.L.D (19)")?;
        soft_assert_eq(result[20], 16u64, "Result after CEIL.L.D (20)")?;
        soft_assert_eq(result[21], 16u64, "Result after FLOOR.L.D (21)")?;

        Ok(())
    }
}

pub struct ComparisonInHalfModeWithOddRegisters;

impl Test for ComparisonInHalfModeWithOddRegisters {
    fn name(&self) -> &str { "Comparisons in half mode with odd register indices" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00000000_22223333);
        dmtc1::<2>(0x11111111_22223333);
        dmtc1::<4>(0x22222222_12345678);
        dmtc1::<5>(0x22222222_22345678);

        set_mode(false);

        let different02: u32;
        let different54: u32;
        let different45: u32;
        let different55: u32;
        unsafe {
            asm!("
                // Do comparisons use the upper 32 bits?
                ORI {different02}, $0, 0
                C.EQ.S $0, $2

                BC1T 1f
                NOP
                ORI {different02}, $0, 1
1:

                // Can the first argument be odd?
                ORI {different54}, $0, 0
                C.EQ.S $5, $4

                BC1T 1f
                NOP
                ORI {different54}, $0, 1
1:

                // Can the second argument be odd?
                ORI {different45}, $0, 0
                C.EQ.S $4, $5

                BC1T 1f
                NOP
                ORI {different45}, $0, 1
1:

                // To really drive this home, compare 5 and 5. It is expected to be non-equal
                ORI {different55}, $0, 0
                C.EQ.S $5, $5

                BC1T 1f
                NOP
                ORI {different55}, $0, 1
1:
            ", different02 = out(reg) different02, different54 = out(reg) different54, different45 = out(reg) different45, different55 = out(reg) different55)
        }
        set_mode(true);

        // Read back as 64 bit values
        soft_assert_eq(different02, 0, "Upper bits in C.EQ.S should be ignored")?;
        soft_assert_eq(different54, 0, "Lowest bit of fs should be ignored")?;
        soft_assert_eq(different45, 1, "Lowest bit of ft should not be ignored")?;
        soft_assert_eq(different55, 1, "Lowest bit of fs should be ignored, but not of ft")?;

        Ok(())
    }
}

/// LWC1 uses a weird indexing scheme in 32 bit mode
pub struct LWC1InHalfMode;

impl Test for LWC1InHalfMode {
    fn name(&self) -> &str { "LWC1 with odd index in 32 bit mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<11>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<12>(0x99988877_76665554);

        let a: u64 = 0x01234567_89ABCDEF;
        let b: u64 = 0x12345678_9ABCDEF0;

        // First load
        set_mode(false);
        unsafe { asm!("
            .set noat
            LWC1 $0, 4({a})", a = in(reg) &a) }
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x00001111_89ABCDEF, "DMFC1(0) after LWC1($0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after LWC1($0)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1(2) after LWC1($0)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after LWC1($0)")?;

        // Second load
        set_mode(false);
        unsafe { asm!("
            .set noat
            LWC1 $1, 0({a})", a = in(reg) &a) }
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x01234567_89ABCDEF, "DMFC1(0) after LWC1($1)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after LWC1($1)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1(2) after LWC1($1)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after LWC1($1)")?;

        // Third load
        set_mode(false);
        unsafe { asm!("
            .set noat
            LWC1 $2, 0({b})", b = in(reg) &b) }
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x01234567_89ABCDEF, "DMFC1(0) after LWC1($2)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after LWC1($2)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_12345678, "DMFC1(2) after LWC1($2)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after LWC1($2)")?;

        // Fourth load
        set_mode(false);
        unsafe { asm!("
            .set noat
            LWC1 $3, 4({b})", b = in(reg) &b) }
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x01234567_89ABCDEF, "DMFC1(0) after LWC1($3)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after LWC1($3)")?;
        soft_assert_eq(dmfc1::<2>(), 0x9ABCDEF0_12345678, "DMFC1(2) after LWC1($3)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after LWC1($3)")?;

        Ok(())
    }
}

/// LDC1 ignores the lowest bit of the target register
pub struct LDC1InHalfMode;

impl Test for LDC1InHalfMode {
    fn name(&self) -> &str { "LDC1 with odd index in 32 bit mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<11>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<12>(0x99988877_76665554);

        let a: u64 = 0x01234567_89ABCDEF;
        let b: u64 = 0xFEDCBA98_76543210;

        // First load
        set_mode(false);
        unsafe { asm!("
            .set noat
            LDC1 $0, 0({a})", a = in(reg) &a) }
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x01234567_89ABCDEF, "DMFC1(0) after LDC1($0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after LDC1($0)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1(2) after LDC1($0)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after LDC1($0)")?;

        // Second load
        set_mode(false);
        unsafe { asm!("
            .set noat
            LDC1 $1, 0({b})", b = in(reg) &b) }
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0xFEDCBA98_76543210, "DMFC1(0) after LDC1($1)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after LDC1($1)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1(2) after LDC1($1)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after LDC1($1)")?;

        Ok(())
    }
}

/// SWC1 uses a weird indexing scheme in 32 bit mode
pub struct SWC1InHalfMode;

impl Test for SWC1InHalfMode {
    fn name(&self) -> &str { "SWC1 with odd index in 32 bit mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);

        let mut a: u64 = 0x01234567_89ABCDEF;
        let mut b: u64 = 0x12345678_9ABCDEF0;

        // First store
        set_mode(false);
        unsafe { asm!("
            .set noat
            SWC1 $0, 4({a})", a = in(reg) &mut a) }
        set_mode(true);

        soft_assert_eq(a, 0x01234567_22223333, "a after SWC1($0)")?;
        soft_assert_eq(b, 0x12345678_9ABCDEF0, "b after SWC1($0)")?;

        // Second store
        set_mode(false);
        unsafe { asm!("
            .set noat
            SWC1 $1, 0({a})", a = in(reg) &mut a) }
        set_mode(true);

        soft_assert_eq(a, 0x00001111_22223333, "a after SWC1($1)")?;
        soft_assert_eq(b, 0x12345678_9ABCDEF0, "b after SWC1($1)")?;

        // Third store
        set_mode(false);
        unsafe { asm!("
            .set noat
            SWC1 $2, 0({b})", b = in(reg) &mut b) }
        set_mode(true);

        soft_assert_eq(a, 0x00001111_22223333, "a after SWC1($2)")?;
        soft_assert_eq(b, 0xAAAABBBB_9ABCDEF0, "b after SWC1($2)")?;

        // Fourth store
        set_mode(false);
        unsafe { asm!("
            .set noat
            SWC1 $3, 4({b})", b = in(reg) &mut b) }
        set_mode(true);

        soft_assert_eq(a, 0x00001111_22223333, "a after SWC1($3)")?;
        soft_assert_eq(b, 0xAAAABBBB_88889999, "b after SWC1($3)")?;

        Ok(())
    }
}

/// SDC1 ignores the lowest register index bit in 32 bit mode
pub struct SDC1InHalfMode;

impl Test for SDC1InHalfMode {
    fn name(&self) -> &str { "SDC1 with odd index in 32 bit mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);

        let mut a: u64 = 0x01234567_89ABCDEF;
        let mut b: u64 = 0x12345678_9ABCDEF0;

        // First store
        set_mode(false);
        unsafe { asm!("
            .set noat
            SDC1 $0, 0({a})", a = in(reg) &mut a) }
        set_mode(true);

        soft_assert_eq(a, 0x00001111_22223333, "a after SDC1($0)")?;
        soft_assert_eq(b, 0x12345678_9ABCDEF0, "b after SDC1($0)")?;

        // Second store
        set_mode(false);
        unsafe { asm!("
            .set noat
            SDC1 $1, 0({b})", b = in(reg) &mut b) }
        set_mode(true);

        soft_assert_eq(a, 0x00001111_22223333, "a after SDC1($1)")?;
        soft_assert_eq(b, 0x00001111_22223333, "b after SDC1($1)")?;

        Ok(())
    }
}

pub struct MTC1InHalfMode;

impl Test for MTC1InHalfMode {
    fn name(&self) -> &str { "MTC1 with odd index in 32 bit mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<11>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<12>(0x99988877_76665554);

        let a: u64 = 0x01234567_89ABCDEF;
        let b: u64 = 0x12345678_9ABCDEF0;

        // First load
        set_mode(false);
        unsafe { asm!("
            .set noat
            MTC1 {a}, $0", a = in(reg) (a as u32)) }
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x00001111_89ABCDEF, "DMFC1(0) after MTC1($0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after MTC1($0)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1(2) after MTC1($0)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after MTC1($0)")?;

        // Second load
        set_mode(false);
        unsafe { asm!("
            .set noat
            MTC1 {a}, $1", a = in(reg) ((a >> 32) as u32)) }
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x01234567_89ABCDEF, "DMFC1(0) after MTC1($1)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after MTC1($1)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1(2) after MTC1($1)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after MTC1($1)")?;

        // Third load
        set_mode(false);
        unsafe { asm!("
            .set noat
            MTC1 {b}, $2", b = in(reg) ((b >> 32) as u32)) }
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x01234567_89ABCDEF, "DMFC1(0) after MTC1($2)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after MTC1($2)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_12345678, "DMFC1(2) after MTC1($2)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after MTC1($2)")?;

        // Third load
        set_mode(false);
        unsafe { asm!("
            .set noat
            MTC1 {b}, $3", b = in(reg) (b as u32)) }
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x01234567_89ABCDEF, "DMFC1(0) after MTC1($3)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after MTC1($3)")?;
        soft_assert_eq(dmfc1::<2>(), 0x9ABCDEF0_12345678, "DMFC1(2) after MTC1($3)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after MTC1($3)")?;

        Ok(())
    }
}

pub struct DMTC1InHalfMode;

impl Test for DMTC1InHalfMode {
    fn name(&self) -> &str { "DMTC1 with odd index in 32 bit mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<11>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<12>(0x99988877_76665554);

        let a: u64 = 0x01234567_89ABCDEF;
        let b: u64 = 0x12345678_9ABCDEF0;

        // First load
        set_mode(false);
        dmtc1::<0>(a);
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x01234567_89ABCDEF, "DMFC1(0) after DMTC1($0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after DMTC1($0)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1(2) after DMTC1($0)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after DMTC1($0)")?;

        // Second load
        set_mode(false);
        dmtc1::<1>(b);
        set_mode(true);

        soft_assert_eq(dmfc1::<0>(), 0x12345678_9ABCDEF0, "DMFC1(0) after DMTC1($1)")?;
        soft_assert_eq(dmfc1::<1>(), 0x44445555_66667777, "DMFC1(1) after DMTC1($1)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1(2) after DMTC1($1)")?;
        soft_assert_eq(dmfc1::<3>(), 0xCCCCDDDD_EEEEFFFF, "DMFC1(3) after DMTC1($1)")?;

        Ok(())
    }
}

pub struct MFC1InHalfMode;

impl Test for MFC1InHalfMode {
    fn name(&self) -> &str { "MFC1 with odd index in 32 bit mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<30>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<31>(0x99988877_76665554);

        set_mode(false);
        soft_assert_eq(mfc1::<0>(), 0x22223333, "MFC1(0)")?;
        soft_assert_eq(mfc1::<1>(), 0x00001111, "MFC1(1)")?;
        soft_assert_eq(mfc1::<2>(), 0xAAAABBBB, "MFC1(2)")?;
        soft_assert_eq(mfc1::<3>(), 0x88889999, "MFC1(3)")?;
        soft_assert_eq(mfc1::<4>(), 0x22332233, "MFC1(4)")?;
        soft_assert_eq(mfc1::<5>(), 0x00110011, "MFC1(5)")?;
        soft_assert_eq(mfc1::<6>(), 0xAABBAABB, "MFC1(6)")?;
        soft_assert_eq(mfc1::<7>(), 0x88998899, "MFC1(7)")?;
        soft_assert_eq(mfc1::<8>(), 0x23334445, "MFC1(8)")?;
        soft_assert_eq(mfc1::<9>(), 0x00011122, "MFC1(9)")?;
        soft_assert_eq(mfc1::<10>(), 0xDDDEEEFF, "MFC1(10)")?;
        soft_assert_eq(mfc1::<11>(), 0xABBBCCCD, "MFC1(11)")?;
        soft_assert_eq(mfc1::<30>(), 0xCCBBBAAA, "MFC1(30)")?;
        soft_assert_eq(mfc1::<31>(), 0xFEEEDDDC, "MFC1(31)")?;
        set_mode(true);

        Ok(())
    }
}

pub struct DMFC1InHalfMode;

impl Test for DMFC1InHalfMode {
    fn name(&self) -> &str { "DMFC1 with odd index in 32 bit mode" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        dmtc1::<0>(0x00001111_22223333);
        dmtc1::<1>(0x44445555_66667777);
        dmtc1::<2>(0x88889999_AAAABBBB);
        dmtc1::<3>(0xCCCCDDDD_EEEEFFFF);
        dmtc1::<4>(0x00110011_22332233);
        dmtc1::<5>(0x44554455_66776677);
        dmtc1::<6>(0x88998899_AABBAABB);
        dmtc1::<7>(0xCCDDCCDD_EEFFEEFF);
        dmtc1::<8>(0x00011122_23334445);
        dmtc1::<9>(0x55666777_888999AA);
        dmtc1::<10>(0xABBBCCCD_DDDEEEFF);
        dmtc1::<30>(0xFEEEDDDC_CCBBBAAA);
        dmtc1::<31>(0x99988877_76665554);

        set_mode(false);
        soft_assert_eq(dmfc1::<0>(), 0x00001111_22223333, "DMFC1(0)")?;
        soft_assert_eq(dmfc1::<1>(), 0x00001111_22223333, "DMFC1(1)")?;
        soft_assert_eq(dmfc1::<2>(), 0x88889999_AAAABBBB, "DMFC1(2)")?;
        soft_assert_eq(dmfc1::<3>(), 0x88889999_AAAABBBB, "DMFC1(3)")?;
        soft_assert_eq(dmfc1::<4>(), 0x00110011_22332233, "DMFC1(4)")?;
        soft_assert_eq(dmfc1::<5>(), 0x00110011_22332233, "DMFC1(5)")?;
        set_mode(true);

        Ok(())
    }
}
