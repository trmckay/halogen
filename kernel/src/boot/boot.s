.option norvc
.section .data

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
    sd    x0, (a0)
    addi  a0, a0, 8
    bltu  a0, a1, BSS_INIT_LOOP
BSS_INIT_DONE:

# Initialize the stack pointer.
    la    sp, _stack_end

# Machine protected mode
#               vv
    li    t0, 0b110010001000
#                   ^   ^
#           Enable interrupts

# Woops, still gotta do this to fully enable interrupts.
    csrw  mstatus, t0

# Go here (to the kernel) when we're done.
    la    t1, kernel
    csrw  mepc, t1

# Also need to initialize the machine trap vector.
    la    t2, mtrap_vector
    csrw  mtvec, t2

# More magic numbers!
    li    t3, 0b100010001000
    csrw  mie, t3
    la    ra, SLEEP
    mret

SLEEP:
    wfi
    j SLEEP

# It took me three days to write these 55 lines <]:)