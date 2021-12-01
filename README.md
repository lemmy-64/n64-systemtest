# n64-systemtest
Tests a wide variety of N64 features, from common to hardware quirks. Written in Rust. Executes quickly.

n64-systemtest is a test rom that is useful for emulator developers or anyone who is interested in understanding how the N64 really works. Design goals of this test rom:
1) Productivity 1: n64-systemtest itself decides whether it failed or succeeded. No need to compare images,
2) Productivity 2: Writing new tests is quick and easy,
3) Readability: Tests should be easy to understand and provide good error messages that make it clear what's broken,
4) Speed: Everything should run quickly so that the test rom can be used for regression tests,
5) Open source Rust: Everything that is used to produce the final rom is open-source, written in Rust.

# Status
n64-systemtest tests common but also some of the more exotic features of the MIPS CPU:
- MFC0/DMFC0/MTC0/DMTC0: Some registers (e.g. EntryHi, BadVAddr) are expected to be 64 bit
- LLD/LD/SC/SCD
- TRAP instructions, BREAK, SYSCALL
- TLB
- Exceptions: Overflow (ADD, DADD etc), unaligned memory access (e.g. LW)
- Reading from and writing to ROM

# How to build
n64-systemtest can be built on Windows, mac or Linux (including within WSL). The steps are pretty much the same.
1. Get Rust: https://www.rust-lang.org/tools/install
2. Get the source, e.g. through **git checkout https://github.com/lemmy-64/n64-systemtest.git**
3. If this is your only rust-on-n64 project, simply run **make install-dependencies** (If you need cargo-n64 for other projects, install manually as needed. Notice that cargo-n64 official is on an older Rust unstable than n64-systemtest).
4. Run **make n64** to build the test rom.

Please note: N64 roms require a bootcode called IPL3. This bootcode is expected to setup hardware and copy the rom into memory. n64-systemtest comes with its own IPL3, which will **NOT** run on hardware. Once there is a community built open-source IPL3, we'll switch to that. If you'd like to use your own IPL3, please update the path in the Makefile at the very top.

# How to run
Run the rom in your emulator of choice. Expect one of three things:
1. The rom says something like "Done! Tests: 262. Failed: 0". If this is your emulator: Congratulations, you are done.
2. The rom says something like "Done! Tests: 262. Failed: 1" OR the screen is full of error messages. This means that issues were found because the emulator isn't perfect. Hopefully, the error messages are clear enough to indicate what's broken.
3. An empty screen: The emulator didn't make it to the end. See _troubleshooting_.

# Troubleshooting
n64-systemtest runs A LOT of tests. If things are very broken, it can be hard to figure out how make any progress. There are two ways to make progress in such a situation:

## ISViewer
All output that is printed on screen is also printed to memory mapped registers. For debugging, it is very valuable to hook this up and e.g. print to the console. To do that, simply provide the following two things:
- 0xB3FF0020 until 0xB3FF0220: A buffer that can be written to using SB
- 0xB3FF0014: A SW-writable length register. When written to, print the contents of the buffer

## Disable tests
While running all tests is nice once a majority passes, it can be a pain for bringup. **tests/testlist.rs** contains the list of all tests. Simply comment out some or all as needed.

## acknowledgment
This project was inspired by Peter Lemon's excellent N64 Bare Metal tests: https://github.com/PeterLemon/N64/
Furthermore, it wouldn't have been possible without the excellent cargo-n64, which brought Rust to the N64: https://github.com/rust-console/cargo-n64
