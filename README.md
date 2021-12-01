# n64-systemtest
Tests a wide variety of N64 features, from common to hardware quirks. Written in Rust. Executes quickly.

n64-systemtest is a test rom that is useful for emulator developers or anyone who is interested in understanding how the N64 really works. Design goals of this test rom:
1) Productivity 1: n64-systemtest itself decides whether it failed or succeeded. No need to compare images,
2) Productivity 2: Writing new tests is quick and easy,
3) Readability: Tests should be easy to understand and provide good error messages that make it clear what's broken,
4) Speed: Everything should run quickly so that the test rom can be used for regression tests
5) Open source: Everything is open-source.

