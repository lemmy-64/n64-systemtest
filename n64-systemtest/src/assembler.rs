use core::iter::Step;
use arbitrary_int::u5;
use bitbybit::bitenum;

// @formatter:off
#[bitenum(u5, exhaustive: true)]
#[allow(dead_code)]
#[derive(PartialOrd, PartialEq, Eq)]
pub enum GPR {
    R0 = 0, AT = 1, V0 = 2, V1 = 3, A0 = 4, A1 = 5, R2 = 6, R3 = 7,
    T0 = 8, T1 = 9, T2 = 10, T3 = 11, T4 = 12, T5 = 13, T6 = 14, T7 = 15,
    S0 = 16, S1 = 17, S2 = 18, S3 = 19, S4 = 20, S5 = 21, S6 = 22, S7 = 23,
    T8 = 24, T9 = 25, K0 = 26, K1 = 27, GP = 28, SP = 29, S8 = 30, RA = 31,
}
// @formatter:on

impl Step for GPR {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        if (*start as usize) < (*end as usize) {
            Some(*end as usize - *start as usize)
        } else {
            None
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let next = start.raw_value().value() as usize + count;
        if next < 32 {
            None
        } else {
            Some(Self::new_with_raw_value(u5::new(next as u8)))
        }
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let next = start.raw_value().value() as usize - count;
        if next < 32 {
            None
        } else {
            Some(Self::new_with_raw_value(u5::new(next as u8)))
        }
    }
}

// @formatter:off
#[bitenum(u5, exhaustive: true)]
#[allow(dead_code)]
#[derive(Debug, PartialOrd, PartialEq, Eq)]
pub enum FR {
    F0 = 0, F1 = 1, F2 = 2, F3 = 3, F4 = 4, F5 = 5, F6 = 6, F7 = 7,
    F8 = 8, F9 = 9, F10 = 10, F11 = 11, F12 = 12, F13 = 13, F14 = 14, F15 = 15,
    F16 = 16, F17 = 17, F18 = 18, F19 = 19, F20 = 20, F21 = 21, F22 = 22, F23 = 23,
    F24 = 24, F25 = 25, F26 = 26, F27 = 27, F28 = 28, F29 = 29, F30 = 30, F31 = 31,
}
// @formatter:on

impl Step for FR {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        if (*start as usize) < (*end as usize) {
            Some(*end as usize - *start as usize)
        } else {
            None
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let next = start.raw_value().value() as usize + count;
        if next < 32 {
            None
        } else {
            Some(Self::new_with_raw_value(u5::new(next as u8)))
        }
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let next = start.raw_value().value() as usize - count;
        if next < 32 {
            None
        } else {
            Some(Self::new_with_raw_value(u5::new(next as u8)))
        }
    }
}


#[allow(dead_code)]
pub enum Opcode {
    SPECIAL = 0,
    REGIMM = 1,
    J = 2,
    JAL = 3,
    BEQ = 4,
    BNE = 5,
    BLEZ = 6,
    BGTZ = 7,
    ADDI = 8,
    ADDIU = 9,
    SLTI = 10,
    SLTIU = 11,
    ANDI = 12,
    ORI = 13,
    XORI = 14,
    LUI = 15,
    COP0 = 16,
    COP1 = 17,
    COP2 = 18,
    COP3 = 19,
    BEQL = 20,
    BNEL = 21,
    BLEZL = 22,
    BGTZL = 23,
    DADDI = 24,
    DADDIU = 25,
    LDL = 26,
    LDR = 27,
    LB = 32,
    LH = 33,
    LWL = 34,
    LW = 35,
    LBU = 36,
    LHU = 37,
    LWR = 38,
    LWU = 39,
    SB = 40,
    SH = 41,
    SWL = 42,
    SW = 43,
    SDL = 44,
    SDR = 45,
    SWR = 46,
    CACHE = 47,
    LL = 48,
    LWC1 = 49,
    LLD = 52,
    LDC1 = 53,
    LD = 55,
    SC = 56,
    SWC1 = 57,
    SCD = 60,
    SDC1 = 61,
    SD = 63,
}

