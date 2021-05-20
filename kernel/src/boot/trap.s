# trap.S
# Assembly-level trap handler.

.section .text

.global mtrap_vector

mtrap_vector:
    mret
