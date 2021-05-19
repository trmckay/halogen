.global _start

_start:
    mv t0, x0
    loop:
    addi t0, t0, 1
    j loop
