use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::any::Any;
use arbitrary_int::{u2, u5, u27};

use crate::assembler::{Assembler, FR, GPR};
use crate::cop0::{self, CauseException, Status, StatusKSU, make_entry_hi, make_entry_lo};
use crate::tests::privilege::{run_mode_program, run_mode_program_with_cop0};
use crate::tests::soft_asserts::{soft_assert_eq, soft_assert_eq2};
use crate::tests::{Level, Test};
use crate::uncached_memory::UncachedHeapMemory;

const PAGE_BYTES: usize = 4096;
const PAGE_WORDS: usize = PAGE_BYTES / 4;
const TOTAL_PAGES: usize = 4;
const TOTAL_WORDS: usize = PAGE_WORDS * TOTAL_PAGES;

const CODE_CACHED_BASE: u32 = 0x0002_0000;
const CODE_UNCACHED_BASE: u32 = CODE_CACHED_BASE + PAGE_BYTES as u32;
const DATA_CACHED_BASE: u32 = 0x0002_2000;
const DATA_UNCACHED_BASE: u32 = DATA_CACHED_BASE + PAGE_BYTES as u32;

const DATA_OFFSET: usize = 0x0200;
const STORE_OFFSET: usize = 0x0300;

const RE_FIXTURE: [u8; 16] = [
    0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x10, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09,
];

#[derive(Copy, Clone)]
enum MemoryKind {
    Cached,
    Uncached,
}

impl MemoryKind {
    fn name(self) -> &'static str {
        match self {
            MemoryKind::Cached => "cached",
            MemoryKind::Uncached => "uncached",
        }
    }

    fn data_base(self) -> u32 {
        match self {
            MemoryKind::Cached => DATA_CACHED_BASE,
            MemoryKind::Uncached => DATA_UNCACHED_BASE,
        }
    }
}

fn make_cop1_move(rs: u8, rt: GPR, fs: FR) -> u32 {
    match rs {
        0 => Assembler::make_mfc1(rt, fs),
        1 => Assembler::make_dmfc1(rt, fs),
        4 => Assembler::make_mtc1(rt, fs),
        5 => Assembler::make_dmtc1(rt, fs),
        _ => panic!("unsupported cop1 move"),
    }
}

fn load_u32_sequence(register: GPR, value: u32) -> [u32; 2] {
    [
        Assembler::make_lui(register, (value >> 16) as u16),
        Assembler::make_ori(register, register, value as u16),
    ]
}

fn load_u64_sequence(target: GPR, temp: GPR, value: u64) -> [u32; 8] {
    let hi = (value >> 32) as u32;
    let lo = value as u32;
    [
        Assembler::make_lui(target, (hi >> 16) as u16),
        Assembler::make_ori(target, target, hi as u16),
        Assembler::make_dsll32(target, target, 0),
        Assembler::make_lui(temp, (lo >> 16) as u16),
        Assembler::make_ori(temp, temp, lo as u16),
        Assembler::make_dsll32(temp, temp, 0),
        Assembler::make_dsrl32(temp, temp, 0),
        Assembler::make_or(target, target, temp),
    ]
}

fn re_fetch_encode(program: &[u32]) -> Vec<u32> {
    let mut encoded = program.to_vec();
    if encoded.len() & 1 != 0 {
        encoded.push(Assembler::make_nop());
    }
    let mut i = 0usize;
    while i + 1 < encoded.len() {
        encoded.swap(i, i + 1);
        i += 2;
    }
    encoded
}

fn re1_then_re0_encode(re1_program: &[u32], re0_program: &[u32]) -> Vec<u32> {
    let mut encoded = re_fetch_encode(re1_program);
    encoded.extend_from_slice(re0_program);
    encoded
}

fn reverse_endian_effective_address(vaddr: u32, bytes: usize) -> u32 {
    match bytes {
        8 => vaddr,
        4 => vaddr ^ 4,
        2 => vaddr ^ 6,
        1 => vaddr ^ 7,
        _ => panic!("unsupported size"),
    }
}

fn read_be(memory: &[u8], offset: usize, bytes: usize) -> u64 {
    let mut value = 0u64;
    for byte in 0..bytes {
        value = (value << 8) | memory[offset + byte] as u64;
    }
    value
}

