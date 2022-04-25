use core::arch::asm;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

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

crate::enum_str! {
    #[derive(FromPrimitive, Copy, Clone, PartialEq, Eq, Debug)]
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
        CpU = 11,
        Ov = 12,
        Tr = 13,
        VirtualCoherencyInstructionFetch = 14,
        FPE = 15,
        Watch = 23,
    }
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

pub fn context_64() -> u64 {
    const INDEX: u32 = RegisterIndex::Context as u32;
    unsafe { read_cop0_64::<INDEX>() }
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

pub fn status() -> u32 {
    const INDEX: u32 = RegisterIndex::Status as u32;
    unsafe { read_cop0::<INDEX>() }
}

pub unsafe fn set_status(value: u32) {
    const INDEX: u32 = RegisterIndex::Status as u32;
    unsafe { write_cop0::<INDEX>(value) }
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

pub fn previd() -> u64 {
    const INDEX: u32 = RegisterIndex::PRevID as u32;
    unsafe { read_cop0_64::<INDEX>() }
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

pub fn xcontext_64() -> u64 {
    const INDEX: u32 = RegisterIndex::XContext as u32;
    unsafe { read_cop0_64::<INDEX>() }
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

pub fn cause_extract_exception(value: u32) -> Result<CauseException, u8> {
    let code = ((value >> 2) & 0b11111) as u8;
    FromPrimitive::from_u8(code).ok_or(code)
}

pub fn cause_extract_delay(value: u32) -> bool {
    (value & 0x80000000) != 0
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
    for i in 0..32 {
        unsafe { write_tlb(i, 0, 0, 0, 0); }
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

pub fn make_entry_hi(asid: u8, vpn: u32) -> u64 {
    assert!(vpn <= 0x1FFFFF);
    (asid as u64) |
        ((vpn as u64) << 13)
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