#[allow(dead_code)]
pub enum SpecialOpcode {
    SLL = 0,
    SRL = 2,
    SRA = 3,
    SLLV = 4,
    SRLV = 6,
    SRAV = 7,
    JR = 8,
    JALR = 9,
    SYSCALL = 12,
    BREAK = 13,
    SYNC = 15,
    MFHI = 16,
    MTHI = 17,
    MFLO = 18,
    MTLO = 19,
    DSLLV = 20,
    DSRLV = 22,
    DSRAV = 23,
    MULT = 24,
    MULTU = 25,
    DIV = 26,
    DIVU = 27,
    DMULT = 28,
    DMULTU = 29,
    DDIV = 30,
    DDIVU = 31,
    ADD = 32,
    ADDU = 33,
    SUB = 34,
    SUBU = 35,
    AND = 36,
    OR = 37,
    XOR = 38,
    NOR = 39,
    SLT = 42,
    SLTU = 43,
    DADD = 44,
    DADDU = 45,
    DSUB = 46,
    DSUBU = 47,
    TGE = 48,
    TGEU = 49,
    TLT = 50,
    TLTU = 51,
    TEQ = 52,
    TNE = 54,
    DSLL = 56,
    DSRL = 58,
    DSRA = 59,
    DSLL32 = 60,
    DSRL32 = 62,
    DSRA32 = 63,
}

#[allow(dead_code)]
pub enum RegimmOpcode {
    BLTZ = 0,
    BGEZ = 1,
    BLTZL = 2,
    BGEZL = 3,
    TGEI = 8,
    TGEIU = 9,
    TLTI = 10,
    TLTIU = 11,
    TEQI = 12,
    TNEI = 14,
    BLTZAL = 16,
    BGEZAL = 17,
    BLTZALL = 18,
    BGEZALL = 19,
}

#[allow(dead_code)]
pub enum Cop1Opcode {
    MFC1 = 0,
    DMFC1 = 1,
    CFC1 = 2,
    _DCFC1 = 3,
    MTC1 = 4,
    DMTC1 = 5,
    CTC1 = 6,
    _DCTC1 = 7,
    BC1 = 8,
    S = 16,
    D = 17,
    W = 20,
    L = 21,
}

#[bitenum(u6, exhaustive: false)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub enum Cop1FloatInstruction {
    ADD = 0,
    SUB = 1,
    MUL = 2,
    DIV = 3,
    SQRT = 4,
    ABS = 5,
    MOV = 6,
    NEG = 7,
    ROUND_L = 8,
    TRUNC_L = 9,
    CEIL_L = 10,
    FLOOR_L = 11,
    ROUND_W = 12,
    TRUNC_W = 13,
    CEIL_W = 14,
    FLOOR_W = 15,
    CVT_S = 32,
    CVT_D = 33,
    CVT_W = 36,
    CVT_L = 37,
    C_F = 48,
    C_UN = 49,
    C_EQ = 50,
    C_UEQ = 51,
    C_OLT = 52,
    C_ULT = 53,
    C_OLE = 54,
    C_ULE = 55,
    C_SF = 56,
    C_NGLE = 57,
    C_SEQ = 58,
    C_NGL = 59,
    C_LT = 60,
    C_NGE = 61,
    C_LE = 62,
    C_NGT = 63,
}

#[bitenum(u6, exhaustive: false)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Eq, PartialEq)]
pub enum Cop1Condition {
    F = 48,
    UN = 49,
    EQ = 50,
    UEQ = 51,
    OLT = 52,
    ULT = 53,
    OLE = 54,
    ULE = 55,
    SF = 56,
    NGLE = 57,
    SEQ = 58,
    NGL = 59,
    LT = 60,
    NGE = 61,
    LE = 62,
    NGT = 63,
}

