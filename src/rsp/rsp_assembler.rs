use crate::rsp::spmem_writer::SPMEMWriter;

#[allow(dead_code)]
#[repr(u8)]
pub enum GPR {
    R0 = 0, AT = 1, V0 = 2, V1 = 3, A0 = 4, A1 = 5, R2 = 6, R3 = 7,
    T0 = 8, T1 = 9, T2 = 10, T3 = 11, T4 = 12, T5 = 13, T6 = 14, T7 = 15,
    S0 = 16, S1 = 17, S2 = 18, S3 = 19, S4 = 20, S5 = 21, S6 = 22, S7 = 23,
    T8 = 24, T9 = 25, K0 = 26, K1 = 27, GP = 28, SP = 29, S8 = 30, RA = 31,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum VR {
    V0 = 0, V1 = 1, V2 = 2, V3 = 3, V4 = 4, V5 = 5, V6 = 6, V7 = 7,
    V8 = 8, V9 = 9, V10 = 10, V11 = 11, V12 = 12, V13 = 13, V14 = 14, V15 = 15,
    V16 = 16, V17 = 17, V18 = 18, V19 = 19, V20 = 20, V21 = 21, V22 = 22, V23 = 23,
    V24 = 24, V25 = 25, V26 = 26, V27 = 27, V28 = 28, V29 = 29, V30 = 30, V31 = 31,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Element {
    All = 0, All1 = 1,
    Q0 = 2, Q1 = 3,
    H0 = 4, H1 = 5, H2 = 6, H3 = 7,
    _0 = 8, _1 = 9, _2 = 10, _3 = 11, _4 = 12, _5 = 13, _6 = 14, _7 = 15,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum E {
    _0 = 0, _1 = 1, _2 = 2, _3 = 3, _4 = 4, _5=5, _6=6, _7=7,
    _8=8, _9=9, _10=10, _11=11, _12=12, _13=13, _14=14, _15=15
}

#[allow(dead_code)]
#[repr(u8)]
pub enum OP {
    SPECIAL = 0, REGIMM = 1, J = 2, JAL = 3, BEQ = 4, BNE = 5, BLEZ = 6, BGTZ = 7,
    ADDI = 8, ADDIU = 9, SLTI = 10, SLTIU = 11, ANDI = 12, ORI = 13, XORI = 14, LUI = 15,
    COP0 = 16, COP2 = 18,
    LB = 32, LH = 33, LW = 35, LBU = 36, LHU = 37, SB = 40, SH = 41, SW = 43,
    LWC2 = 50, SWC2 = 58,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum WC2OP {
    BV = 0, SV = 1, LV = 2, DV = 3, QV = 4, RV = 5, PV = 6, UV = 7, HV = 8, FV = 9, WV = 10, TV = 11,
}

pub struct RSPAssembler {
    writer: SPMEMWriter,
}

impl RSPAssembler {
    pub const fn new(start_offset: usize) -> Self {
        // IMEM starts at 0x1000
        Self { writer: SPMEMWriter::new(start_offset | 0x1000) }
    }

    fn write_main_immediate(&mut self, op: OP, rt: GPR, rs: GPR, imm: u16) {
        let instruction: u32 =
            (imm as u32) |
                ((rt as u32) << 16) |
                ((rs as u32) << 21) |
                ((op as u32) << 26);
        self.writer.write(instruction);
    }

    fn write_special(&mut self, function: u32) {
        assert!(function < 0b111111);
        self.writer.write(function);
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

    // Main instructions
    pub fn write_nop(&mut self) {
        self.write_special(0);
    }

    pub fn write_ori(&mut self, rt: GPR, rs: GPR, imm: u16) {
        self.write_main_immediate(OP::ORI, rt, rs, imm);
    }

    // Special instructions
    pub fn write_break(&mut self) {
        self.write_special(13);
    }

    // Vector load/store instructions
    pub fn write_lqv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b1111) == 0);
        self.write_wc2(OP::LWC2, WC2OP::QV, vt, element, offset >> 4, base);
    }

    pub fn write_sqv(&mut self, vt: VR, element: E, offset: i32, base: GPR) {
        assert!((offset & 0b1111) == 0);
        self.write_wc2(OP::SWC2, WC2OP::QV, vt, element, offset >> 4, base);
    }
}