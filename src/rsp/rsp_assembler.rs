use core::iter::Step;
use core::mem::transmute;

use crate::rsp::dmem_writer::DMEMWriter;

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum GPR {
    R0 = 0, AT = 1, V0 = 2, V1 = 3, A0 = 4, A1 = 5, R2 = 6, R3 = 7,
    T0 = 8, T1 = 9, T2 = 10, T3 = 11, T4 = 12, T5 = 13, T6 = 14, T7 = 15,
    S0 = 16, S1 = 17, S2 = 18, S3 = 19, S4 = 20, S5 = 21, S6 = 22, S7 = 23,
    T8 = 24, T9 = 25, K0 = 26, K1 = 27, GP = 28, SP = 29, S8 = 30, RA = 31,
}
// @formatter:on

impl GPR {
    pub const fn from_index(index: usize) -> Option<Self> {
        if index <= 31 {
            Some(unsafe { transmute(index as u8) })
        } else {
            None
        }
    }
}

impl Step for GPR {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        if (*start as usize) < (*end as usize) {
            Some(*end as usize - *start as usize)
        } else {
            None
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Self::from_index(start as usize + count)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Self::from_index(start as usize - count)
    }
}

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum VR {
    V0 = 0, V1 = 1, V2 = 2, V3 = 3, V4 = 4, V5 = 5, V6 = 6, V7 = 7,
    V8 = 8, V9 = 9, V10 = 10, V11 = 11, V12 = 12, V13 = 13, V14 = 14, V15 = 15,
    V16 = 16, V17 = 17, V18 = 18, V19 = 19, V20 = 20, V21 = 21, V22 = 22, V23 = 23,
    V24 = 24, V25 = 25, V26 = 26, V27 = 27, V28 = 28, V29 = 29, V30 = 30, V31 = 31,
}
// @formatter:on

impl VR {
    pub const fn from_index(index: usize) -> Option<Self> {
        if index <= 31 {
            Some(unsafe { transmute(index as u8) })
        } else {
            None
        }
    }

    pub const fn index(&self) -> usize {
        *self as usize
    }
}

impl Step for VR {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        if (*start as usize) < (*end as usize) {
            Some(*end as usize - *start as usize)
        } else {
            None
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Self::from_index(start as usize + count)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Self::from_index(start as usize - count)
    }
}

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum Element {
    All = 0, All1 = 1,
    Q0 = 2, Q1 = 3,
    H0 = 4, H1 = 5, H2 = 6, H3 = 7,
    _0 = 8, _1 = 9, _2 = 10, _3 = 11, _4 = 12, _5 = 13, _6 = 14, _7 = 15,
}
// @formatter:on

impl Element {
    pub const fn from_index(index: usize) -> Option<Self> {
        if index <= 15 {
            Some(unsafe { transmute(index as u8) })
        } else {
            None
        }
    }

    pub fn get_effective_element_index(&self, index: usize) -> usize {
        const fn q(n: usize) -> [usize; 8] { [n, n, n + 2, n + 2, n + 4, n + 4, n + 6, n + 6] }
        const fn h(n: usize) -> [usize; 8] { [n, n, n, n, n + 4, n + 4, n + 4, n + 4] }
        const EFFECTIVE_INDEX: [[usize; 8]; 16] = [
            [0, 1, 2, 3, 4, 5, 6, 7],
            [0, 1, 2, 3, 4, 5, 6, 7],
            q(0),
            q(1),
            h(0),
            h(1),
            h(2),
            h(3),
            [0; 8],
            [1; 8],
            [2; 8],
            [3; 8],
            [4; 8],
            [5; 8],
            [6; 8],
            [7; 8],
        ];

        EFFECTIVE_INDEX[*self as usize][index]
    }
}

impl Step for Element {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        if (*start as usize) < (*end as usize) {
            Some(*end as usize - *start as usize)
        } else {
            None
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Self::from_index(start as usize + count)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Self::from_index(start as usize - count)
    }
}


// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum E {
    _0 = 0, _1 = 1, _2 = 2, _3 = 3, _4 = 4, _5=5, _6=6, _7=7,
    _8 = 8, _9 = 9, _10 = 10, _11 = 11, _12 = 12, _13 = 13, _14 = 14, _15 = 15,
}
// @formatter:on