fn sign_extend(value: u64, bits: u8) -> u64 {
    let shift = 64 - bits as u32;
    (((value << shift) as i64) >> shift) as u64
}

struct EndianHarness {
    backing: UncachedHeapMemory<u32>,
}

impl EndianHarness {
    fn new() -> Self {
        Self { backing: UncachedHeapMemory::<u32>::new_with_align(TOTAL_WORDS, PAGE_BYTES * TOTAL_PAGES) }
    }

    fn setup_mappings(&mut self) {
        let paddr0 = self.backing.start_phyiscal();
        let paddr1 = paddr0 + PAGE_BYTES;
        let paddr2 = paddr1 + PAGE_BYTES;
        let paddr3 = paddr2 + PAGE_BYTES;
        unsafe {
            cop0::clear_tlb();
            cop0::set_context_64(0);
            cop0::set_xcontext_64(0);
            cop0::write_tlb(
                20,
                0,
                make_entry_lo(true, true, true, 0, (paddr0 >> 12) as u32),
                make_entry_lo(true, true, true, 2, (paddr1 >> 12) as u32),
                make_entry_hi(0, u27::extract_u64(CODE_CACHED_BASE as u64, 13), u2::new(0)),
            );
            cop0::write_tlb(
                21,
                0,
                make_entry_lo(true, true, true, 0, (paddr2 >> 12) as u32),
                make_entry_lo(true, true, true, 2, (paddr3 >> 12) as u32),
                make_entry_hi(0, u27::extract_u64(DATA_CACHED_BASE as u64, 13), u2::new(0)),
            );
        }
    }

    fn ptr_u8(&self) -> *mut u8 {
        self.backing.as_ptr().cast::<u8>()
    }

    fn page_offset(page_index: usize) -> usize {
        page_index * PAGE_BYTES
    }

    fn code_page_index(cached: bool) -> usize {
        if cached { 0 } else { 1 }
    }

    fn data_page_index(kind: MemoryKind) -> usize {
        match kind {
            MemoryKind::Cached => 2,
            MemoryKind::Uncached => 3,
        }
    }

    fn fill_fixture(&mut self) {
        for kind in [MemoryKind::Cached, MemoryKind::Uncached] {
            let base = Self::page_offset(Self::data_page_index(kind)) + DATA_OFFSET;
            for (index, value) in RE_FIXTURE.iter().enumerate() {
                unsafe {
                    self.ptr_u8().add(base + index).write_volatile(*value);
                }
            }
        }
    }

    fn clear_store_area(&mut self) {
        for kind in [MemoryKind::Cached, MemoryKind::Uncached] {
            let base = Self::page_offset(Self::data_page_index(kind)) + STORE_OFFSET;
            for offset in 0..32 {
                unsafe {
                    self.ptr_u8().add(base + offset).write_volatile(0xaa);
                }
            }
        }
    }

    fn write_page_bytes(&mut self, page_index: usize, start: usize, bytes: &[u8]) {
        let base = Self::page_offset(page_index) + start;
        for (index, value) in bytes.iter().enumerate() {
            unsafe {
                self.ptr_u8().add(base + index).write_volatile(*value);
            }
        }
    }

    fn read_code_bytes(&self, cached: bool, start: usize, len: usize) -> [u8; 32] {
        let mut out = [0u8; 32];
        let page_index = Self::code_page_index(cached);
        let offset = Self::page_offset(page_index) + start;
        for index in 0..len {
            out[index] = unsafe { self.ptr_u8().add(offset + index).read_volatile() };
        }
        out
    }

    fn write_code_bytes(&mut self, cached: bool, start: usize, bytes: &[u8]) {
        self.write_page_bytes(Self::code_page_index(cached), start, bytes);
    }

    fn write_program(&mut self, cached_code: bool, program: &[u32]) -> u32 {
        let page_index = Self::code_page_index(cached_code);
        let page_word_base = Self::page_offset(page_index) / 4;
        for (index, instruction) in program.iter().enumerate() {
            self.backing.write(page_word_base + index, *instruction);
        }
        let code_base = if cached_code { CODE_CACHED_BASE } else { CODE_UNCACHED_BASE };
        if cached_code {
            self.flush_icache(code_base, program.len() * 4 + 32);
        }
        code_base
    }

    fn flush_icache(&self, base: u32, len: usize) {
        unsafe {
            cop0::dcache_index_writeback_invalidate_range(base as usize, len);
            cop0::icache_index_invalidate_range(base as usize, len);
            cop0::sync();
        }
    }

