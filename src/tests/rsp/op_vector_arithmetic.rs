use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::math::vector::Vector;
use crate::rsp::rsp::RSP;
use crate::rsp::rsp_assembler::{CP2FlagsRegister, E, Element, GPR, RSPAssembler, VR, VSARAccumulator};
use crate::rsp::spmem::SPMEM;
use crate::tests::{Level, Test};
use crate::tests::soft_asserts::{soft_assert_eq2, soft_assert_eq_vector};

struct EmulationRegisters {
    source_register1: Vector,
    source_register2: Vector,
    target_register: Vector,
    accum_0_16: Vector,
    vco: u16,
    vcc: u16,
    vce: u8,
}

struct VectorElements {
    source1: u16,
    source2: u16,
    target: u16,
    accum_0_16: u16,
    vco_low: bool,
    vco_high: bool,
    vcc_low: bool,
    vcc_high: bool,
    vce: bool,
}

impl EmulationRegisters {
    pub fn set_vco_low(&mut self, i: usize, b: bool) {
        self.vco = (self.vco & !(1 << i)) | ((b as u16) << i);
    }
    pub fn set_vco_high(&mut self, i: usize, b: bool) {
        let j = i + 8;
        self.vco = (self.vco & !(1 << j)) | ((b as u16) << j);
    }
    pub fn set_vcc_low(&mut self, i: usize, b: bool) {
        self.vcc = (self.vcc & !(1 << i)) | ((b as u16) << i);
    }
    pub fn set_vcc_high(&mut self, i: usize, b: bool) {
        let j = i + 8;
        self.vcc = (self.vcc & !(1 << j)) | ((b as u16) << j);
    }
    pub fn set_vce(&mut self, i: usize, b: bool) {
        self.vce = (self.vce & !(1 << i)) | ((b as u8) << i);
    }
}

trait TestCase {
    fn emit(&self, assembler: &mut RSPAssembler, target: VR, source1: VR, source2: VR, e: Element);
    fn emulate(&self, registers: &mut VectorElements);
}