impl E {
    pub const fn from_index(i: usize) -> Option<Self> {
        if i <= 15 {
            Some(unsafe { transmute(i as u8) })
        } else {
            None
        }
    }

    pub const fn index(&self) -> usize {
        *self as usize
    }
}

impl Step for E {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        if (*start as usize) < (*end as usize) {
            Some(*end as usize - *start as usize)
        } else {
            None
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Self::from_index(start as usize + count)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Self::from_index(start as usize - count)
    }
}


// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
enum OP {
    SPECIAL = 0, REGIMM = 1, J = 2, JAL = 3, BEQ = 4, BNE = 5, BLEZ = 6, BGTZ = 7,
    ADDI = 8, ADDIU = 9, SLTI = 10, SLTIU = 11, ANDI = 12, ORI = 13, XORI = 14, LUI = 15,
    COP0 = 16, COP2 = 18,
    LB = 32, LH = 33, LW = 35, LBU = 36, LHU = 37, LWU = 39, SB = 40, SH = 41, SW = 43,
    LWC2 = 50, SWC2 = 58,
}
// @formatter:on

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
enum SpecialOP {
    SLL = 0, SRL = 2, SRA = 3, SLLV = 4, SRLV = 6, SRAV = 7,
    JR = 8, JALR = 9,
    BREAK = 13,
    ADD = 32, ADDU = 33, SUB = 34, SUBU = 35,
    AND = 36, OR = 37, XOR = 38, NOR = 39,
    SLT = 42, SLTU = 43,
}
// @formatter:on

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
enum RegimmOP {
    BLTZ = 0, BGEZ = 1, BLTZAL = 16, BGEZAL = 17,
}
// @formatter:on

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
enum CP0OP {
    MFC0 = 0, MTC0 = 4,
}
// @formatter:on

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
pub enum CP0Register {
    SPAddress = 0, DRAMAddress = 1, ReadLength = 2, WriteLength = 3, SPStatus = 4, DmaFull = 5, DmaBusy = 6, Semaphore = 7,
    DPStart = 8, DPEnd = 9, DPStatus = 11, DPClock = 12
}
// @formatter:on

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
enum WC2OP {
    BV = 0, SV = 1, LV = 2, DV = 3, QV = 4, RV = 5, PV = 6, UV = 7, HV = 8, FV = 9, WV = 10, TV = 11,
}
// @formatter:on

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
enum CP2OP {
    MFC2 = 0, CFC2 = 2, MTC2 = 4, CTC2 = 6, VECTOR = 16,
}
// @formatter:on

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
pub enum CP2FlagsRegister {
    VCO = 0, VCC = 1, VCE = 2
}
// @formatter:on

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
enum VectorOp {
    VMULF = 0, VMULU = 1, VRNDP = 2, VMULQ = 3, VMUDL = 4, VMUDM = 5, VMUDN = 6, VMUDH = 7, VMACF = 8, VMACU = 9, VRNDN = 10, VMACQ = 11, VMADL = 12, VMADM = 13, VMADN = 14, VMADH = 15,
    VADD = 16, VSUB = 17, VSUT = 18, VABS = 19, VADDC = 20, VSUBC = 21, VADDB = 22, VSUBB = 23, VACCB = 24, VSUCB = 25, VSAD = 26, VSAC = 27, VSUM = 28, VSAR = 29, V30 = 30, V31 = 31,
    VLT = 32, VEQ = 33, VNE = 34, VGE = 35, VCL = 36, VCH = 37, VCR = 38, VMRG = 39, VAND = 40, VNAND = 41, VOR = 42, VNOR = 43, VXOR = 44, VNXOR = 45, V46 = 46, V47 = 47,
    VRCP = 48, VRCPL = 49, VRCPH = 50, VMOV = 51, VRSQ = 52, VRSQL = 53, VRSQH = 54, VNOP = 55, VEXTT = 56, VEXTQ = 57, VEXTN = 58, V59 = 59, VINST = 60, VINSQ = 61, VINSN = 62, VNULL = 63,
}
// @formatter:on

// @formatter:off
#[allow(dead_code)]
#[repr(u8)]
pub enum VSARAccumulator {
    High = 8, Mid = 9, Low = 10
}
// @formatter:on

pub struct RSMAssemblerJumpTarget {
    offset: usize,
}