    fn writeback_dcache(&self, base: u32, len: usize) {
        unsafe {
            cop0::dcache_index_writeback_invalidate_range(base as usize, len);
            cop0::sync();
        }
    }

    fn read_page_bytes(&self, kind: MemoryKind, start: usize, len: usize) -> [u8; 32] {
        let mut out = [0u8; 32];
        let offset = Self::page_offset(Self::data_page_index(kind)) + start;
        for index in 0..len {
            out[index] = unsafe { self.ptr_u8().add(offset + index).read_volatile() };
        }
        out
    }
}

fn run_re_program(program: &[u32], cached_code: bool) -> Result<crate::exception_handler::ExceptionContext, String> {
    let mut harness = EndianHarness::new();
    harness.setup_mappings();
    harness.fill_fixture();
    harness.clear_store_area();
    let mut wrapped = Vec::with_capacity(program.len() + 1);
    wrapped.extend_from_slice(program);
    wrapped.push(Assembler::make_syscall(0x2e1));
    let encoded = re_fetch_encode(&wrapped);
    let entry = harness.write_program(cached_code, &encoded);
    run_mode_program(StatusKSU::User, true, true, entry, CauseException::Sys, 1)
}

fn run_re_program_labeled(label: &str, program: &[u32], cached_code: bool)
    -> Result<crate::exception_handler::ExceptionContext, String> {
    run_re_program(program, cached_code).map_err(|e| format!("{}: {}", label, e))
}

fn run_re_entry_labeled(label: &str, entry: u32) -> Result<crate::exception_handler::ExceptionContext, String> {
    run_mode_program(StatusKSU::User, true, true, entry, CauseException::Sys, 1)
        .map_err(|e| format!("{}: {}", label, e))
}

pub struct ReExecuteCodePaths;

impl Test for ReExecuteCodePaths {
    fn name(&self) -> &str { "RE user mode execute from uncached and cached code pages" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let i1 = Assembler::make_addiu(GPR::V0, GPR::R0, 1);
        let i2 = Assembler::make_sll(GPR::V0, GPR::V0, 1);
        let i3 = Assembler::make_addiu(GPR::V0, GPR::V0, 3);
        let i4 = Assembler::make_syscall(0x1f3);

        let swapped = [i2, i1, i4, i3];

        let mut harness = EndianHarness::new();
        harness.setup_mappings();
        harness.fill_fixture();
        harness.clear_store_area();

        let cached_entry = harness.write_program(true, &swapped);
        let uncached_entry = harness.write_program(false, &swapped);

        let ctx_cached = run_re_entry_labeled("execute cached code path", cached_entry)?;
        let ctx_uncached = run_re_entry_labeled("execute uncached code path", uncached_entry)?;
        soft_assert_eq(ctx_cached.v0, 5, "cached code path result")?;
        soft_assert_eq(ctx_uncached.v0, 5, "uncached code path result")?;
        Ok(())
    }
}

pub struct ReLoadMatrix;

impl Test for ReLoadMatrix {
    fn name(&self) -> &str { "RE user mode loads: cached/uncached + COP1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        struct LoadCase {
            name: &'static str,
            emit: fn(GPR, i16, GPR) -> u32,
            bytes: usize,
            signed: bool,
            cop1: bool,
            cop1_64: bool,
        }
        let load_cases = [
            LoadCase { name: "LB", emit: Assembler::make_lb, bytes: 1, signed: true, cop1: false, cop1_64: false },
            LoadCase { name: "LBU", emit: Assembler::make_lbu, bytes: 1, signed: false, cop1: false, cop1_64: false },
            LoadCase { name: "LH", emit: Assembler::make_lh, bytes: 2, signed: true, cop1: false, cop1_64: false },
            LoadCase { name: "LHU", emit: Assembler::make_lhu, bytes: 2, signed: false, cop1: false, cop1_64: false },
            LoadCase { name: "LW", emit: Assembler::make_lw, bytes: 4, signed: true, cop1: false, cop1_64: false },
            LoadCase { name: "LWU", emit: Assembler::make_lwu, bytes: 4, signed: false, cop1: false, cop1_64: false },
            LoadCase { name: "LD", emit: Assembler::make_ld, bytes: 8, signed: false, cop1: false, cop1_64: false },
            LoadCase { name: "LWC1", emit: Assembler::make_lwc1, bytes: 4, signed: true, cop1: true, cop1_64: false },
            LoadCase { name: "LDC1", emit: Assembler::make_ldc1, bytes: 8, signed: false, cop1: true, cop1_64: true },
        ];