#[allow(dead_code)]
pub enum Cop2Opcode {
    MFC2 = 0,
    DMFC2 = 1,
    CFC2 = 2,
    _DCFC2 = 3,
    MTC2 = 4,
    DMTC2 = 5,
    CTC2 = 6,
    _DCTC2 = 7,
}

#[allow(dead_code)]
pub enum Cop3Opcode {
    MFC3 = 0,
    DMFC3 = 1,
    MTC3 = 4,
    DMTC3 = 5,
}

pub struct Assembler {}

impl Assembler {
    const fn make_main_immediate(op: Opcode, rt: GPR, rs: GPR, imm: u16) -> u32 {
        (imm as u32) |
            ((rt.raw_value().value() as u32) << 16) |
            ((rs.raw_value().value() as u32) << 21) |
            ((op as u32) << 26)
    }

    // TODO: Drop the public make_special, make_regimm_trap

    pub const fn make_special(op: SpecialOpcode, sa: u5, rd: u5, rs: u5, rt: u5) -> u32 {
        (op as u32) |
            ((sa.value() as u32) << 6) |
            ((rd.value() as u32) << 11) |
            ((rt.value() as u32) << 16) |
            ((rs.value() as u32) << 21) |
            ((Opcode::SPECIAL as u32) << 26)
    }

    pub const fn make_regimm_trap(op: RegimmOpcode, rs: u5, imm: u16) -> u32 {
        (imm as u32) |
            ((op as u32) << 16) |
            ((rs.value() as u32) << 21) |
            ((Opcode::REGIMM as u32) << 26)
    }

    const fn make_cop1instruction(instruction: Cop1Opcode, rt: u5, rd: u5) -> u32 {
        ((rd.value() as u32) << 11) |
            ((rt.value() as u32) << 16) |
            ((instruction as u32) << 21) |
            ((Opcode::COP1 as u32) << 26)
    }

    const fn make_cop2instruction(instruction: Cop2Opcode, rt: u5, rd: u5) -> u32 {
        ((rd.value() as u32) << 11) |
            ((rt.value() as u32) << 16) |
            ((instruction as u32) << 21) |
            ((Opcode::COP2 as u32) << 26)
    }

    const fn make_cop3instruction(instruction: Cop3Opcode, rt: u5, rd: u5) -> u32 {
        ((rd.value() as u32) << 11) |
            ((rt.value() as u32) << 16) |
            ((instruction as u32) << 21) |
            ((Opcode::COP3 as u32) << 26)
    }

    const fn make_cop1_double_instruction(instruction: Cop1FloatInstruction, fd: FR, fs: FR, ft: FR) -> u32 {
        (instruction as u32) |
            ((fd.raw_value().value() as u32) << 6) |
            ((fs.raw_value().value() as u32) << 11) |
            ((ft.raw_value().value() as u32) << 16) |
            ((Cop1Opcode::D as u32) << 21) |
            ((Opcode::COP1 as u32) << 26)
    }

    const fn make_cop1_long_instruction(instruction: Cop1FloatInstruction, fd: FR, fs: FR, ft: FR) -> u32 {
        (instruction as u32) |
            ((fd.raw_value().value() as u32) << 6) |
            ((fs.raw_value().value() as u32) << 11) |
            ((ft.raw_value().value() as u32) << 16) |
            ((Cop1Opcode::L as u32) << 21) |
            ((Opcode::COP1 as u32) << 26)
    }

    const fn make_cop1_single_instruction(instruction: Cop1FloatInstruction, fd: FR, fs: FR, ft: FR) -> u32 {
        (instruction as u32) |
            ((fd.raw_value().value() as u32) << 6) |
            ((fs.raw_value().value() as u32) << 11) |
            ((ft.raw_value().value() as u32) << 16) |
            ((Cop1Opcode::S as u32) << 21) |
            ((Opcode::COP1 as u32) << 26)
    }