impl RSMAssemblerJumpTarget {
    pub fn new(offset: usize) -> Self { Self { offset } }
}

pub struct RSPAssembler {
    writer: DMEMWriter,
}

impl RSPAssembler {
    pub const fn new(start_offset: usize) -> Self {
        // IMEM starts at 0x1000
        Self { writer: DMEMWriter::new(start_offset) }
    }

    pub fn writer(&self) -> &DMEMWriter { &self.writer }

    pub fn get_jump_target(&self) -> RSMAssemblerJumpTarget {
        RSMAssemblerJumpTarget::new(self.writer.offset())
    }

    fn write_main_immediate(&mut self, op: OP, rt: GPR, rs: GPR, imm: u16) {
        let instruction: u32 =
            (imm as u32) |
                ((rt as u32) << 16) |
                ((rs as u32) << 21) |
                ((op as u32) << 26);
        self.writer.write(instruction);
    }

    fn write_main_jump(&mut self, op: OP, jump_target_shifted_by_2: u32) {
        assert!(jump_target_shifted_by_2 < (1 << 26));
        let instruction: u32 =
            jump_target_shifted_by_2 |
                ((op as u32) << 26);
        self.writer.write(instruction);
    }

    fn write_special(&mut self, function: SpecialOP, sa: u32, rd: GPR, rs: GPR, rt: GPR) {
        assert!(sa <= 0b11111);
        self.writer.write((function as u32) |
            (sa << 6) |
            ((rd as u32) << 11) |
            ((rt as u32) << 16) |
            ((rs as u32) << 21) |
            ((OP::SPECIAL as u32) << 26));
    }

    fn write_regimm(&mut self, regimm_op: RegimmOP, rs: GPR, imm: u16) {
        self.writer.write((imm as u32) |
            ((regimm_op as u32) << 16) |
            ((rs as u32) << 21) |
            ((OP::REGIMM as u32) << 26));
    }

    fn write_cop0(&mut self, cp0op: CP0OP, cp0register: CP0Register, rt: GPR) {
        let instruction: u32 =
            ((cp0register as u32) << 11) |
                ((rt as u32) << 16) |
                ((cp0op as u32) << 21) |
                ((OP::COP0 as u32) << 26);
        self.writer.write(instruction);
    }

    fn write_wc2(&mut self, op: OP, wc2op: WC2OP, vt: VR, element: E, imm7: i32, base: GPR) {
        assert!(imm7 <= 63 && imm7 >= -64);
        let instruction: u32 =
            ((imm7 as u32) & 0b111_1111) |
                ((element as u32) << 7) |
                ((wc2op as u32) << 11) |
                ((vt as u32) << 16) |
                ((base as u32) << 21) |
                ((op as u32) << 26);
        self.writer.write(instruction);
    }

    fn write_cop2(&mut self, cp2op: CP2OP, rd: u32, rt: GPR, e: E) {
        assert!(rd < 32);
        let instruction: u32 =
            ((e as u32) << 7) |
                ((rd as u32) << 11) |
                ((rt as u32) << 16) |
                ((cp2op as u32) << 21) |
                ((OP::COP2 as u32) << 26);
        self.writer.write(instruction);
    }

    fn write_vector(&mut self, vector_op: VectorOp, vd: VR, vt: VR, vs: VR, e: Element) {
        // CP2OP::VECTOR has a bunch of 0 bits at the bottom, which are being reused for e. That explains the strange encoding
        let instruction: u32 =
            (vector_op as u32) |
                ((vd as u32) << 6) |
                ((vs as u32) << 11) |
                ((vt as u32) << 16) |
                ((e as u32) << 21) |
                ((CP2OP::VECTOR as u32) << 21) |
                ((OP::COP2 as u32) << 26);
        self.writer.write(instruction);
    }

    fn write_vector_e(&mut self, vector_op: VectorOp, vd: VR, vt: VR, vs: VR, e: E) {
        // CP2OP::VECTOR has a bunch of 0 bits at the bottom, which are being reused for e. That explains the strange encoding
        let instruction: u32 =
            (vector_op as u32) |
                ((vd as u32) << 6) |
                ((vs as u32) << 11) |
                ((vt as u32) << 16) |
                ((e as u32) << 21) |
                ((CP2OP::VECTOR as u32) << 21) |
                ((OP::COP2 as u32) << 26);
        self.writer.write(instruction);
    }