        for memory_kind in [MemoryKind::Cached, MemoryKind::Uncached] {
            for case in load_cases.iter() {
                let address = memory_kind.data_base() + DATA_OFFSET as u32;
                let mut program = vec![
                    Assembler::make_lui(GPR::T0, (address >> 16) as u16),
                    Assembler::make_ori(GPR::T0, GPR::T0, address as u16),
                    (case.emit)(GPR::T1, 0, GPR::T0),
                ];
                if case.cop1 {
                    if case.cop1_64 {
                        program.push(make_cop1_move(1, GPR::T1, FR::F9));
                    } else {
                        program.push(make_cop1_move(0, GPR::T1, FR::F9));
                    }
                }
                let label = format!(
                    "load {} {} addr={:#010x} fixture={:02x?}",
                    case.name,
                    memory_kind.name(),
                    address,
                    RE_FIXTURE
                );
                let context = run_re_program_labeled(&label, &program, true)?;
                let actual = context.t1;
                let effective = reverse_endian_effective_address(address, case.bytes) - address;
                let raw = read_be(&RE_FIXTURE, effective as usize, case.bytes);
                let expected = if case.signed {
                    sign_extend(raw, (case.bytes * 8) as u8)
                } else {
                    raw
                };
                soft_assert_eq2(actual, expected, || format!("{} from {}", case.name, memory_kind.name()))?;
            }
        }
        Ok(())
    }
}

pub struct ReStoreMatrix;

impl Test for ReStoreMatrix {
    fn name(&self) -> &str { "RE user mode stores: cached/uncached + COP1" }

    fn level(&self) -> Level { Level::BasicFunctionality }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        struct StoreCase {
            name: &'static str,
            emit: fn(GPR, i16, GPR) -> u32,
            rt: GPR,
            bytes: usize,
            value: u64,
            cop1: bool,
            cop1_64: bool,
        }
        let store_cases = [
            StoreCase { name: "SB", emit: Assembler::make_sb, rt: GPR::T1, bytes: 1, value: 0x81, cop1: false, cop1_64: false },
            StoreCase { name: "SH", emit: Assembler::make_sh, rt: GPR::T1, bytes: 2, value: 0x92a3, cop1: false, cop1_64: false },
            StoreCase { name: "SW", emit: Assembler::make_sw, rt: GPR::T1, bytes: 4, value: 0xb4c5d6e7, cop1: false, cop1_64: false },
            StoreCase { name: "SD", emit: Assembler::make_sd, rt: GPR::T1, bytes: 8, value: 0x1122334455667788, cop1: false, cop1_64: false },
            StoreCase { name: "SWC1", emit: Assembler::make_swc1, rt: GPR::T0, bytes: 4, value: 0x89abcdef, cop1: true, cop1_64: false },
            StoreCase { name: "SDC1", emit: Assembler::make_sdc1, rt: GPR::T0, bytes: 8, value: 0x0123456789abcdef, cop1: true, cop1_64: true },
        ];