fn run_test_with_emulation_whole_reg<FEmit: Fn(&mut RSPAssembler, VR, VR, VR, Element), FEmulate: Fn(Element, &mut EmulationRegisters)>(
    vco: u16, vcc: u16, vce: u8,
    e: Element,
    emit: FEmit,
    emulate: FEmulate,
    vector1: Vector, vector2: Vector) -> Result<(), String> {

    // Two vectors to multiply upfront. That sets the accumulator register
    SPMEM::write_vector_into_dmem(0x00, &Vector::from_u16([0x7FFF, 0x7FFF, 0x7FFF, 0x0000, 0x0001, 0xFFFF, 0x7FFF, 0x8000]));
    SPMEM::write_vector_into_dmem(0x10, &Vector::from_u16([0x7FFF, 0xFFFF, 0x0010, 0x0000, 0xFFFF, 0xFFFF, 0x7FFF, 0x8000]));

    // The actual input data for the instruction
    SPMEM::write_vector_into_dmem(0x20, &vector1);
    SPMEM::write_vector_into_dmem(0x30, &vector2);

    // This is what the resulting vector will be filled with before the instruction runs
    SPMEM::write_vector_into_dmem(0x40, &Vector::from_u16([0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF]));

    // Assemble RSP program
    let mut assembler = RSPAssembler::new(0);

    // Do a multiplication to ensure that the accumulator bits are set
    assembler.write_lqv(VR::V0, E::_0, 0x000, GPR::R0);
    assembler.write_lqv(VR::V1, E::_0, 0x010, GPR::R0);
    assembler.write_vmudh(VR::V2, VR::V0, VR::V1, Element::All);
    assembler.write_vmadn(VR::V2, VR::V0, VR::V1, Element::All);

    // The accumulators will now be as follows:
    //    high  mid  low
    // 0: 3FFF 4000 0001
    // 1: FFFF FFFF 8001
    // 2: 0007 FFF7 FFF0
    // 3: 0000 0000 0000
    // 4: FFFF FFFF FFFF
    // 5: 0000 0000 0001
    // 6: 3FFF 4000 0001
    // 7: 3FFF C000 0000
    const ACC_HIGH: Vector = Vector::from_u16([0x3FFF, 0xFFFF, 0x0007, 0x0000, 0xFFFF, 0x0000, 0x3FFF, 0x3FFF]);
    const ACC_MID: Vector = Vector::from_u16([0x4000, 0xFFFF, 0xFFF7, 0x0000, 0xFFFF, 0x0000, 0x4000, 0xC000]);

    const REGISTER_CONFIGURATIONS: [(i16, VR, VR, VR); 5] = [
        // all three different
        (0x90 + 80 * 0, VR::V2, VR::V4, VR::V5),
        // target == source1
        (0x90 + 80 * 1, VR::V6, VR::V6, VR::V7),
        // target == source2
        (0x90 + 80 * 2, VR::V8, VR::V9, VR::V8),
        // source1==source2
        (0x90 + 80 * 3, VR::V10, VR::V11, VR::V11),
        // all three the same
        (0x90 + 80 * 4, VR::V12, VR::V12, VR::V12),
    ];

    // We'll run the test several times with different source/target configurations (so that source and target are also the same).
    for (result_address, target, source1, source2) in REGISTER_CONFIGURATIONS {
        // Set flags
        assembler.write_li(GPR::AT, vco as u32);
        assembler.write_ctc2(CP2FlagsRegister::VCO, GPR::AT);
        assembler.write_li(GPR::AT, vcc as u32);
        assembler.write_ctc2(CP2FlagsRegister::VCC, GPR::AT);
        assembler.write_li(GPR::AT, vce as u32);
        assembler.write_ctc2(CP2FlagsRegister::VCE, GPR::AT);

        // Load the actual input
        assembler.write_lqv(source1, E::_0, 0x020, GPR::R0);
        if source1 != source2 {
            assembler.write_lqv(source2, E::_0, 0x030, GPR::R0);
        }

        // Perform the calculation
        emit(&mut assembler, target, source1, source2, e);

        // Get flags and accumulators
        assembler.write_cfc2(CP2FlagsRegister::VCO, GPR::S0);
        assembler.write_cfc2(CP2FlagsRegister::VCC, GPR::S1);
        assembler.write_cfc2(CP2FlagsRegister::VCE, GPR::S2);
        assembler.write_vsar(VR::V3, VSARAccumulator::High);
        assembler.write_vsar(VR::V4, VSARAccumulator::Mid);
        assembler.write_vsar(VR::V5, VSARAccumulator::Low);

        assembler.write_sw(GPR::S0, GPR::R0, result_address + 0);
        assembler.write_sw(GPR::S1, GPR::R0, result_address + 4);
        assembler.write_sw(GPR::S2, GPR::R0, result_address + 8);
        assembler.write_sqv(target, E::_0, (result_address + 16) as i32, GPR::R0);
        assembler.write_sqv(VR::V3, E::_0, (result_address + 32) as i32, GPR::R0);
        assembler.write_sqv(VR::V4, E::_0, (result_address + 48) as i32, GPR::R0);
        assembler.write_sqv(VR::V5, E::_0, (result_address + 64) as i32, GPR::R0);
    }

    assembler.write_break();

    RSP::start_running(0);

    // Dependings on the input registers, we get different results. However, running the emulation is
    // somewhat expensive so we're making some assumptions:
    // - The first three can be treated the same way (with the exception of complete NOPs, which are handled in the result checker)
    // -
    const TARGET_REGISTER_DEFAULT: Vector = Vector::from_u16([0xFFFF, 0x8001, 0xFFFF, 0, 0xFFFF, 0x0001, 0xFFFF, 0xFFFF]);
    let mut emulation_registers = EmulationRegisters {
        source_register1: vector1,
        source_register2: vector2,
        target_register: TARGET_REGISTER_DEFAULT,
        accum_0_16: Vector::from_u16([0x0001, 0x8001, 0xFFF0, 0x0000, 0xFFFF, 0x0001, 0x0001, 0x0000]),
        vco,
        vcc,
        vce,
    };

    emulate(e, &mut emulation_registers);

    let mut emulation_registers_same_input = EmulationRegisters {
        source_register1: vector1,
        source_register2: vector1,
        target_register: TARGET_REGISTER_DEFAULT,
        accum_0_16: Vector::from_u16([0x0001, 0x8001, 0xFFF0, 0x0000, 0xFFFF, 0x0001, 0x0001, 0x0000]),
        vco,
        vcc,
        vce,
    };

    emulate(e, &mut emulation_registers_same_input);

    RSP::wait_until_rsp_is_halted();

    for (result_address, target, source1, source2) in REGISTER_CONFIGURATIONS {
        let expected_result = if source1 == source2 { &emulation_registers_same_input } else { &emulation_registers };
        let addr = result_address as usize;
        // The default for target_register is only accurate when V2 is the target reg. NOOP instructions don't overwrite it, so don't check for those
        if (target == VR::V2) || (expected_result.target_register != TARGET_REGISTER_DEFAULT) {
            soft_assert_eq_vector(SPMEM::read_vector_from_dmem(addr + 16), expected_result.target_register, || format!("Output register (main calculation result) for {:?},{:?},{:?}[{:?}]", target, source1, source2, e))?;
        }
        soft_assert_eq2(SPMEM::read(addr) as u16, expected_result.vco, || format!("VCO after calculation for {:?},{:?},{:?}[{:?}]", target, source1, source2, e))?;
        soft_assert_eq2(SPMEM::read(addr + 4) as u16, expected_result.vcc, || format!("VCC after calculation for {:?},{:?},{:?}[{:?}]", target, source1, source2, e))?;
        soft_assert_eq2(SPMEM::read(addr + 8) as u8, expected_result.vce, || format!("VCE after calculation for {:?},{:?},{:?}[{:?}]", target, source1, source2, e))?;
        soft_assert_eq_vector(SPMEM::read_vector_from_dmem(addr + 64), expected_result.accum_0_16, || format!("Acc[0..16] after calculation for {:?},{:?},{:?}[{:?}]", target, source1, source2, e))?;
        soft_assert_eq_vector(SPMEM::read_vector_from_dmem(addr + 48), ACC_MID, || format!("Acc[16..32] after calculation for {:?},{:?},{:?}[{:?}]", target, source1, source2, e))?;
        soft_assert_eq_vector(SPMEM::read_vector_from_dmem(addr + 32), ACC_HIGH, || format!("Acc[32..48] after calculation for {:?},{:?},{:?}[{:?}]", target, source1, source2, e))?;
    }

    Ok(())
}