    // Main instructions
    pub fn write_addi(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::ADDI, rt, rs, imm as u16);
    }

    pub fn write_addiu(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::ADDIU, rt, rs, imm as u16);
    }

    pub fn write_andi(&mut self, rt: GPR, rs: GPR, imm: u16) {
        self.write_main_immediate(OP::ANDI, rt, rs, imm);
    }

    pub fn write_lb(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::LB, rt, rs, imm as u16);
    }

    pub fn write_lbu(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::LBU, rt, rs, imm as u16);
    }

    pub fn write_lh(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::LH, rt, rs, imm as u16);
    }

    pub fn write_lhu(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::LHU, rt, rs, imm as u16);
    }

    pub fn write_lw(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::LW, rt, rs, imm as u16);
    }

    pub fn write_lwu(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::LWU, rt, rs, imm as u16);
    }

    pub fn write_sb(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::SB, rt, rs, imm as u16);
    }

    pub fn write_sh(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::SH, rt, rs, imm as u16);
    }

    pub fn write_slti(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::SLTI, rt, rs, imm as u16);
    }

    pub fn write_sltiu(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::SLTIU, rt, rs, imm as u16);
    }

    pub fn write_sw(&mut self, rt: GPR, rs: GPR, imm: i16) {
        self.write_main_immediate(OP::SW, rt, rs, imm as u16);
    }

    /// Writes a 32 bit value into the target register. Uses 1 or 2 instructions
    pub fn write_li(&mut self, rt: GPR, imm: u32) {
        if (imm & 0xFFFF0000) != 0 {
            self.write_lui(rt, (imm >> 16) as u16);
            if (imm & 0xFFFF) != 0 {
                self.write_ori(rt, rt, imm as u16);
            }
        } else {
            self.write_ori(rt, GPR::R0, imm as u16);
        }
    }

    pub fn write_lui(&mut self, rt: GPR, imm: u16) {
        self.write_main_immediate(OP::LUI, rt, GPR::R0, imm);
    }

    pub fn write_ori(&mut self, rt: GPR, rs: GPR, imm: u16) {
        self.write_main_immediate(OP::ORI, rt, rs, imm);
    }

    pub fn write_xori(&mut self, rt: GPR, rs: GPR, imm: u16) {
        self.write_main_immediate(OP::XORI, rt, rs, imm);
    }

    pub fn write_j(&mut self, destination_as_byte_offset: u32) {
        assert!((destination_as_byte_offset & 3) == 0);
        self.write_main_jump(OP::J, destination_as_byte_offset >> 2);
    }

    pub fn write_jal(&mut self, destination_as_byte_offset: u32) {
        assert!((destination_as_byte_offset & 3) == 0);
        self.write_main_jump(OP::JAL, destination_as_byte_offset >> 2);
    }

    pub fn write_beq(&mut self, rt: GPR, rs: GPR, offset_as_instruction_count: i16) {
        self.write_main_immediate(OP::BEQ, rt, rs, offset_as_instruction_count as u16);
    }

    pub fn write_blez(&mut self, rs: GPR, offset_as_instruction_count: i16) {
        self.write_main_immediate(OP::BLEZ, GPR::R0, rs, offset_as_instruction_count as u16);
    }

    pub fn write_bne(&mut self, rt: GPR, rs: GPR, offset_as_instruction_count: i16) {
        self.write_main_immediate(OP::BNE, rt, rs, offset_as_instruction_count as u16);
    }

    pub fn write_bgtz(&mut self, rs: GPR, offset_as_instruction_count: i16) {
        self.write_main_immediate(OP::BGTZ, GPR::R0, rs, offset_as_instruction_count as u16);
    }

    pub fn write_bgtz_backwards(&mut self, rs: GPR, target: &RSMAssemblerJumpTarget) {
        let offset = (((target.offset - self.writer.offset()) & 0xFFF) >> 2) - 1;
        self.write_bgtz(rs, offset as i16);
    }

    // COP0
    pub fn write_mfc0(&mut self, cp0register: CP0Register, rt: GPR) {
        self.write_cop0(CP0OP::MFC0, cp0register, rt);
    }

    pub fn write_mtc0(&mut self, cp0register: CP0Register, rt: GPR) {
        self.write_cop0(CP0OP::MTC0, cp0register, rt);
    }

    // COP2
    pub fn write_ctc2(&mut self, flags_register: CP2FlagsRegister, rt: GPR) {
        self.write_ctc2_any_index(flags_register as u32, rt);
    }

    pub fn write_ctc2_any_index(&mut self, flags_register: u32, rt: GPR) {
        assert!(flags_register < 32);
        self.write_cop2(CP2OP::CTC2, flags_register as u32, rt, E::_0);
    }

    pub fn write_cfc2(&mut self, flags_register: CP2FlagsRegister, rt: GPR) {
        self.write_cfc2_any_index(flags_register as u32, rt);
    }

    pub fn write_cfc2_any_index(&mut self, flags_register: u32, rt: GPR) {
        assert!(flags_register < 32);
        self.write_cop2(CP2OP::CFC2, flags_register as u32, rt, E::_0);
    }

    pub fn write_mfc2(&mut self, vd: VR, rt: GPR, e: E) {
        self.write_cop2(CP2OP::MFC2, vd as u32, rt, e);
    }

    pub fn write_mtc2(&mut self, vd: VR, rt: GPR, e: E) {
        self.write_cop2(CP2OP::MTC2, vd as u32, rt, e);
    }

    // Special instructions
    pub fn write_sll(&mut self, rd: GPR, rt: GPR, sa: u32) {
        self.write_special(SpecialOP::SLL, sa, rd, GPR::R0, rt);
    }

    pub fn write_sra(&mut self, rd: GPR, rt: GPR, sa: u32) {
        self.write_special(SpecialOP::SRA, sa, rd, GPR::R0, rt);
    }

    pub fn write_srl(&mut self, rd: GPR, rt: GPR, sa: u32) {
        self.write_special(SpecialOP::SRL, sa, rd, GPR::R0, rt);
    }

    pub fn write_sllv(&mut self, rd: GPR, rt: GPR, rs: GPR) {
        self.write_special(SpecialOP::SLLV, 0, rd, rs, rt);
    }

    pub fn write_srav(&mut self, rd: GPR, rt: GPR, rs: GPR) {
        self.write_special(SpecialOP::SRAV, 0, rd, rs, rt);
    }

    pub fn write_srlv(&mut self, rd: GPR, rt: GPR, rs: GPR) {
        self.write_special(SpecialOP::SRLV, 0, rd, rs, rt);
    }

    pub fn write_nop(&mut self) {
        self.write_sll(GPR::R0, GPR::R0, 0);
    }

    pub fn write_add(&mut self, rd: GPR, rt: GPR, rs: GPR) {
        self.write_special(SpecialOP::ADD, 0, rd, rs, rt);
    }

    pub fn write_addu(&mut self, rd: GPR, rt: GPR, rs: GPR) {
        self.write_special(SpecialOP::ADDU, 0, rd, rs, rt);
    }

    pub fn write_sub(&mut self, rd: GPR, rt: GPR, rs: GPR) {
        self.write_special(SpecialOP::SUB, 0, rd, rs, rt);
    }

    pub fn write_subu(&mut self, rd: GPR, rt: GPR, rs: GPR) {
        self.write_special(SpecialOP::SUBU, 0, rd, rs, rt);
    }

    pub fn write_and(&mut self, rd: GPR, rt: GPR, rs: GPR) {
        self.write_special(SpecialOP::AND, 0, rd, rs, rt);
    }

    pub fn write_break(&mut self) {
        self.write_special(SpecialOP::BREAK, 0, GPR::R0, GPR::R0, GPR::R0);
    }

    pub fn write_nor(&mut self, rd: GPR, rs: GPR, rt: GPR) {
        self.write_special(SpecialOP::NOR, 0, rd, rs, rt);
    }

    pub fn write_or(&mut self, rd: GPR, rs: GPR, rt: GPR) {
        self.write_special(SpecialOP::OR, 0, rd, rs, rt);
    }

    pub fn write_xor(&mut self, rd: GPR, rs: GPR, rt: GPR) {
        self.write_special(SpecialOP::XOR, 0, rd, rs, rt);
    }

    pub fn write_slt(&mut self, rd: GPR, rs: GPR, rt: GPR) {
        self.write_special(SpecialOP::SLT, 0, rd, rs, rt);
    }

    pub fn write_sltu(&mut self, rd: GPR, rs: GPR, rt: GPR) {
        self.write_special(SpecialOP::SLTU, 0, rd, rs, rt);
    }

    pub fn write_jr(&mut self, rs: GPR) {
        self.write_special(SpecialOP::JR, 0, GPR::R0, rs, GPR::R0);
    }

    pub fn write_jalr(&mut self, ra: GPR, target: GPR) {
        self.write_special(SpecialOP::JALR, 0, ra, target, GPR::R0);
    }

    // Regimm instructions
    pub fn write_bltz(&mut self, rs: GPR, offset_as_instruction_count: i16) {
        self.write_regimm(RegimmOP::BLTZ, rs, offset_as_instruction_count as u16);
    }

    pub fn write_bgez(&mut self, rs: GPR, offset_as_instruction_count: i16) {
        self.write_regimm(RegimmOP::BGEZ, rs, offset_as_instruction_count as u16);
    }

    pub fn write_bltzal(&mut self, rs: GPR, offset_as_instruction_count: i16) {
        self.write_regimm(RegimmOP::BLTZAL, rs, offset_as_instruction_count as u16);
    }

    pub fn write_bgezal(&mut self, rs: GPR, offset_as_instruction_count: i16) {
        self.write_regimm(RegimmOP::BGEZAL, rs, offset_as_instruction_count as u16);
    }

    // Vector load/store instructions
    pub fn write_lbv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        self.write_wc2(OP::LWC2, WC2OP::BV, vt, element, offset, base);
    }

    pub fn write_ldv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b111) == 0);
        self.write_wc2(OP::LWC2, WC2OP::DV, vt, element, offset >> 3, base);
    }

    pub fn write_lfv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b1111) == 0);
        self.write_wc2(OP::LWC2, WC2OP::FV, vt, element, offset >> 4, base);
    }

    pub fn write_lhv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b1111) == 0);
        self.write_wc2(OP::LWC2, WC2OP::HV, vt, element, offset >> 4, base);
    }

    pub fn write_llv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b11) == 0);
        self.write_wc2(OP::LWC2, WC2OP::LV, vt, element, offset >> 2, base);
    }

    pub fn write_lpv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b111) == 0);
        self.write_wc2(OP::LWC2, WC2OP::PV, vt, element, offset >> 3, base);
    }

    pub fn write_lqv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b1111) == 0);
        self.write_wc2(OP::LWC2, WC2OP::QV, vt, element, offset >> 4, base);
    }

    pub fn write_lrv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b1111) == 0);
        self.write_wc2(OP::LWC2, WC2OP::RV, vt, element, offset >> 4, base);
    }

    pub fn write_lsv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b1) == 0);
        self.write_wc2(OP::LWC2, WC2OP::SV, vt, element, offset >> 1, base);
    }

    pub fn write_ltv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b1111) == 0);
        self.write_wc2(OP::LWC2, WC2OP::TV, vt, element, offset >> 4, base);
    }

    pub fn write_luv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b111) == 0);
        self.write_wc2(OP::LWC2, WC2OP::UV, vt, element, offset >> 3, base);
    }

    pub fn write_lwv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b1111) == 0);
        self.write_wc2(OP::LWC2, WC2OP::WV, vt, element, offset >> 4, base);
    }

    pub fn write_sqv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b1111) == 0);
        self.write_wc2(OP::SWC2, WC2OP::QV, vt, element, offset >> 4, base);
    }

    // Regular vector instructions

    pub fn write_vabs(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VABS, vd, vt, vs, e);
    }

    pub fn write_vaccb(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VACCB, vd, vt, vs, e);
    }

    pub fn write_vadd(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VADD, vd, vt, vs, e);
    }

    pub fn write_vaddb(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VADDB, vd, vt, vs, e);
    }

    pub fn write_vaddc(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VADDC, vd, vt, vs, e);
    }

    pub fn write_vand(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VAND, vd, vt, vs, e);
    }

    pub fn write_vch(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VCH, vd, vt, vs, e);
    }

    pub fn write_vcl(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VCL, vd, vt, vs, e);
    }

    pub fn write_vcr(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VCR, vd, vt, vs, e);
    }

    pub fn write_vextn(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VEXTN, vd, vt, vs, e);
    }

    pub fn write_vextq(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VEXTQ, vd, vt, vs, e);
    }

    pub fn write_vextt(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VEXTT, vd, vt, vs, e);
    }

    pub fn write_vlt(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VLT, vd, vt, vs, e);
    }

    pub fn write_veq(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VEQ, vd, vt, vs, e);
    }

    pub fn write_vge(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VGE, vd, vt, vs, e);
    }

    pub fn write_vinsn(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VINSN, vd, vt, vs, e);
    }

    pub fn write_vinsq(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VINSQ, vd, vt, vs, e);
    }

    pub fn write_vinst(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VINST, vd, vt, vs, e);
    }

    pub fn write_vmacf(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMACF, vd, vt, vs, e);
    }

    pub fn write_vmacq(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMACQ, vd, vt, vs, e);
    }

    pub fn write_vmacu(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMACU, vd, vt, vs, e);
    }

    pub fn write_vmadh(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMADH, vd, vt, vs, e);
    }

    pub fn write_vmadl(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMADL, vd, vt, vs, e);
    }

    pub fn write_vmadm(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMADM, vd, vt, vs, e);
    }

    pub fn write_vmadn(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMADN, vd, vt, vs, e);
    }

    pub fn write_vmrg(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMRG, vd, vt, vs, e);
    }

    pub fn write_vmudh(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMUDH, vd, vt, vs, e);
    }

    pub fn write_vmudl(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMUDL, vd, vt, vs, e);
    }

    pub fn write_vmudn(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMUDN, vd, vt, vs, e);
    }

    pub fn write_vmudm(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMUDM, vd, vt, vs, e);
    }

    pub fn write_vmulf(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMULF, vd, vt, vs, e);
    }

    pub fn write_vmulq(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMULQ, vd, vt, vs, e);
    }

    pub fn write_vmulu(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VMULU, vd, vt, vs, e);
    }

    pub fn write_vnand(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VNAND, vd, vt, vs, e);
    }

    pub fn write_vne(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VNE, vd, vt, vs, e);
    }

    pub fn write_vnop(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VNOP, vd, vt, vs, e);
    }

    pub fn write_vnor(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VNOR, vd, vt, vs, e);
    }

    pub fn write_vnull(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VNULL, vd, vt, vs, e);
    }

    pub fn write_vnxor(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VNXOR, vd, vt, vs, e);
    }

    pub fn write_vor(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VOR, vd, vt, vs, e);
    }

    pub fn write_vrndn(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VRNDN, vd, vt, vs, e);
    }

    pub fn write_vrndp(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VRNDP, vd, vt, vs, e);
    }

    pub fn write_vsac(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VSAC, vd, vt, vs, e);
    }

    pub fn write_vsad(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VSAD, vd, vt, vs, e);
    }

    pub fn write_vsar_any_index(&mut self, vd: VR, vt: VR, vs: VR, e: E) {
        self.write_vector_e(VectorOp::VSAR, vd, vt, vs, e);
    }

    pub fn write_vsar(&mut self, vd: VR, source: VSARAccumulator) {
        self.write_vsar_any_index(vd, VR::V0, VR::V0,  E::from_index(source as usize).unwrap());
    }

    pub fn write_vsub(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VSUB, vd, vt, vs, e);
    }

    pub fn write_vsubb(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VSUBB, vd, vt, vs, e);
    }

    pub fn write_vsubc(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VSUBC, vd, vt, vs, e);
    }

    pub fn write_vsucb(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VSUCB, vd, vt, vs, e);
    }

    pub fn write_vsum(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VSUM, vd, vt, vs, e);
    }

    pub fn write_vsut(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VSUT, vd, vt, vs, e);
    }

    pub fn write_vxor(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::VXOR, vd, vt, vs, e);
    }

    pub fn write_v30(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::V30, vd, vt, vs, e);
    }

    pub fn write_v31(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::V31, vd, vt, vs, e);
    }

    pub fn write_v46(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::V46, vd, vt, vs, e);
    }

    pub fn write_v47(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::V47, vd, vt, vs, e);
    }

    pub fn write_v59(&mut self, vd: VR, vt: VR, vs: VR, e: Element) {
        self.write_vector(VectorOp::V59, vd, vt, vs, e);
    }
}