        for memory_kind in [MemoryKind::Cached, MemoryKind::Uncached] {
            for case in store_cases.iter() {
                let mut harness = EndianHarness::new();
                harness.setup_mappings();
                harness.fill_fixture();
                harness.clear_store_area();

                let address = memory_kind.data_base() + STORE_OFFSET as u32;
                let mut program = vec![
                    Assembler::make_lui(GPR::T0, (address >> 16) as u16),
                    Assembler::make_ori(GPR::T0, GPR::T0, address as u16),
                ];
                if case.bytes == 8 {
                    program.extend_from_slice(&load_u64_sequence(GPR::T1, GPR::T2, case.value));
                } else {
                    let seq = load_u32_sequence(GPR::T1, case.value as u32);
                    program.push(seq[0]);
                    program.push(seq[1]);
                }
                if case.cop1 {
                    if case.cop1_64 {
                        program.push(make_cop1_move(5, GPR::T1, FR::F8));
                    } else {
                        program.push(make_cop1_move(4, GPR::T1, FR::F8));
                    }
                }
                program.push((case.emit)(case.rt, 0, GPR::T0));
                program.push(Assembler::make_syscall(0x2e2));
                let encoded = re_fetch_encode(&program);
                let entry = harness.write_program(true, &encoded);
                let label = format!(
                    "store {} {} addr={:#010x} value={:#018x}",
                    case.name,
                    memory_kind.name(),
                    address,
                    case.value
                );
                let _ = run_re_entry_labeled(&label, entry)?;
                if matches!(memory_kind, MemoryKind::Cached) {
                    harness.writeback_dcache(address, 64);
                }
                let effective = reverse_endian_effective_address(address, case.bytes);
                let offset = (effective - memory_kind.data_base()) as usize;
                let bytes = harness.read_page_bytes(memory_kind, offset, case.bytes);
                let expected = case.value.to_be_bytes();
                for i in 0..case.bytes {
                    soft_assert_eq2(bytes[i], expected[8 - case.bytes + i], || {
                        format!("{} byte {} on {}", case.name, i, memory_kind.name())
                    })?;
                }
            }
        }
        Ok(())
    }
}

pub struct ReDcacheLineToggle;

impl Test for ReDcacheLineToggle {
    fn name(&self) -> &str { "RE dcache single-pass: fill(RE=1) then writeback(RE=0)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut harness = EndianHarness::new();
        harness.setup_mappings();
        harness.fill_fixture();
        harness.clear_store_area();

        let address = DATA_CACHED_BASE + DATA_OFFSET as u32;
        let line_before = harness.read_page_bytes(MemoryKind::Cached, DATA_OFFSET, cop0::DCACHE_LINE_BYTES);
        let status_re0 = Status::DEFAULT
            .with_ksu(StatusKSU::User)
            .with_reverse_endian(false)
            .with_exl(true)
            .with_erl(false)
            .with_kx(true)
            .with_sx(true)
            .with_ux(true)
            .raw_value();
        let status_re0_seq = load_u32_sequence(GPR::T2, status_re0);
        let re1_program = vec![
            Assembler::make_lui(GPR::T0, (address >> 16) as u16),
            Assembler::make_ori(GPR::T0, GPR::T0, address as u16),
            Assembler::make_lw(GPR::T1, 0, GPR::T0),
            status_re0_seq[0],
            status_re0_seq[1],
            Assembler::make_mtc0(GPR::T2, u5::new(12)),
            Assembler::make_nop(),
            Assembler::make_nop(),
            Assembler::make_nop(),
            Assembler::make_nop(),
        ];
        let re0_program = vec![
            Assembler::make_addiu(GPR::T1, GPR::R0, 0x005a),
            Assembler::make_sb(GPR::T1, 0, GPR::T0),
            Assembler::make_cache(cop0::DCACHE_HIT_WRITEBACK, 0, GPR::T0),
            Assembler::make_syscall(0x2f1),
        ];
        let program = re1_then_re0_encode(&re1_program, &re0_program);
        let entry = harness.write_program(true, &program);
        run_mode_program_with_cop0(StatusKSU::User, true, true, entry, CauseException::Sys, 1, true)
            .map_err(|e| format!("dcache single-pass RE1->RE0: {}", e))?;
        let line_after = harness.read_page_bytes(MemoryKind::Cached, DATA_OFFSET, cop0::DCACHE_LINE_BYTES);
        for i in 1..cop0::DCACHE_LINE_BYTES {
            soft_assert_eq2(line_after[i], line_before[i], || format!("dcache byte {} changed after RE1->RE0", i))?;
        }
        Ok(())
    }
}

pub struct ReIcacheLineToggle;