fn run_test_with_emulation_all_flags_and_elements(
    test_case: &dyn TestCase,
    vector1: Vector, vector2: Vector) -> Result<(), String> {
    for e in Element::range() {
        // There are five flags: VCO.low, VCO.high, VCC.low, VCC.high, VCE. We can set the bits in a way that four tests are enough to get through all combinations
        // For VCC and VCE, the first bitmask is the one that should test all combinations for a given vector. Throw in two extras to also have some other cases
        for vco in [0x0000, 0x00FF, 0xFF00, 0xFFFF] {
            for (vcc, vce) in [(0b00001111_00110011, 0b10101001), (0, 0), (0xFFFF, 0xFF), (0xFFFF, 0)] {
                run_test_with_emulation_whole_reg(
                    vco, vcc, vce, e,
                    |assembler, target, source1, source2, e| test_case.emit(assembler, target, source1, source2, e),
                    |e, registers| {
                        for i in 0..8 {
                            let mut vector_elements = VectorElements {
                                source1: registers.source_register1.get16(e.get_effective_element_index(i)),
                                source2: registers.source_register2.get16(i),
                                target: registers.target_register.get16(i),
                                accum_0_16: registers.accum_0_16.get16(i),
                                vco_low: ((registers.vco >> i) & 1) != 0,
                                vco_high: ((registers.vco >> (8 + i)) & 1) != 0,
                                vcc_low: ((registers.vcc >> i) & 1) != 0,
                                vcc_high: ((registers.vcc >> (8 + i)) & 1) != 0,
                                vce: ((registers.vce >> i) & 1) != 0,
                            };
                            test_case.emulate(&mut vector_elements);
                            registers.source_register1.set16(i, vector_elements.source1);
                            registers.source_register2.set16(i, vector_elements.source2);
                            registers.target_register.set16(i, vector_elements.target);
                            registers.accum_0_16.set16(i, vector_elements.accum_0_16);
                            registers.set_vcc_low(i, vector_elements.vcc_low);
                            registers.set_vcc_high(i, vector_elements.vcc_high);
                            registers.set_vco_low(i, vector_elements.vco_low);
                            registers.set_vco_high(i, vector_elements.vco_high);
                            registers.set_vce(i, vector_elements.vce);
                        }
                    }, vector1, vector2)?;
            }
        }
    }

    Ok(())
}

fn run_test_with_emulation_all_flags_and_elements_vector2_variations(
    test_case: &dyn TestCase,
    vector1: Vector, vector2: Vector) -> Result<(), String> {

    run_test_with_emulation_all_flags_and_elements(test_case, vector1, vector2)?;
    run_test_with_emulation_all_flags_and_elements(test_case, vector1, vector2.copy_with_broadcast_16(0))?;

    Ok(())
}

