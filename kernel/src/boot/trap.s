.section .text

.global mtrap_vector

# Kernel trap vector.
# Handles syscalls and exceptions.
mtrap_vector:
    mret