impl Test for ReIcacheLineToggle {
    fn name(&self) -> &str { "RE icache single-pass: fill(RE=1) then writeback(RE=0)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        let mut harness = EndianHarness::new();
        harness.setup_mappings();
        harness.fill_fixture();
        harness.clear_store_area();

        let line_offset = 0x0200usize;
        let address = CODE_CACHED_BASE + line_offset as u32;
        let mut line_before = [0u8; 32];
        for i in 0..cop0::ICACHE_LINE_BYTES {
            line_before[i] = 0x40 + i as u8;
        }
        harness.write_code_bytes(true, line_offset, &line_before[..cop0::ICACHE_LINE_BYTES]);
        let status_re0 = Status::DEFAULT
            .with_ksu(StatusKSU::User)
            .with_reverse_endian(false)
            .with_exl(true)
            .with_erl(false)
            .with_kx(true)
            .with_sx(true)
            .with_ux(true)
            .raw_value();
        let status_re0_seq = load_u32_sequence(GPR::T2, status_re0);
        let re1_program = vec![
            Assembler::make_lui(GPR::T0, (address >> 16) as u16),
            Assembler::make_ori(GPR::T0, GPR::T0, address as u16),
            Assembler::make_cache(cop0::ICACHE_FILL, 0, GPR::T0),
            status_re0_seq[0],
            status_re0_seq[1],
            Assembler::make_mtc0(GPR::T2, u5::new(12)),
            Assembler::make_nop(),
            Assembler::make_nop(),
            Assembler::make_nop(),
            Assembler::make_nop(),
        ];
        let re0_program = vec![
            Assembler::make_cache(cop0::ICACHE_HIT_WRITEBACK, 0, GPR::T0),
            Assembler::make_syscall(0x2f3),
        ];
        let program = re1_then_re0_encode(&re1_program, &re0_program);
        let entry = harness.write_program(true, &program);
        run_mode_program_with_cop0(StatusKSU::User, true, true, entry, CauseException::Sys, 1, true)
            .map_err(|e| format!("icache single-pass RE1->RE0: {}", e))?;
        let line_after = harness.read_code_bytes(true, line_offset, cop0::ICACHE_LINE_BYTES);
        for i in 0..cop0::ICACHE_LINE_BYTES {
            soft_assert_eq2(line_after[i], line_before[i], || format!("icache byte {} changed after RE1->RE0", i))?;
        }
        Ok(())
    }
}

fn run_unaligned_32bit_load_pairs(memory_kind: MemoryKind) -> Result<(), String> {
    let base = memory_kind.data_base() + DATA_OFFSET as u32;
    let prog_lw = vec![
        Assembler::make_lui(GPR::T0, (base >> 16) as u16),
        Assembler::make_ori(GPR::T0, GPR::T0, base as u16),
        Assembler::make_lw(GPR::T1, 0, GPR::T0),
    ];
    let ctx_lw = run_re_program_labeled(&format!("unaligned base LW {}", memory_kind.name()), &prog_lw, true)?;
    let prog_lwl_lwr = vec![
        Assembler::make_lui(GPR::T0, (base >> 16) as u16),
        Assembler::make_ori(GPR::T0, GPR::T0, base as u16),
        Assembler::make_lui(GPR::T1, 0xbeef),
        Assembler::make_ori(GPR::T1, GPR::T1, 0x0102),
        Assembler::make_lwl(GPR::T1, 3, GPR::T0),
        Assembler::make_lwr(GPR::T1, 0, GPR::T0),
    ];
    let ctx_lwl_lwr = run_re_program_labeled(
        &format!("unaligned LWL/LWR {}", memory_kind.name()),
        &prog_lwl_lwr,
        true,
    )?;
    soft_assert_eq2(
        ctx_lwl_lwr.t1 as u32,
        ctx_lw.t1 as u32,
        || format!("LWL/LWR pair on {}", memory_kind.name()),
    )?;
    Ok(())
}

fn run_unaligned_64bit_load_pairs(memory_kind: MemoryKind) -> Result<(), String> {
    let base = memory_kind.data_base() + DATA_OFFSET as u32;
    let prog_ld = vec![
        Assembler::make_lui(GPR::T0, (base >> 16) as u16),
        Assembler::make_ori(GPR::T0, GPR::T0, base as u16),
        Assembler::make_ld(GPR::T1, 0, GPR::T0),
    ];
    let ctx_ld = run_re_program_labeled(&format!("unaligned base LD {}", memory_kind.name()), &prog_ld, true)?;
    let prog_ldl_ldr = vec![
        Assembler::make_lui(GPR::T0, (base >> 16) as u16),
        Assembler::make_ori(GPR::T0, GPR::T0, base as u16),
        Assembler::make_lui(GPR::T1, 0xbeef),
        Assembler::make_ori(GPR::T1, GPR::T1, 0x0102),
        Assembler::make_ldl(GPR::T1, 7, GPR::T0),
        Assembler::make_ldr(GPR::T1, 0, GPR::T0),
    ];
    let ctx_ldl_ldr = run_re_program_labeled(
        &format!("unaligned LDL/LDR {}", memory_kind.name()),
        &prog_ldl_ldr,
        true,
    )?;
    soft_assert_eq2(ctx_ldl_ldr.t1, ctx_ld.t1, || format!("LDL/LDR pair on {}", memory_kind.name()))?;
    Ok(())
}