/// A couple of instructions add up the input vectors, put that on the accumulator and otherwise zero out
/// the target register
fn run_vzero<FEmitter: Fn(&mut RSPAssembler, VR, VR, VR, Element)>(emitter: &'static FEmitter) -> Result<(), String> {
    run_test_with_emulation_all_flags_and_elements_vector2_variations(
        make_test_case(
            emitter,
            |elements| {
                elements.accum_0_16 = elements.source1 + elements.source2;
                elements.target = 0;
            }).as_ref(),
        Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0xFFFF]),
        Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0xFFFF]))
}

/// Some instructions do absolutely nothing
fn run_noop<FEmitter: Fn(&mut RSPAssembler, VR, VR, VR, Element)>(emitter: &'static FEmitter) -> Result<(), String> {
    run_test_with_emulation_all_flags_and_elements_vector2_variations(
        make_test_case(
            emitter,
            |_| {}).as_ref(),
        Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0xFFFF]),
        Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0xFFFF]))
}

fn make_test_case<FEmitter: Fn(&mut RSPAssembler, VR, VR, VR, Element) + 'static, FEmulation: Fn(&mut VectorElements) + 'static>(emitter: FEmitter, emulation: FEmulation) -> Box<dyn TestCase> {
    struct CustomTestCase<FInnerEmitter, FInnerEmulation> { emitter: FInnerEmitter, emulation: FInnerEmulation }
    impl<FInnerEmitter: Fn(&mut RSPAssembler, VR, VR, VR, Element), FInnerEmulation: Fn(&mut VectorElements)> TestCase for CustomTestCase<FInnerEmitter, FInnerEmulation> {
        fn emit(&self, assembler: &mut RSPAssembler, target: VR, source1: VR, source2: VR, e: Element) {
            (self.emitter)(assembler, target, source1, source2, e);
        }

        fn emulate(&self, elements: &mut VectorElements) {
            (self.emulation)(elements);
        }
    }
    Box::new(CustomTestCase { emitter, emulation })
}

pub struct VADD {}

impl Test for VADD {
    fn name(&self) -> &str { "RSP VADD" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements_vector2_variations(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vadd(target, source1, source2, e); },
                |elements| {
                    let unclamped = (elements.source1 as i16 as i32) + (elements.source2 as i16 as i32) + elements.vco_low as i32;
                    let clamped = unclamped.clamp(-32768, 32767);
                    elements.target = clamped as u16;
                    elements.accum_0_16 = unclamped as u16;
                    elements.vco_low = false;
                    elements.vco_high = false;
                }
            ).as_ref(),
            Vector::from_u16([0, 1, 0x8000, 0xFFFF, 0x7fff, 0x8001, 0x8000, 0x0001]),
            Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x7fff, 0x8001, 0xFFFF, 0xFFFF]))
    }
}

pub struct VSUB {}

impl Test for VSUB {
    fn name(&self) -> &str { "RSP VSUB" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements_vector2_variations(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vsub(target, source1, source2, e); },
                |elements| {
                    let unclamped = (elements.source2 as i16 as i32) - (elements.source1 as i16 as i32) - elements.vco_low as i32;
                    let clamped = unclamped.clamp(-32768, 32767);
                    elements.target = clamped as u16;
                    elements.accum_0_16 = unclamped as u16;
                    elements.vco_low = false;
                    elements.vco_high = false;
                }
            ).as_ref(),
            Vector::from_u16([0, 1, 0x0010, 0xFFFF, 0x7FFF, 0x7FFF, 0x7FFF, 0x8000]),
            Vector::from_u16([0, 2, 0x7FFF, 0x7FFF, 0x0000, 0xFFFF, 0xFFFE, 0x7FFF]))
    }
}

pub struct VABS {}

impl Test for VABS {
    fn name(&self) -> &str { "RSP VABS" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements_vector2_variations(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vabs(target, source1, source2, e); },
                |elements| {
                    if (elements.source2 as i16) < 0 {
                        if elements.source1 == 0x8000 {
                            elements.accum_0_16 = 0x8000;
                            elements.target = 0x7FFF;
                        } else {
                            elements.accum_0_16 = (-(elements.source1 as i16)) as u16;
                            elements.target = elements.accum_0_16;
                        }
                    } else if elements.source2 == 0 {
                        elements.accum_0_16 = 0;
                        elements.target = 0;
                    } else {
                        elements.accum_0_16 = elements.source1;
                        elements.target = elements.source1;
                    }
                }
            ).as_ref(),
            Vector::from_u16([0x1234, 0x1234, 0x8765, 0x0001, 0xFFFF, 0x0000, 0x7FFF, 0x8000]),
            Vector::from_u16([0x0000, 0x0002, 0x0002, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF]))
    }
}

