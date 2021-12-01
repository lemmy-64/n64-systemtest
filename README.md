# n64-systemtest
Tests a wide variety of N64 features, from common to hardware quirks. Written in Rust. Executes quickly.

n64-systemtest is a test rom that is useful for emulator developers or anyone who is interested in understanding how the N64 really works. Design goals of this test rom:
1) Productivity: It should be easy to write new tests quickly
2) Readability: Tests should be easy to understand and provide good error messages that make it clear what's broken
3) Speed: Everything should run quickly so that the test rom can be used for regression test
4) Open source: Everything is open-source
