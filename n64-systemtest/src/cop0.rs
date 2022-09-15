use alloc::string::{String, ToString};
use core::arch::asm;
use arbitrary_int::{u19, u2, u27, u31, u41};
use bitbybit::{bitenum, bitfield};
use crate::exception_handler::expect_exception;

#[allow(dead_code)]
pub enum RegisterIndex {
    Index = 0x00,
    Random = 0x01,
    EntryLo0 = 0x02,
    EntryLo1 = 0x03,
    Context = 0x04,
    PageMask = 0x05,
    Wired = 0x06,
    BadVAddr = 0x08,
    Count = 0x09,
    EntryHi = 0x0A,
    Compare = 0x0B,
    Status = 0x0C,
    Cause = 0x0D,
    ExceptPC = 0x0E,
    PRevID = 0x0F,
    Config = 0x10,
    LLAddr = 0x11,
    WatchLo = 0x12,
    WatchHi = 0x13,
    XContext = 0x14,
    PErr = 0x1A,
    CacheErr = 0x1B,
    TagLo = 0x1C,
    TagHi = 0x1D,
    ErrorEPC = 0x1E,
}

#[bitfield(u32, default: 0)]
#[derive(Debug, PartialEq, Eq)]
pub struct Cause {
    #[bit(31, rw)]
    branch_delay : bool,

    #[bits(28..=29, rw)]
    coprocessor_error : u2,

    #[bit(15, rw)]
    interrupt_compare : bool,

    #[bit(14, rw)]
    interrupt_int4 : bool,

    #[bit(13, rw)]
    interrupt_int3 : bool,

    #[bit(12, rw)]
    interrupt_int2 : bool,

    #[bit(11, rw)]
    interrupt_int1 : bool,

    #[bit(10, rw)]
    interrupt_int0 : bool,

    #[bit(9, rw)]
    interrupt_sw2 : bool,

    #[bit(8, rw)]
    interrupt_sw1 : bool,

    #[bits(2..=6, rw)]
    exception : Option<CauseException>,
}

#[allow(dead_code)]
#[bitenum(u2, exhaustive: false)]
pub enum StatusKSU {
    Kernel = 0,
    Supervisor = 1,
    User = 2,
}

#[bitfield(u32, default: 0)]
#[derive(Debug, Eq, PartialEq)]
pub struct Status {
    #[bit(31, rw)]
    cop3usable : bool,

    #[bit(30, rw)]
    cop2usable : bool,

    #[bit(29, rw)]
    cop1usable : bool,

    #[bit(28, rw)]
    cop0usable : bool,

    #[bit(27, rw)]
    reduced_power : bool,

    #[bit(26, rw)]
    fpu64 : bool,

    #[bit(19, rw)]
    nmi : bool,

    #[bit(15, r)]
    interrupt_mask_compare : bool,

    #[bit(14, r)]
    interrupt_mask_int4 : bool,

    #[bit(13, r)]
    interrupt_mask_int3 : bool,

    #[bit(12, r)]
    interrupt_mask_int2 : bool,

    #[bit(11, r)]
    interrupt_mask_int1 : bool,

    #[bit(10, r)]
    interrupt_mask_int0 : bool,

    #[bit(9, r)]
    interrupt_mask_sw2 : bool,

    #[bit(8, r)]
    interrupt_mask_sw1 : bool,

    #[bit(7, rw)]
    kx : bool,

    #[bit(6, rw)]
    sx : bool,

    #[bit(5, rw)]
    ux : bool,

    #[bits(3..=4, rw)]
    ksu : Option<StatusKSU>,

    #[bit(2, rw)]
    erl : bool,

    #[bit(1, rw)]
    exl : bool,

    #[bit(0, rw)]
    ie : bool,
}

impl Status {
    pub const DEFAULT: Status = Status::new().with_cop1usable(true).with_fpu64(true);
    pub const ADDRESSING_MODE_64_BIT: Status = Self::DEFAULT.with_kx(true).with_sx(true).with_ux(true);
}

#[bitfield(u64, default: 0)]
#[derive(Debug, Eq, PartialEq)]
pub struct Context {
    #[bits(23..=63, rw)]
    pte_base: u41,

    #[bits(4..=22, rw)]
    bad_vpn2: u19,
}

impl Context {
    pub fn from_virtual_address(a: u64) -> Self {
        Self::new().with_bad_vpn2(u19::extract_u64(a, 13))
    }
}

#[bitfield(u64, default: 0)]
#[derive(Debug, Eq, PartialEq)]
pub struct XContext {
    #[bits(33..=63, rw)]
    pte_base: u31,