pub struct VADDC {}

impl Test for VADDC {
    fn name(&self) -> &str { "RSP VADDC" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements_vector2_variations(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vaddc(target, source1, source2, e); },
                |elements| {
                    let sum32 = (elements.source1 as u32) + (elements.source2 as u32);
                    let sum16 = sum32 as u16;
                    elements.vco_low = (sum16 as u32) != sum32;
                    elements.vco_high = false;
                    elements.target = sum16;
                    elements.accum_0_16 = sum16;
                }).as_ref(),
            Vector::from_u16([0x0001, 0x7FFF, 0xF000, 0xF000, 0xFFFF, 0x8000, 0xFFFF, 0xFFFF]),
            Vector::from_u16([0x0001, 0x7FFF, 0x1000, 0xF001, 0xFFFF, 0xFFFF, 0x8000, 0x0001]))
    }
}

pub struct VSUBC {}

impl Test for VSUBC {
    fn name(&self) -> &str { "RSP VSUBC" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements_vector2_variations(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vsubc(target, source1, source2, e); },
                |elements| {
                    let result32 = (elements.source2 as i32) - (elements.source1 as i32);
                    let result16 = result32 as u16;
                    elements.vco_high = result32 != 0;
                    elements.vco_low = result32 < 0;
                    elements.target = result16;
                    elements.accum_0_16 = result16;
                }).as_ref(),
            Vector::from_u16([0x0001, 0x0002, 0xFFFF, 0x0000, 0xFFFF, 0x0050, 0x0050, 0x0050]),
            Vector::from_u16([0x0003, 0x0003, 0x0000, 0xFFFF, 0xFFFF, 0x004F, 0x0050, 0x0051]))
    }
}

pub struct VSUT {}

impl Test for VSUT {
    fn name(&self) -> &str { "RSP VSUT" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vsut(target, source1, source2, e); })
    }
}

pub struct VADDB {}

impl Test for VADDB {
    fn name(&self) -> &str { "RSP VADDB" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vaddb(target, source1, source2, e); })
    }
}

pub struct VSUBB {}

impl Test for VSUBB {
    fn name(&self) -> &str { "RSP VSUBB" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vsubb(target, source1, source2, e); })
    }
}

pub struct VACCB {}

impl Test for VACCB {
    fn name(&self) -> &str { "RSP VACCB" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vaccb(target, source1, source2, e); })
    }
}

pub struct VSUCB {}

impl Test for VSUCB {
    fn name(&self) -> &str { "RSP VSUCB" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vsucb(target, source1, source2, e); })
    }
}

pub struct VSAD {}

impl Test for VSAD {
    fn name(&self) -> &str { "RSP VSAD" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vsad(target, source1, source2, e); })
    }
}

pub struct VSAC {}

impl Test for VSAC {
    fn name(&self) -> &str { "RSP VSAC" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vsac(target, source1, source2, e); })
    }
}


pub struct VSUM {}

impl Test for VSUM {
    fn name(&self) -> &str { "RSP VSUM" }

    fn level(&self) -> Level { Level::Weird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| {
            // Use fewer than 3 NOPs here and the test will fail on hardware - it seems that one
            // of the previous multiplications will still be able to write to the accumulator.
            // See test below
            assembler.write_nop();
            assembler.write_nop();
            assembler.write_nop();
            assembler.write_vsum(target, source1, source2, e);
        })
    }
}

pub struct VSUMNoNops {}

impl Test for VSUMNoNops {
    fn name(&self) -> &str { "RSP VSUM (without NOPs before)" }

    fn level(&self) -> Level { Level::TooWeird }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        // VSUM seems to broken - if it runs after a multiplication, the multiplication might still
        // be able to change (some) of the accumulator - the result is deterministic, so we'll keep
        // the test but this sounds like a bug that no one would probably ever need,
        // so the test it marked as TooWeird to prevent it from running
        run_vzero(&|assembler, target, source1, source2, e| {
            assembler.write_vsum(target, source1, source2, e);
        })
    }
}

pub struct VLT {}