    const fn make_cop1_word_instruction(instruction: Cop1FloatInstruction, fd: FR, fs: FR, ft: FR) -> u32 {
        (instruction as u32) |
            ((fd.raw_value().value() as u32) << 6) |
            ((fs.raw_value().value() as u32) << 11) |
            ((ft.raw_value().value() as u32) << 16) |
            ((Cop1Opcode::W as u32) << 21) |
            ((Opcode::COP1 as u32) << 26)
    }

    pub const fn make_beq(rt: GPR, rs: GPR, offset_as_instruction_count: i16) -> u32 {
        Self::make_main_immediate(Opcode::BEQ, rt, rs, offset_as_instruction_count as u16)
    }

    pub const fn make_c_cond_s(condition: Cop1Condition, fs: FR, ft: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::new_with_raw_value(condition.raw_value()).ok().unwrap(), FR::F0, fs, ft)
    }

    pub const fn make_c_cond_d(condition: Cop1Condition, fs: FR, ft: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::new_with_raw_value(condition.raw_value()).ok().unwrap(), FR::F0, fs, ft)
    }

    pub const fn make_cfc1(rt: GPR, rd: u5) -> u32 {
        Self::make_cop1instruction(Cop1Opcode::CFC1, rt.raw_value(), rd)
    }

    pub const fn make_dcfc1(rt: GPR, rd: u5) -> u32 {
        Self::make_cop1instruction(Cop1Opcode::_DCFC1, rt.raw_value(), rd)
    }

    pub const fn make_ctc1(rt: GPR, rd: u5) -> u32 {
        Self::make_cop1instruction(Cop1Opcode::CTC1, rt.raw_value(), rd)
    }

    pub const fn make_dctc1(rt: GPR, rd: u5) -> u32 {
        Self::make_cop1instruction(Cop1Opcode::_DCTC1, rt.raw_value(), rd)
    }

    pub const fn make_abs_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::ABS, fd, fs, FR::F0)
    }

    pub const fn make_abs_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::ABS, fd, fs, FR::F0)
    }

    pub const fn make_add_d(fd: FR, fs: FR, ft: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::ADD, fd, fs, ft)
    }

    pub const fn make_add_s(fd: FR, fs: FR, ft: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::ADD, fd, fs, ft)
    }

    /// This is not a valid instruction
    pub const fn make_cvt_d_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::CVT_D, fd, fs, FR::F0)
    }

    pub const fn make_cvt_d_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::CVT_D, fd, fs, FR::F0)
    }

    pub const fn make_cvt_d_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::CVT_D, fd, fs, FR::F0)
    }

    pub const fn make_cvt_d_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::CVT_D, fd, fs, FR::F0)
    }

    pub const fn make_cvt_l_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::CVT_L, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_cvt_l_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::CVT_L, fd, fs, FR::F0)
    }

    pub const fn make_cvt_l_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::CVT_L, fd, fs, FR::F0)
    }

    pub const fn make_cvt_l_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::CVT_L, fd, fs, FR::F0)
    }

    pub const fn make_cvt_s_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::CVT_S, fd, fs, FR::F0)
    }

    pub const fn make_cvt_s_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::CVT_S, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_cvt_s_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::CVT_S, fd, fs, FR::F0)
    }

    pub const fn make_cvt_s_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::CVT_S, fd, fs, FR::F0)
    }

    pub const fn make_cvt_w_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::CVT_W, fd, fs, FR::F0)
    }

    pub const fn make_cvt_w_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::CVT_W, fd, fs, FR::F0)
    }

    pub const fn make_cvt_w_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::CVT_W, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_cvt_w_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::CVT_W, fd, fs, FR::F0)
    }

    pub const fn make_round_w_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::ROUND_W, fd, fs, FR::F0)
    }

    pub const fn make_round_w_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::ROUND_W, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_round_w_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::ROUND_W, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_round_w_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::ROUND_W, fd, fs, FR::F0)
    }

    pub const fn make_round_l_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::ROUND_L, fd, fs, FR::F0)
    }

    pub const fn make_round_l_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::ROUND_L, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_round_l_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::ROUND_L, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_round_l_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::ROUND_L, fd, fs, FR::F0)
    }

    pub const fn make_trunc_w_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::TRUNC_W, fd, fs, FR::F0)
    }

    pub const fn make_trunc_w_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::TRUNC_W, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_trunc_w_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::TRUNC_W, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_trunc_w_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::TRUNC_W, fd, fs, FR::F0)
    }

    pub const fn make_trunc_l_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::TRUNC_L, fd, fs, FR::F0)
    }

    pub const fn make_trunc_l_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::TRUNC_L, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_trunc_l_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::TRUNC_L, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_trunc_l_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::TRUNC_L, fd, fs, FR::F0)
    }

    pub const fn make_floor_w_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::FLOOR_W, fd, fs, FR::F0)
    }

    pub const fn make_floor_w_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::FLOOR_W, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_floor_w_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::FLOOR_W, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_floor_w_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::FLOOR_W, fd, fs, FR::F0)
    }

    pub const fn make_floor_l_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::FLOOR_L, fd, fs, FR::F0)
    }

    pub const fn make_floor_l_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::FLOOR_L, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_floor_l_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::FLOOR_L, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_floor_l_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::FLOOR_L, fd, fs, FR::F0)
    }

    pub const fn make_ceil_w_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::CEIL_W, fd, fs, FR::F0)
    }

    pub const fn make_ceil_w_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::CEIL_W, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_ceil_w_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::CEIL_W, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_ceil_w_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::CEIL_W, fd, fs, FR::F0)
    }

    pub const fn make_ceil_l_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::CEIL_L, fd, fs, FR::F0)
    }

    pub const fn make_ceil_l_l(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_long_instruction(Cop1FloatInstruction::CEIL_L, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_ceil_l_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::CEIL_L, fd, fs, FR::F0)
    }

    /// This is not a valid instruction
    pub const fn make_ceil_l_w(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_word_instruction(Cop1FloatInstruction::CEIL_L, fd, fs, FR::F0)
    }

    pub const fn make_div_d(fd: FR, fs: FR, ft: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::DIV, fd, fs, ft)
    }

    pub const fn make_div_s(fd: FR, fs: FR, ft: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::DIV, fd, fs, ft)
    }

    pub const fn make_mov_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::MOV, fd, fs, FR::F0)
    }

    pub const fn make_mov_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::MOV, fd, fs, FR::F0)
    }

    pub const fn make_mul_d(fd: FR, fs: FR, ft: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::MUL, fd, fs, ft)
    }

    pub const fn make_mul_s(fd: FR, fs: FR, ft: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::MUL, fd, fs, ft)
    }

    pub const fn make_neg_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::NEG, fd, fs, FR::F0)
    }

    pub const fn make_neg_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::NEG, fd, fs, FR::F0)
    }

    pub const fn make_sqrt_d(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::SQRT, fd, fs, FR::F0)
    }

    pub const fn make_sqrt_s(fd: FR, fs: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::SQRT, fd, fs, FR::F0)
    }

    pub const fn make_sub_d(fd: FR, fs: FR, ft: FR) -> u32 {
        Self::make_cop1_double_instruction(Cop1FloatInstruction::SUB, fd, fs, ft)
    }

    pub const fn make_sub_s(fd: FR, fs: FR, ft: FR) -> u32 {
        Self::make_cop1_single_instruction(Cop1FloatInstruction::SUB, fd, fs, ft)
    }

    pub const fn make_sd(rt: GPR, offset: i16, base: GPR) -> u32 {
        Self::make_main_immediate(Opcode::SD, rt, base, offset as u16)
    }

    pub const fn make_sw(rt: GPR, offset: i16, base: GPR) -> u32 {
        Self::make_main_immediate(Opcode::SW, rt, base, offset as u16)
    }

    pub const fn make_sh(rt: GPR, offset: i16, base: GPR) -> u32 {
        Self::make_main_immediate(Opcode::SH, rt, base, offset as u16)
    }

    pub const fn make_sb(rt: GPR, offset: i16, base: GPR) -> u32 {
        Self::make_main_immediate(Opcode::SB, rt, base, offset as u16)
    }

    pub const fn make_lwc1(rt: GPR, offset: i16, base: GPR) -> u32 {
        Self::make_main_immediate(Opcode::LWC1, rt, base, offset as u16)
    }

    pub const fn make_ldc1(rt: GPR, offset: i16, base: GPR) -> u32 {
        Self::make_main_immediate(Opcode::LDC1, rt, base, offset as u16)
    }

    pub const fn make_swc1(rt: GPR, offset: i16, base: GPR) -> u32 {
        Self::make_main_immediate(Opcode::SWC1, rt, base, offset as u16)
    }

    pub const fn make_sdc1(rt: GPR, offset: i16, base: GPR) -> u32 {
        Self::make_main_immediate(Opcode::SDC1, rt, base, offset as u16)
    }

    pub const fn make_mfc1(rt: GPR, rd: FR) -> u32 {
        Self::make_cop1instruction(Cop1Opcode::MFC1, rt.raw_value(), rd.raw_value())
    }

    pub const fn make_mtc1(rt: GPR, rd: FR) -> u32 {
        Self::make_cop1instruction(Cop1Opcode::MTC1, rt.raw_value(), rd.raw_value())
    }

    pub const fn make_dmfc1(rt: GPR, rd: FR) -> u32 {
        Self::make_cop1instruction(Cop1Opcode::DMFC1, rt.raw_value(), rd.raw_value())
    }

    pub const fn make_dmtc1(rt: GPR, rd: FR) -> u32 {
        Self::make_cop1instruction(Cop1Opcode::DMTC1, rt.raw_value(), rd.raw_value())
    }

    pub const fn make_mfc2(rt: GPR, rd: u5) -> u32 {
        Self::make_cop2instruction(Cop2Opcode::MFC2, rt.raw_value(), rd)
    }

    pub const fn make_mtc2(rt: GPR, rd: u5) -> u32 {
        Self::make_cop2instruction(Cop2Opcode::MTC2, rt.raw_value(), rd)
    }

    pub const fn make_dmfc2(rt: GPR, rd: u5) -> u32 {
        Self::make_cop2instruction(Cop2Opcode::DMFC2, rt.raw_value(), rd)
    }

    pub const fn make_dmtc2(rt: GPR, rd: u5) -> u32 {
        Self::make_cop2instruction(Cop2Opcode::DMTC2, rt.raw_value(), rd)
    }

    pub const fn make_cfc2(rt: GPR, rd: u5) -> u32 {
        Self::make_cop2instruction(Cop2Opcode::CFC2, rt.raw_value(), rd)
    }

    pub const fn make_ctc2(rt: GPR, rd: u5) -> u32 {
        Self::make_cop2instruction(Cop2Opcode::CTC2, rt.raw_value(), rd)
    }

    pub const fn make_dcfc2(rt: GPR, rd: u5) -> u32 {
        Self::make_cop2instruction(Cop2Opcode::_DCFC2, rt.raw_value(), rd)
    }

    pub const fn make_dctc2(rt: GPR, rd: u5) -> u32 {
        Self::make_cop2instruction(Cop2Opcode::_DCTC2, rt.raw_value(), rd)
    }

    pub const fn make_mfc3(rt: GPR, rd: u5) -> u32 {
        Self::make_cop3instruction(Cop3Opcode::MFC3, rt.raw_value(), rd)
    }
}
