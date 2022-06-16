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

pub struct Assembler {}

impl Assembler {
    pub const fn make_loadstore(op: Opcode, rt: u32, offset: u16, base: u32) -> u32 {
        assert!(base <= 0b11111);
        assert!(rt <= 0b11111);

        (offset as u32) |
            (rt << 16) |
            (base << 21) |
            ((op as u32) << 26)
    }

    pub const fn make_special(op: SpecialOpcode, sa: u32, rd: u32, rs: u32, rt: u32) -> u32 {
        assert!(sa <= 0b11111);
        assert!(rd <= 0b11111);
        assert!(rs <= 0b11111);
        assert!(rt <= 0b11111);

        (op as u32) |
            (sa << 6) |
            (rd << 11) |
            (rt << 16) |
            (rs << 21) |
            ((Opcode::SPECIAL as u32) << 26)
    }

    pub const fn make_regimm_trap(op: RegimmOpcode, rs: u32, imm: u16) -> u32 {
        assert!(rs <= 0b11111);

        (imm as u32) |
            ((op as u32) << 16) |
            (rs << 21) |
            ((Opcode::REGIMM as u32) << 26)
    }
}
