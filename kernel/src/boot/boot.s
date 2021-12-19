# General behavior is this:
#   1. Select the correct hart to boot with. RISC-V requires 1.
#   2. Set up the global pointer
#   3. Initialize the BSS section.
#   4. Initialize the stack pointer.

.option norvc
.section .data

# Expose these linker symbols to Rust.
.global K_HEAP_BEGIN
K_HEAP_BEGIN: .dword _K_HEAP

.global K_HEAP_SIZE
K_HEAP_SIZE: .dword _K_HEAP_SIZE

.global K_STACK_BEGIN
K_STACK_BEGIN: .dword _K_STACK

.global K_STACK_END
K_STACK_END: .dword _K_STACK_END

.global TEXT_BEGIN
TEXT_BEGIN: .dword _MEM

.global MEM_END
MEM_END: .dword _MEM_END

.section .text.init

.global _start
_start:
# Boot with hart one.
    csrr  t0, mhartid
    bnez  t0, SLEEP
    csrw  satp, x0

.option push
.option norelax
 la    gp, _global_pointer
.option pop

# Initialize BSS section to zero for newly allocated memory.
    la    a0, _bss_start
    la    a1, _bss_end
    bgeu  a0, a1, BSS_INIT_DONE
BSS_INIT_LOOP:
    sw    x0, (a0)
    addi  a0, a0, 4
    bltu  a0, a1, BSS_INIT_LOOP
BSS_INIT_DONE:

# Initialize the stack pointer.
la    sp, _K_STACK_END

# Machine protected mode
#           vv
li    t0, 0b110010001000
#               ^   ^
#           Enable interrupts
csrw  mstatus, t0

# Go here (to the kernel) when we're done.
la    t1, kernel_start
csrw  mepc, t1

# Also need to initialize the machine trap vector.
la    t2, _mtrap_vector
csrw  mtvec, t2

la    ra, SLEEP
call kernel_start

SLEEP:
    wfi
    j SLEEP
