.section .boot, "ax"
.global _start
.set noreorder

// Floating Point GPRs
.set FPC_CSR,   $31

// Floating Point status register
.set FPCSR_FS,  0x01000000 // Flush denormalized to zero
.set FPCSR_EV,  0x00000800 // Enable invalid operation exceptions

// N64 PIF/OS pointers
.set OS_MEM_SIZE,           0x80000318
.set PIF_ENTRY_POINT,       0xBFC00000
.set PIF_CONTROL,           0x07FC

// Runtime environment pointers
.set FS_START,              0x8000031C

_start:
    // Initialize stack
    li $t0, OS_MEM_SIZE
    lw $t0, 0($t0)
    li $t1, 0x80000000
    or $sp, $t0, $t1

    // Clear .bss section
    la $t0, __bss_start
    la $t1, __bss_end
bss_clear_loop:
    bge $t0, $t1, bss_clear_done
    nop
    sw $zero, 0($t0)
    addiu $t0, $t0, 4
    b bss_clear_loop
    nop
bss_clear_done:

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
