.section .boot, "ax"
.global _start
.set noreorder

// Floating Point GPRs
.set FPC_CSR,   $31

// Floating Point status register
.set FPCSR_FS,  0x01000000 // Flush denormalized to zero
.set FPCSR_EV,  0x00000800 // Enable invalid operation exceptions

// N64 PIF/OS pointers
.set PIF_ENTRY_POINT,       0xBFC00000
.set PIF_CONTROL,           0x07FC

// Runtime environment pointers
.set FS_START,              0x8000031C

_start:
    // Find out the amount of RAM, in 2 MB increments. 4MB/8MB are normal, but an emulator might provide more
    li $t0, 0                                    // memory size determined, so far
    li $t1, 0xA0000000 | (2 * 1024 * 1024 - 4)   // address to check
    li $t2, 2 * 1024 * 1024                      // 2mb increment value
    li $t3, 0xCAFFEE                             // test value to write, which we expect to read back

memory_size_loop:
    lw $t5, 0($t1)  // backup previous value
    sw $t3, 0($t1)
    lw $t4, 0($t1)
    bne $t3, $t4, memory_size_found
    sw $t5, 0($t1)  // restore previous value (delay slot)

    addu $t0, $t0, $t2
    b memory_size_loop
    addu $t1, $t1, $t2  // delay slot

memory_size_found:
    // Make this the first argument for the Rust entrypoint
    move $a0, $t0

    // Initialize stack
    li $t1, 0x80000000
    or $sp, $t1, $t0

    // Configure Floating Point Unit
    li $t0, (FPCSR_FS | FPCSR_EV)
    ctc1 $t0, FPC_CSR

    // Enable PIF NMI
    li $t0, PIF_ENTRY_POINT
    ori $t1, $zero, 8
    sw $t1, PIF_CONTROL($t0)

    // Store the FS location for the OS
    la $t0, __rom_end
    li $t1, FS_START
    sw $t0, 0($t1)

    // Jump to Rust
    jal rust_entrypoint
    nop
