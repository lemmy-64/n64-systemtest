// This is an IPL3 that is good enough to run inside of an emulator. However, it doesn't initialize any hardware components,
// so it wouldn't run on real hardware.
// This is built using make ipl3, which requires the binary available from https://github.com/ARM9/bass in the path

constant r0 = 0
constant r1 = 1
constant r2 = 2
constant r3 = 3
constant r4 = 4
constant r5 = 5
constant r6 = 6
constant r7 = 7
constant r8 = 8
constant r9 = 9
constant r10 = 10
constant r11 = 11
constant r12 = 12
constant r13 = 13
constant r14 = 14
constant r15 = 15
constant r16 = 16
constant r17 = 17
constant r18 = 18
constant r19 = 19
constant r20 = 20
constant r21 = 21
constant r22 = 22
constant r23 = 23
constant r24 = 24
constant r25 = 25
constant r26 = 26
constant r27 = 27
constant r28 = 28
constant r29 = 29
constant r30 = 30
constant r31 = 31

constant t0 = 8
constant t1 = 9
constant t2 = 10
constant t3 = 11
constant t4 = 12

constant Count = 9
constant Compare = 11
constant Cause = 13

constant PIReg_BASE_UPPER = 0xA460
constant PIReg_DRAM_ADDR_OFFSET = 0
constant PIReg_CART_ADDR_OFFSET = 0x4
constant PIReg_WR_LEN_OFFSET = 0xC
constant PIReg_STATUS_OFFSET = 0x10

output "mini-ipl3.bin", create
arch n64.cpu
endian msb
base 0xa400'0040

Entrypoint:
  mtc0 r0, Cause
  mtc0 r0, Count
  mtc0 r0, Compare

  // ** Memory size detection. Save into 0x80000318 **
  la t0, 0x8000'0000 + 4*1024*1024
  la t1, 0xBADDECAF
  sw t1, 0(t0)
  lw t2, 0(t0)
  beq t1, t2, expansion_pack_found
  la t1, 4*1024*1024
  j expansion_pack_done
  nop
expansion_pack_found:
  la t1, 8*1024*1024
expansion_pack_done:
  la t0, 0x80000318
  sw t1, 0(t0)

  // ** Load cart into DMEM **
  constant SOURCE_ADDRESS = 0x1000'1000  // has to be physical address
  constant TARGET_ADDRESS = 0x8000'0400  // can be physical or virtual - upper bits are ignored
  constant DMEM_SIZE = 0xFFFFF

  lui t0, PIReg_BASE_UPPER

  la t1, SOURCE_ADDRESS
  la t3, TARGET_ADDRESS
  la t2, DMEM_SIZE

  sw t1, PIReg_CART_ADDR_OFFSET(t0)
  sw t3, PIReg_DRAM_ADDR_OFFSET(t0)
  sw t2, PIReg_WR_LEN_OFFSET(t0)

wait_for_dma_finished:
  lw t1, PIReg_STATUS_OFFSET(t0)
  andi t1, t1, 1
  bne t1, r0, wait_for_dma_finished
  nop

  // Jump into the new code
  jr t3
  nop

origin 4024

// If we ever want to run on N64, place collision data here
dw 0x0, 0x0