fn run_unaligned_32bit_store_pairs(memory_kind: MemoryKind) -> Result<(), String> {
    let store_address = memory_kind.data_base() + STORE_OFFSET as u32;
    let mut harness_sw = EndianHarness::new();
    harness_sw.setup_mappings();
    harness_sw.fill_fixture();
    harness_sw.clear_store_area();
    let program_sw = vec![
        Assembler::make_lui(GPR::T0, (store_address >> 16) as u16),
        Assembler::make_ori(GPR::T0, GPR::T0, store_address as u16),
        Assembler::make_lui(GPR::T1, 0x5060),
        Assembler::make_ori(GPR::T1, GPR::T1, 0x7080),
        Assembler::make_sw(GPR::T1, 0, GPR::T0),
        Assembler::make_syscall(0x2e3),
    ];
    let encoded_sw = re_fetch_encode(&program_sw);
    let entry_sw = harness_sw.write_program(true, &encoded_sw);
    let _ = run_re_entry_labeled(&format!("unaligned SW baseline {}", memory_kind.name()), entry_sw)?;
    if matches!(memory_kind, MemoryKind::Cached) {
        harness_sw.writeback_dcache(store_address, 64);
    }
    let eff_sw = reverse_endian_effective_address(store_address, 4);
    let bytes_sw = harness_sw.read_page_bytes(memory_kind, (eff_sw - memory_kind.data_base()) as usize, 8);
    let mut harness_swl_swr = EndianHarness::new();
    harness_swl_swr.setup_mappings();
    harness_swl_swr.fill_fixture();
    harness_swl_swr.clear_store_area();
    let mut program_swl_swr = vec![
        Assembler::make_lui(GPR::T0, (store_address >> 16) as u16),
        Assembler::make_ori(GPR::T0, GPR::T0, store_address as u16),
        Assembler::make_lui(GPR::T1, 0x5060),
        Assembler::make_ori(GPR::T1, GPR::T1, 0x7080),
        Assembler::make_swl(GPR::T1, 3, GPR::T0),
        Assembler::make_swr(GPR::T1, 0, GPR::T0),
    ];
    program_swl_swr.push(Assembler::make_syscall(0x2e4));
    let encoded_swl_swr = re_fetch_encode(&program_swl_swr);
    let entry_swl_swr = harness_swl_swr.write_program(true, &encoded_swl_swr);
    let _ = run_re_entry_labeled(&format!("unaligned SWL/SWR {}", memory_kind.name()), entry_swl_swr)?;
    if matches!(memory_kind, MemoryKind::Cached) {
        harness_swl_swr.writeback_dcache(store_address, 64);
    }
    let eff_swl_swr = reverse_endian_effective_address(store_address, 4);
    let bytes_swl_swr = harness_swl_swr.read_page_bytes(memory_kind, (eff_swl_swr - memory_kind.data_base()) as usize, 8);
    for i in 0..8 {
        soft_assert_eq2(bytes_swl_swr[i], bytes_sw[i], || {
            format!("SWL/SWR pair byte {} on {}", i, memory_kind.name())
        })?;
    }
    Ok(())
}