impl Test for VLT {
    fn name(&self) -> &str { "RSP VLT" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements_vector2_variations(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vlt(target, source1, source2, e); },
                |elements| {
                    elements.vcc_high = false;
                    let on_equal = elements.vco_high && elements.vco_low;
                    elements.vcc_low = ((elements.source2 as i16) < (elements.source1 as i16)) || ((elements.source1 == elements.source2) && on_equal);
                    elements.vco_low = false;
                    elements.vco_high = false;
                    elements.target = if elements.vcc_low { elements.source2 } else { elements.source1 };
                    elements.accum_0_16 = elements.target;
                }).as_ref(),
            Vector::from_u16([0x1234, 0x1234, 0x1234, 0xF234, 0xF234, 0xF234, 0xF234, 0x1234]),
            Vector::from_u16([0x1234, 0x1233, 0x1235, 0xF233, 0xF234, 0xF235, 0x1234, 0xF234]))
    }
}

pub struct VEQ {}

impl Test for VEQ {
    fn name(&self) -> &str { "RSP VEQ" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements_vector2_variations(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_veq(target, source1, source2, e); },
                |elements| {
                    elements.vcc_high = false;
                    elements.vcc_low = (elements.source1 == elements.source2) && !elements.vco_high;
                    elements.vco_low = false;
                    elements.vco_high = false;
                    elements.target = elements.source1;
                    elements.accum_0_16 = elements.source1;
                }).as_ref(),
            Vector::from_u16([0x1234, 0x1234, 0x1234, 0xF234, 0xF234, 0xF234, 0xF234, 0x1234]),
            Vector::from_u16([0x1234, 0x1233, 0x1235, 0xF233, 0xF234, 0xF235, 0x1234, 0xF234]))
    }
}

pub struct VNE {}

impl Test for VNE {
    fn name(&self) -> &str { "RSP VNE" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements_vector2_variations(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vne(target, source1, source2, e); },
                |elements| {
                    elements.vcc_high = false;
                    elements.vcc_low = (elements.source1 != elements.source2) || elements.vco_high;
                    elements.vco_low = false;
                    elements.vco_high = false;
                    elements.target = elements.source2;
                    elements.accum_0_16 = elements.source2;
                }).as_ref(),
            Vector::from_u16([0x1234, 0x1234, 0x1234, 0xF234, 0xF234, 0xF234, 0xF234, 0x1234]),
            Vector::from_u16([0x1234, 0x1233, 0x1235, 0xF233, 0xF234, 0xF235, 0x1234, 0xF234]))
    }
}

pub struct VGE {}

impl Test for VGE {
    fn name(&self) -> &str { "RSP VGE" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements_vector2_variations(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vge(target, source1, source2, e); },
                |elements| {
                    elements.vcc_high = false;
                    let on_equal = !(elements.vco_high && elements.vco_low);
                    elements.vcc_low = ((elements.source2 as i16) > (elements.source1 as i16)) || ((elements.source1 == elements.source2) && on_equal);
                    elements.vco_low = false;
                    elements.vco_high = false;
                    elements.target = if elements.vcc_low { elements.source2 } else { elements.source1 };
                    elements.accum_0_16 = elements.target;
                }).as_ref(),
            Vector::from_u16([0x1234, 0x1234, 0x1234, 0xF234, 0xF234, 0xF234, 0xF234, 0x1234]),
            Vector::from_u16([0x1234, 0x1233, 0x1235, 0xF233, 0xF234, 0xF235, 0x1234, 0xF234]))
    }
}

pub struct VMRG {}

impl Test for VMRG {
    fn name(&self) -> &str { "RSP VMRG" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements_vector2_variations(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vmrg(target, source1, source2, e); },
                |elements| {
                    elements.target = if elements.vcc_low { elements.source2 } else { elements.source1 };
                    elements.accum_0_16 = elements.target;
                    elements.vco_low = false;
                    elements.vco_high = false;
                }).as_ref(),
            Vector::from_u16([0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888]),
            Vector::from_u16([0xAAAA, 0xBBBB, 0xCCCC, 0xDDDD, 0xEEEE, 0xFFFF, 0xEFEF, 0xEFEF]))
    }
}

pub struct VAND {}

impl Test for VAND {
    fn name(&self) -> &str { "RSP VAND" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vand(target, source1, source2, e); },
                |elements| {
                    elements.target = elements.source1 & elements.source2;
                    elements.accum_0_16 = elements.target;
                }).as_ref(),
            Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
            Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]))
    }
}

pub struct VNAND {}

