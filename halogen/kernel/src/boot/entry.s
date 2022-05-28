# No compression
.option norvc

# Disable supervisor interrupts for now
csrw sie, zero
csrci sstatus, 2

# Load the stack pointer
lla sp, __tmp_stack_top

# Load the global pointer
.option push
.option norelax
la gp, __global_pointer$

# Clear BSS
lla t0, __bss
lla t1, __bss_end

1:
    beq  t0, t1, 2f
    sd x0, 0(t0)
    addi t0, t0, 8
    j 1b
2:

call enable_paging

unimp