fn run_unaligned_64bit_store_pairs(memory_kind: MemoryKind) -> Result<(), String> {
    let store_address = memory_kind.data_base() + STORE_OFFSET as u32;
    let store_value = 0x1020304050607080u64;
    let mut harness_sd = EndianHarness::new();
    harness_sd.setup_mappings();
    harness_sd.fill_fixture();
    harness_sd.clear_store_area();
    let mut program_sd = vec![
        Assembler::make_lui(GPR::T0, (store_address >> 16) as u16),
        Assembler::make_ori(GPR::T0, GPR::T0, store_address as u16),
    ];
    program_sd.extend_from_slice(&load_u64_sequence(GPR::T1, GPR::T2, store_value));
    program_sd.push(Assembler::make_sd(GPR::T1, 0, GPR::T0));
    program_sd.push(Assembler::make_syscall(0x2e5));
    let encoded_sd = re_fetch_encode(&program_sd);
    let entry_sd = harness_sd.write_program(true, &encoded_sd);
    let _ = run_re_entry_labeled(&format!("unaligned SD baseline {}", memory_kind.name()), entry_sd)?;
    if matches!(memory_kind, MemoryKind::Cached) {
        harness_sd.writeback_dcache(store_address, 64);
    }
    let eff_sd = reverse_endian_effective_address(store_address, 8);
    let bytes_sd = harness_sd.read_page_bytes(memory_kind, (eff_sd - memory_kind.data_base()) as usize, 16);
    let mut harness_sdl_sdr = EndianHarness::new();
    harness_sdl_sdr.setup_mappings();
    harness_sdl_sdr.fill_fixture();
    harness_sdl_sdr.clear_store_area();
    let mut program_sdl_sdr = vec![
        Assembler::make_lui(GPR::T0, (store_address >> 16) as u16),
        Assembler::make_ori(GPR::T0, GPR::T0, store_address as u16),
    ];
    program_sdl_sdr.extend_from_slice(&load_u64_sequence(GPR::T1, GPR::T2, store_value));
    program_sdl_sdr.push(Assembler::make_sdl(GPR::T1, 7, GPR::T0));
    program_sdl_sdr.push(Assembler::make_sdr(GPR::T1, 0, GPR::T0));
    program_sdl_sdr.push(Assembler::make_syscall(0x2e6));
    let encoded_sdl_sdr = re_fetch_encode(&program_sdl_sdr);
    let entry_sdl_sdr = harness_sdl_sdr.write_program(true, &encoded_sdl_sdr);
    let _ = run_re_entry_labeled(&format!("unaligned SDL/SDR {}", memory_kind.name()), entry_sdl_sdr)?;
    if matches!(memory_kind, MemoryKind::Cached) {
        harness_sdl_sdr.writeback_dcache(store_address, 64);
    }
    let eff_sdl_sdr = reverse_endian_effective_address(store_address, 8);
    let bytes_sdl_sdr = harness_sdl_sdr.read_page_bytes(memory_kind, (eff_sdl_sdr - memory_kind.data_base()) as usize, 16);
    for i in 0..16 {
        soft_assert_eq2(bytes_sdl_sdr[i], bytes_sd[i], || {
            format!("SDL/SDR pair byte {} on {}", i, memory_kind.name())
        })?;
    }
    Ok(())
}

pub struct ReUnaligned32BitLoads;

impl Test for ReUnaligned32BitLoads {
    fn name(&self) -> &str { "RE user mode unaligned 32-bit loads (LWL/LWR)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for memory_kind in [MemoryKind::Cached, MemoryKind::Uncached] {
            run_unaligned_32bit_load_pairs(memory_kind)?;
        }
        Ok(())
    }
}

pub struct ReUnaligned32BitStores;

impl Test for ReUnaligned32BitStores {
    fn name(&self) -> &str { "RE user mode unaligned 32-bit stores (SWL/SWR)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for memory_kind in [MemoryKind::Cached, MemoryKind::Uncached] {
            run_unaligned_32bit_store_pairs(memory_kind)?;
        }
        Ok(())
    }
}

pub struct ReUnaligned64BitLoads;

impl Test for ReUnaligned64BitLoads {
    fn name(&self) -> &str { "RE user mode unaligned 64-bit loads (LDL/LDR)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for memory_kind in [MemoryKind::Cached, MemoryKind::Uncached] {
            run_unaligned_64bit_load_pairs(memory_kind)?;
        }
        Ok(())
    }
}

pub struct ReUnaligned64BitStores;

impl Test for ReUnaligned64BitStores {
    fn name(&self) -> &str { "RE user mode unaligned 64-bit stores (SDL/SDR)" }

    fn level(&self) -> Level { Level::RarelyUsed }

    fn values(&self) -> Vec<Box<dyn Any>> { Vec::new() }

    fn run(&self, _value: &Box<dyn Any>) -> Result<(), String> {
        for memory_kind in [MemoryKind::Cached, MemoryKind::Uncached] {
            run_unaligned_64bit_store_pairs(memory_kind)?;
        }
        Ok(())
    }
}