impl Test for VNAND {
    fn name(&self) -> &str { "RSP VNAND" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vnand(target, source1, source2, e); },
                |elements| {
                    elements.target = !(elements.source1 & elements.source2);
                    elements.accum_0_16 = elements.target;
                }).as_ref(),
            Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
            Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]))
    }
}


pub struct VOR {}

impl Test for VOR {
    fn name(&self) -> &str { "RSP VOR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vor(target, source1, source2, e); },
                |elements| {
                    elements.target = elements.source1 | elements.source2;
                    elements.accum_0_16 = elements.target;
                }).as_ref(),
            Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
            Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]))
    }
}

pub struct VNOR {}

impl Test for VNOR {
    fn name(&self) -> &str { "RSP VNOR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vnor(target, source1, source2, e); },
                |elements| {
                    elements.target = !(elements.source1 | elements.source2);
                    elements.accum_0_16 = elements.target;
                }).as_ref(),
            Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
            Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]))
    }
}

pub struct VXOR {}

impl Test for VXOR {
    fn name(&self) -> &str { "RSP VXOR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements(
            make_test_case(
            |assembler, target, source1, source2, e| { assembler.write_vxor(target, source1, source2, e); },
            |elements| {
                elements.target = elements.source1 ^ elements.source2;
                elements.accum_0_16 = elements.target;
            }).as_ref(),
            Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
            Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]))
    }
}

pub struct VNXOR {}

impl Test for VNXOR {
    fn name(&self) -> &str { "RSP VNXOR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vnxor(target, source1, source2, e); },
                |elements| {
                    elements.target = !(elements.source1 ^ elements.source2);
                    elements.accum_0_16 = elements.target;
                }).as_ref(),
            Vector::from_u16([0x1111, 0x1245, 0x3333, 0x4444, 0xB0C5, 0x6666, 0x0000, 0xFFFF]),
            Vector::from_u16([0xFF0F, 0xEF20, 0x0000, 0xFFFF, 0x3312, 0x0000, 0xEFEF, 0xEFEF]))
    }
}

pub struct VNOP {}

impl Test for VNOP {
    fn name(&self) -> &str { "RSP VNOP" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_noop(&|assembler, target, source1, source2, e| { assembler.write_vnop(target, source1, source2, e); })
    }
}

pub struct VEXTT {}

impl Test for VEXTT {
    fn name(&self) -> &str { "RSP VEXTT" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vextt(target, source1, source2, e); })
    }
}

pub struct VEXTQ {}

impl Test for VEXTQ {
    fn name(&self) -> &str { "RSP VEXTQ" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vextq(target, source1, source2, e); })
    }
}

pub struct VEXTN {}

impl Test for VEXTN {
    fn name(&self) -> &str { "RSP VEXTN" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vextn(target, source1, source2, e); })
    }
}

pub struct VINST {}

impl Test for VINST {
    fn name(&self) -> &str { "RSP VINST" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vinst(target, source1, source2, e); })
    }
}

pub struct VINSQ {}

impl Test for VINSQ {
    fn name(&self) -> &str { "RSP VINSQ" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vinsq(target, source1, source2, e); })
    }
}

pub struct VINSN {}

impl Test for VINSN {
    fn name(&self) -> &str { "RSP VINSN" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_vinsn(target, source1, source2, e); })
    }
}

pub struct VNULL {}

impl Test for VNULL {
    fn name(&self) -> &str { "RSP VNULL" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_noop(&|assembler, target, source1, source2, e| { assembler.write_vnull(target, source1, source2, e); })
    }
}

pub struct V30 {}

impl Test for V30 {
    fn name(&self) -> &str { "RSP V30" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| {
            // Use fewer than 3 NOPs here and the test will fail on hardware - it seems that one
            // of the previous multiplications will still be able to write to the accumulator.
            // See test below
            assembler.write_nop();
            assembler.write_nop();
            assembler.write_nop();
            assembler.write_v30(target, source1, source2, e);
        })
    }
}

pub struct V31 {}

impl Test for V31 {
    fn name(&self) -> &str { "RSP V31" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| {
            // Use fewer than 3 NOPs here and the test will fail on hardware - it seems that one
            // of the previous multiplications will still be able to write to the accumulator.
            // See test below
            assembler.write_nop();
            assembler.write_nop();
            assembler.write_nop();
            assembler.write_v31(target, source1, source2, e);
        })
    }
}

pub struct V46 {}

impl Test for V46 {
    fn name(&self) -> &str { "RSP V46" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_v46(target, source1, source2, e); })
    }
}

