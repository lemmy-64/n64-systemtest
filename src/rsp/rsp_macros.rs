use crate::rsp::rsp_assembler::{E, Element, GPR, RSPAssembler, VR};

/// Sets the accumulator to the value that is currently in the three registers top::mid::low
pub fn assemble_set_accumulator_to(assembler: &mut RSPAssembler, top: VR, mid: VR, low: VR, scratch: VR, scratch2: VR, scratch3: VR, scratch_gpr: GPR) {
    // Put some constants that we need into scratch2:
    // 0000 40000 0001
    assembler.write_mtc2(scratch2, GPR::R0, E::_0);
    assembler.write_li(scratch_gpr, 0x4000);
    assembler.write_mtc2(scratch2, scratch_gpr, E::_2);
    assembler.write_li(scratch_gpr, 1);
    assembler.write_mtc2(scratch2, scratch_gpr, E::_4);

    // Set top part through 4 VMADH, then middle part through 1 VMADH.
    // However, when the middle part is negative, it will reduce 1 from top. To compensate,
    // add one to high whenever mid is negative

    // Set VCO.low if number is negative
    assembler.write_vaddc(scratch, mid, mid, Element::All);
    assembler.write_vxor(scratch, scratch, scratch, Element::All);
    // Set scratch to 1 if VCO.low; 0 otherwise
    assembler.write_vadd(scratch, scratch2, scratch, Element::_0);
    // Add 0/1 to top, without saturation
    assembler.write_vaddc(scratch, scratch, top, Element::All);

    // Accumulator top: Multiply by 16384 and add up four times.
    assembler.write_vmudh(scratch3, scratch2, scratch, Element::_1);
    assembler.write_vmadh(scratch3, scratch2, scratch, Element::_1);
    assembler.write_vmadh(scratch3, scratch2, scratch, Element::_1);
    assembler.write_vmadh(scratch3, scratch2, scratch, Element::_1);

    // Accumulator mid: Multiply by 1 and add to accumulator
    assembler.write_vmadh(scratch3, scratch2, mid, Element::_2);

    // For low, we can use VADDC with 0 which just sets low

    assembler.write_vaddc(scratch3, scratch2, low, Element::_0);
}

