# n64-systemtest
Tests a wide variety of N64 features, from common to hardware quirks. Written in Rust. Executes quickly.

n64-systemtest is a test rom that is useful for emulator developers or anyone who is interested in understanding how the N64 really works. Design goals of this test rom:
1) Productivity 1: n64-systemtest itself decides whether it failed or succeeded. No need to compare images,
2) Productivity 2: Writing new tests is quick and easy,
3) Readability: Tests should be easy to understand and provide good error messages that make it clear what's broken,
4) Speed: Everything should run quickly so that the test rom can be used for regression tests,
5) Open source Rust: Everything that is used to produce the final rom is open-source, written in Rust.

# Status
n64-systemtest tests common but also some of the more exotic features of the N64:
- MFC0/DMFC0/MTC0/DMTC0: Some registers (e.g. EntryHi, BadVAddr) are expected to be 64 bit
- LLD/LD/SC/SCD
- Exceptions: Overflow (ADD, DADD etc), unaligned memory access (e.g. LW), TRAP instructions, BREAK, SYSCALL
- TLB
- Access (8, 16, 32, 64 bit) to RAM, ROM, SPMEM, PIF
- RSP

# How to build
n64-systemtest can be built on Windows, Mac or Linux (including within WSL). The steps are pretty much the same.
1. Install Rust: https://www.rust-lang.org/tools/install
2. Get the source: (e.g. using git, or downloading an archive manually)
```
git clone https://github.com/lemmy-64/n64-systemtest.git
cd n64-systemtest
```
3. Install prerequisites:
```
cargo install nust64
```
4. Run `cargo run --release` to build the test rom.

# Expanded test-set
In addition to the regular set of tests, n64-systemtest has a few additional sets which can
be enabled individually: timing, cycle and cop0hazard. Refer to cargo.toml for a detailed description.

```
cargo run --release --features cycle,timing
```

# Stresstests
n64-systemtest has stresstests, which take too long to be included by default. To compile just the stresstests,
use --no-default-features (to exclude the base set) and then specify the test you want. See cargo.toml for a full list.

```
cargo run --release --no-default-features --features vmulf_stress_test,vmulu_stress_test
```

# How to run
Run the rom in your emulator of choice. Expect one of three things:
1. The rom says something like "Done! Tests: 262. Failed: 0". If this is your emulator: Congratulations, you are done.
2. The rom says something like "Done! Tests: 262. Failed: 1" OR the screen is full of error messages. This means that issues were found because the emulator isn't perfect. Hopefully, the error messages are clear enough to indicate what's broken.
3. An empty screen: The emulator didn't make it to the end. See _troubleshooting_.

# Troubleshooting
n64-systemtest runs A LOT of tests. If things are very broken, it can be hard to figure out how make any progress. Some tips on how to make progress:

## Missing instructions
(If you emulator supports LL, SC, DMFC0, DMTC0, feel free to skip this part)

n64-systemtest uses some unusual instructions. If your emulator doesn't support those, there's a good chance the test suite won't run until the end. To avoid those crashes, it can be helpful to implement the following instructions:
- Make LL work like LW
- Make SC work like SW
- Make DMFC0 work like MFC0
- Make DMTC0 work like MTC0

Just to be clear: The things above are wrong. But they are good enough approximations to allow the testsuite to reach the end. It will show plenty of errors that require correct implementations of the instructions above.

## ISViewer
All output that is printed on screen is also printed to memory mapped registers. For debugging, it is very valuable to hook this up and e.g. print to the console. To do that, simply provide the following two things:
- 0xB3FF0020 until 0xB3FF0220: A buffer that can be written to using SB
- 0xB3FF0014: A SW-writable length register. When written to, print the contents of the buffer

## Disable tests
While running all tests is nice once a majority passes, it can be a pain for bringup. **tests/testlist.rs** contains the list of all tests. Simply comment out some or all as needed.

## Acknowledgment
This project was inspired by Peter Lemon's excellent N64 Bare Metal tests: https://github.com/PeterLemon/N64/
Furthermore, it wouldn't have been possible without the excellent cargo-n64, which brought Rust to the N64: https://github.com/rust-console/cargo-n64