pub struct V47 {}

impl Test for V47 {
    fn name(&self) -> &str { "RSP V47" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_v47(target, source1, source2, e); })
    }
}

pub struct V59 {}

impl Test for V59 {
    fn name(&self) -> &str { "RSP V59" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_vzero(&|assembler, target, source1, source2, e| { assembler.write_v59(target, source1, source2, e); })
    }
}

pub struct VCL {}

impl Test for VCL {
    fn name(&self) -> &str { "RSP VCL" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vcl(target, source1, source2, e); },
                |elements| {
                    if elements.vco_low {
                        let (sum, carry) = elements.source2.overflowing_add(elements.source1);
                        if !elements.vco_high {
                            elements.vcc_low = (((sum == 0) && !carry)) || (elements.vce && ((sum == 0) || !carry));
                        }
                        elements.target = if elements.vcc_low { -(elements.source1 as i16) as u16 } else { elements.source2 };
                    } else {
                        if !elements.vco_high {
                            elements.vcc_high = elements.source2 >= elements.source1;
                        }
                        elements.target = if elements.vcc_high { elements.source1 } else { elements.source2 };
                    }

                    elements.accum_0_16 = elements.target;
                    elements.vco_low = false;
                    elements.vco_high = false;
                    elements.vce = false;
                }).as_ref(),
            Vector::from_u16([0x0000, 0x0001, 0x7FFE, 0x7FFF, 0x8000, 0xFFFE, 0xFFFF, 0x0000]),
            Vector::from_u16([0x8000, 0xFFFE, 0xFFFF, 0x0000, 0x0000, 0x0001, 0x7FFE, 0x7FFF]))
    }
}

pub struct VCH {}

impl Test for VCH {
    fn name(&self) -> &str { "RSP VCH" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vch(target, source1, source2, e); },
                |elements| {
                    let i1 = elements.source1 as i16;
                    let i2 = elements.source2 as i16;
                    elements.vco_low = (i1 ^ i2) < 0;
                    if elements.vco_low {
                        elements.vcc_high = i1 < 0;
                        let sum = i1 + i2;
                        elements.vcc_low = sum <= 0;
                        elements.vce = sum == -1;
                        elements.vco_high = sum != 0 && (i1 != !i2);
                        elements.target = (if elements.vcc_low { -i1 } else { i2 }) as u16;
                    } else {
                        elements.vcc_low = i1 < 0;
                        let diff = i2 - i1;
                        elements.vcc_high = diff >= 0;
                        elements.vce = false;
                        elements.vco_high = diff != 0;
                        elements.target = (if elements.vcc_high { i1 } else { i2 }) as u16;
                    }

                    elements.accum_0_16 = elements.target;
                }).as_ref(),
            Vector::from_u16([0x0000, 0x0001, 0x7FFE, 0x7FFF, 0x8000, 0xFFFE, 0xFFFF, 0x0000]),
            Vector::from_u16([0x8000, 0xFFFE, 0xFFFF, 0x0000, 0x0000, 0x0001, 0x7FFE, 0x7FFF]))
    }
}

pub struct VCR {}

impl Test for VCR {
    fn name(&self) -> &str { "RSP VCR" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        run_test_with_emulation_all_flags_and_elements(
            make_test_case(
                |assembler, target, source1, source2, e| { assembler.write_vcr(target, source1, source2, e); },
                |elements| {
                    let i1 = elements.source1 as i16;
                    let i2 = elements.source2 as i16;
                    let high_bit_set = (i1 ^ i2) < 0;
                    if high_bit_set {
                        elements.vcc_high = i1 < 0;
                        elements.vcc_low = i1 + i2 < 0;
                        elements.target = (if elements.vcc_low { !i1 } else { i2 }) as u16;
                    } else {
                        elements.vcc_low = i1 < 0;
                        elements.vcc_high = (i2 - i1) >= 0;
                        elements.target = (if elements.vcc_high { i1 } else { i2 }) as u16;
                    }

                    elements.accum_0_16 = elements.target;
                    elements.vco_low = false;
                    elements.vco_high = false;
                    elements.vce = false;
                }).as_ref(),
            Vector::from_u16([0x0000, 0x0001, 0x7FFE, 0x7FFF, 0x8000, 0xFFFE, 0xFFFF, 0x0000]),
            Vector::from_u16([0x8000, 0xFFFE, 0xFFFF, 0x0000, 0x0000, 0x0001, 0x7FFE, 0x7FFF]))
    }
}