    #[bits(31..=32, rw)]
    r: u2,

    #[bits(4..=30, rw)]
    bad_vpn2: u27,
}

impl XContext {
    pub fn from_virtual_address(a: u64) -> Self {
        Self::new()
            .with_bad_vpn2(u27::extract_u64(a, 13))
            .with_r(u2::extract_u64(a, 62))
    }
}


#[bitenum(u5, exhaustive: false)]
#[derive(PartialEq, Eq, Debug)]
pub enum CauseException {
    Int = 0,
    Mod = 1,
    TLBL = 2,
    TLBS = 3,
    AdEL = 4,
    AdES = 5,
    IBE = 6,
    DBE = 7,
    Sys = 8,
    Bp = 9,
    RI = 10,
    CopUnusable = 11,
    Ov = 12,
    Tr = 13,
    VirtualCoherencyInstructionFetch = 14,
    FPE = 15,
    Watch = 23,
}

#[inline]
unsafe fn read_cop0<const INDEX: u32>() -> u32 {
    let raw_value: u32;
    unsafe {
        asm!("
        .set noat
        mfc0 {gpr}, ${cop0reg}
    ", gpr = out(reg) raw_value, cop0reg = const INDEX)
    }
    raw_value
}

#[inline]
unsafe fn read_cop0_64<const INDEX: u32>() -> u64 {
    // The inline assembler isn't properly setup for 64 bit - workaround in asm
    let raw_value_lo: u32;
    let raw_value_hi: u32;
    unsafe {
        asm!("
        .set noat
        dmfc0 {tmp}, ${cop0reg}
        add {gpr_lo}, $0, {tmp}
        dsrl32 {gpr_hi}, {tmp}, 0
    ", gpr_lo = out(reg) raw_value_lo, gpr_hi = out(reg) raw_value_hi, tmp = out(reg) _, cop0reg = const INDEX)
    }
    ((raw_value_hi as u64) << 32) | (raw_value_lo as u64)
}

#[inline]
unsafe fn write_cop0<const INDEX: u32>(value: u32) {
    unsafe {
        asm!("
        .set noat
        mtc0 {gpr}, ${cop0reg}
        nop
        nop
    ", gpr = in(reg) value, cop0reg = const INDEX)
    }
}

#[inline]
unsafe fn write_cop0_64<const INDEX: u32>(value: u64) {
    unsafe {
        asm!("
        .set noat
        dsll32 {tmp}, {gpr_hi}, 0
        // Zero extend gpr_lo
        dsll32 {tmp2}, {gpr_lo}, 0
        dsrl32 {tmp2}, {tmp2}, 0
        or {tmp}, {tmp}, {tmp2}
        dmtc0 {tmp}, ${cop0reg}
        nop
    ", gpr_lo = in(reg) (value as u32), gpr_hi = in(reg) ((value >> 32) as u32),
        tmp = out(reg) _, tmp2 = out(reg) _, cop0reg = const INDEX)
    }
}

pub fn index() -> u32 {
    const INDEX: u32 = RegisterIndex::Index as u32;
    unsafe { read_cop0::<INDEX>() }
}

pub unsafe fn set_index(value: u32) {
    const INDEX: u32 = RegisterIndex::Index as u32;
    unsafe { write_cop0::<INDEX>(value) }
}

pub fn random() -> u32 {
    const INDEX: u32 = RegisterIndex::Random as u32;
    unsafe { read_cop0::<INDEX>() }
}

pub fn set_random(value: u32) {
    const INDEX: u32 = RegisterIndex::Random as u32;
    unsafe { write_cop0::<INDEX>(value) }
}

pub fn entry_lo0() -> u32 {
    const INDEX: u32 = RegisterIndex::EntryLo0 as u32;
    unsafe { read_cop0::<INDEX>() }
}

pub fn entry_lo0_64() -> u64 {
    const INDEX: u32 = RegisterIndex::EntryLo0 as u32;
    unsafe { read_cop0_64::<INDEX>() }
}

pub unsafe fn set_entry_lo0(value: u32) {
    const INDEX: u32 = RegisterIndex::EntryLo0 as u32;
    unsafe { write_cop0::<INDEX>(value) }
}

pub unsafe fn set_entry_lo0_64(value: u64) {
    const INDEX: u32 = RegisterIndex::EntryLo0 as u32;
    unsafe { write_cop0_64::<INDEX>(value) }
}

pub fn entry_lo1() -> u32 {
    const INDEX: u32 = RegisterIndex::EntryLo1 as u32;
    unsafe { read_cop0::<INDEX>() }
}

pub fn entry_lo1_64() -> u64 {
    const INDEX: u32 = RegisterIndex::EntryLo1 as u32;
    unsafe { read_cop0_64::<INDEX>() }
}

pub unsafe fn set_entry_lo1(value: u32) {
    const INDEX: u32 = RegisterIndex::EntryLo1 as u32;
    unsafe { write_cop0::<INDEX>(value) }
}

pub unsafe fn set_entry_lo1_64(value: u64) {
    const INDEX: u32 = RegisterIndex::EntryLo1 as u32;
    unsafe { write_cop0_64::<INDEX>(value) }
}

pub fn context() -> Context {
    const INDEX: u32 = RegisterIndex::Context as u32;
    Context::new_with_raw_value(unsafe { read_cop0_64::<INDEX>() })
}

pub fn context_64() -> u64 {
    context().raw_value()
}

pub unsafe fn set_context_64(value: u64) {
    const INDEX: u32 = RegisterIndex::Context as u32;
    unsafe { write_cop0_64::<INDEX>(value) }
}

pub unsafe fn set_context_32(value: u32) {
    const INDEX: u32 = RegisterIndex::Context as u32;
    unsafe { write_cop0::<INDEX>(value) }
}

pub fn pagemask() -> u32 {
    const INDEX: u32 = RegisterIndex::PageMask as u32;
    unsafe { read_cop0::<INDEX>() }
}

pub unsafe fn set_pagemask(value: u32) {
    const INDEX: u32 = RegisterIndex::PageMask as u32;
    unsafe { write_cop0::<INDEX>(value) }
}

pub fn wired() -> u32 {
    const INDEX: u32 = RegisterIndex::Wired as u32;
    unsafe { read_cop0::<INDEX>() }
}

pub unsafe fn set_wired(value: u32) {
    const INDEX: u32 = RegisterIndex::Wired as u32;
    unsafe { write_cop0::<INDEX>(value) }
}

pub fn badvaddr() -> u64 {
    const INDEX: u32 = RegisterIndex::BadVAddr as u32;
    unsafe { read_cop0_64::<INDEX>() }
}

pub unsafe fn set_badvaddr(value: u64) {
    const INDEX: u32 = RegisterIndex::BadVAddr as u32;
    unsafe { write_cop0_64::<INDEX>(value) }
}

pub fn count() -> u32 {
    const INDEX: u32 = RegisterIndex::Count as u32;
    unsafe { read_cop0::<INDEX>() }
}

pub fn entry_hi() -> u64 {
    const INDEX: u32 = RegisterIndex::EntryHi as u32;
    unsafe { read_cop0_64::<INDEX>() }
}

pub unsafe fn set_entry_hi(value: u64) {
    const INDEX: u32 = RegisterIndex::EntryHi as u32;
    unsafe { write_cop0_64::<INDEX>(value) }
}

pub fn status() -> Status {
    const INDEX: u32 = RegisterIndex::Status as u32;
    Status::new_with_raw_value(unsafe { read_cop0::<INDEX>() })
}

pub unsafe fn set_status(value: Status) {
    const INDEX: u32 = RegisterIndex::Status as u32;
    unsafe { write_cop0::<INDEX>(value.raw_value()) }
}

pub fn status_64() -> u64 {
    const INDEX: u32 = RegisterIndex::Status as u32;
    unsafe { read_cop0_64::<INDEX>() }
}

pub unsafe fn set_status_64(value: u64) {
    const INDEX: u32 = RegisterIndex::Status as u32;
    unsafe { write_cop0_64::<INDEX>(value) }
}

pub fn exceptpc() -> u64 {
    const INDEX: u32 = RegisterIndex::ExceptPC as u32;
    unsafe { read_cop0_64::<INDEX>() }
}

pub unsafe fn set_exceptpc(value: u64) {
    const INDEX: u32 = RegisterIndex::ExceptPC as u32;
    unsafe { write_cop0_64::<INDEX>(value) }
}

pub fn previd() -> u32 {
    const INDEX: u32 = RegisterIndex::PRevID as u32;
    unsafe { read_cop0::<INDEX>() }
}

pub unsafe fn set_previd(value: u32) {
    const INDEX: u32 = RegisterIndex::PRevID as u32;
    unsafe { write_cop0::<INDEX>(value) }
}

pub fn config() -> u32 {
    const INDEX: u32 = RegisterIndex::Config as u32;
    unsafe { read_cop0::<INDEX>() }
}

pub unsafe fn set_config(value: u32) {
    const INDEX: u32 = RegisterIndex::Config as u32;
    unsafe { write_cop0::<INDEX>(value) }
}

pub fn lladdr() -> u64 {
    const INDEX: u32 = RegisterIndex::LLAddr as u32;
    unsafe { read_cop0_64::<INDEX>() }
}

pub unsafe fn set_lladdr(value: u64) {
    const INDEX: u32 = RegisterIndex::LLAddr as u32;
    unsafe { write_cop0_64::<INDEX>(value) }
}

pub fn xcontext() -> XContext {
    const INDEX: u32 = RegisterIndex::XContext as u32;
    XContext::new_with_raw_value(unsafe { read_cop0_64::<INDEX>() })
}

pub fn xcontext_64() -> u64 {
    xcontext().raw_value()
}

pub unsafe fn set_xcontext_64(value: u64) {
    const INDEX: u32 = RegisterIndex::XContext as u32;
    unsafe { write_cop0_64::<INDEX>(value) }
}

pub unsafe fn set_xcontext_32(value: u32) {
    const INDEX: u32 = RegisterIndex::XContext as u32;
    unsafe { write_cop0::<INDEX>(value) }
}

pub fn errorepc() -> u64 {
    const INDEX: u32 = RegisterIndex::ErrorEPC as u32;
    unsafe { read_cop0_64::<INDEX>() }
}

pub unsafe fn set_errorepc(value: u64) {
    const INDEX: u32 = RegisterIndex::ErrorEPC as u32;
    unsafe { write_cop0_64::<INDEX>(value) }
}

pub unsafe fn tlbwi() {
    unsafe {
        asm!("tlbwi; nop; nop;")
    }
}

pub unsafe fn tlbr() {
    unsafe {
        asm!("tlbr; nop; nop;")
    }
}

pub fn tlbp() {
    unsafe {
        asm!("nop; tlbp; nop;")
    }
}

pub unsafe fn write_tlb(index: u32, pagemask: u32, entry_lo0: u32, entry_lo1: u32, entry_hi: u64) {
    unsafe {
        set_index(index);
        set_entry_lo0(entry_lo0);
        set_entry_lo1(entry_lo1);
        set_entry_hi(entry_hi);
        set_pagemask(pagemask);
        tlbwi();
    }
}

pub unsafe fn clear_tlb() {
    // use entry_hi.asid=1 to ensure nothing matches
    unsafe {
        set_entry_lo0(0);
        set_entry_lo1(0);
        set_entry_hi(make_entry_hi(1, u27::new(0), u2::new(0)));
        set_pagemask(0);
        for i in 0..32 {
            set_index(i);
            tlbwi();
        }
        set_entry_hi(0);
    }
}

pub fn make_entry_lo(global: bool, valid: bool, dirty: bool, coherency: u8, pfn: u32) -> u32 {
    assert!(coherency <= 7);
    assert!(pfn <= 0xFFFFFF);
    (global as u32) |
        ((valid as u32) << 1) |
        ((dirty as u32) << 2) |
        ((coherency as u32) << 3) |
        (pfn << 6)
}

pub fn make_entry_hi(asid: u8, vpn: u27, r: u2) -> u64 {
    (asid as u64) |
        ((vpn.value() as u64) << 13) |
        ((r.value() as u64) << 62)
}

#[inline(always)]
pub unsafe fn cache<const OP: u8, const OFFSET: u16>(location: usize) {
    unsafe {
        asm!(".set noat
        cache {op}, {offset} ({gpr})",
        gpr = in(reg) location,
        offset = const OFFSET,
        op = const OP)
    }
}

/// Like cache, but accepts a u64 address
#[inline(always)]
pub unsafe fn cache64<const OP: u8, const OFFSET: u16>(location: u64) {
    unsafe {
        asm!("
            .set noat
            ld {temp}, 0 ({location})
            cache {op}, {offset} ({temp})",
        location = in(reg) &location,
        offset = const OFFSET,
        op = const OP,
        temp = out(reg) _)
    }
}

// Fires a cop2 usable exception and handles it itself. Afterwards, Cause.copindex will be set to 2
pub fn preset_cause_to_copindex2() -> Result<(), String> {
    // Fire a COP2 unusable exception. This is to write something into Cause.copindex so that we can see whether it gets overwritten
    let temp_context = expect_exception(CauseException::CopUnusable, 1, || {
        unsafe { asm!("MFC2 $0, $0"); }
        Ok(())
    })?;
    if temp_context.cause != Cause::new().with_coprocessor_error(u2::new(2)).with_exception(CauseException::CopUnusable) {
        return Err("Presetting Cause.copindex to 2 failed".to_string());
    };

    Ok(())
}
