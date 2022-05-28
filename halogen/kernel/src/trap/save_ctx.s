# 1. Disable interrupts (this is the locking mechanism for the memory at `sscratch`)
# 2. Store the CPU context at the pointer in `sscratch`
# 3. Use the space after the CPU context as a temporary stack
# 4. Restore the original value of `sscratch`
# 5. Call the Rust handler with the correct arguments
# 6. Configure the trap return based on the next context
# 7. Load the next context
# 8. Execute the trap return

# See `context.rs` for the structure definition
.equ CTX_REG_ARR_COUNT, 31
.equ CTX_REG_ARR_ELEM_SIZE, 8
.equ CTX_REG_ARR_OFFST, 0
.equ CTX_REG_ARR_SIZE, CTX_REG_ARR_COUNT * CTX_REG_ARR_ELEM_SIZE

.equ CTX_PC_SIZE, 8
.equ CTX_PC_OFFST, CTX_REG_ARR_OFFST + CTX_REG_ARR_SIZE

.equ CTX_PRIV_SIZE, 8
.equ CTX_PRIV_OFFST, CTX_PC_OFFST + CTX_PC_SIZE

.equ CTX_STRUCT_SIZE, CTX_PRIV_OFFST + CTX_PRIV_SIZE

csrci sstatus, 1

# Can't lose the value of sscratch or any of the registers

# Swap sp with sscratch storage
# The old sp is safe in sscratch
csrrw sp, sscratch, sp

# The sscratch value is now in sp and can go in the sp's spot for now:
# ctx.regs[1] i.e. sp - CTX_STRUCT_SIZE + CTX_REG_ARR_ELEM_SIZE
sd sp, (-CTX_STRUCT_SIZE + CTX_REG_ARR_ELEM_SIZE)(sp)

# Adjust the stack pointer now that we've saved it
# sp <- &regs
addi sp, sp, -CTX_STRUCT_SIZE

# ctx.registers <- GP registers
sd x1,  (CTX_REG_ARR_OFFST + 0  * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x3,  (CTX_REG_ARR_OFFST + 2  * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x4,  (CTX_REG_ARR_OFFST + 3  * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x5,  (CTX_REG_ARR_OFFST + 4  * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x6,  (CTX_REG_ARR_OFFST + 5  * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x7,  (CTX_REG_ARR_OFFST + 6  * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x8,  (CTX_REG_ARR_OFFST + 7  * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x9,  (CTX_REG_ARR_OFFST + 8  * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x10, (CTX_REG_ARR_OFFST + 9  * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x11, (CTX_REG_ARR_OFFST + 10 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x12, (CTX_REG_ARR_OFFST + 11 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x13, (CTX_REG_ARR_OFFST + 12 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x14, (CTX_REG_ARR_OFFST + 13 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x15, (CTX_REG_ARR_OFFST + 14 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x16, (CTX_REG_ARR_OFFST + 15 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x17, (CTX_REG_ARR_OFFST + 18 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x18, (CTX_REG_ARR_OFFST + 17 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x19, (CTX_REG_ARR_OFFST + 18 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x20, (CTX_REG_ARR_OFFST + 19 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x21, (CTX_REG_ARR_OFFST + 20 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x22, (CTX_REG_ARR_OFFST + 21 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x23, (CTX_REG_ARR_OFFST + 22 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x24, (CTX_REG_ARR_OFFST + 23 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x25, (CTX_REG_ARR_OFFST + 24 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x26, (CTX_REG_ARR_OFFST + 25 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x27, (CTX_REG_ARR_OFFST + 26 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x28, (CTX_REG_ARR_OFFST + 27 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x29, (CTX_REG_ARR_OFFST + 28 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x30, (CTX_REG_ARR_OFFST + 29 * CTX_REG_ARR_ELEM_SIZE)(sp)
sd x31, (CTX_REG_ARR_OFFST + 30 * CTX_REG_ARR_ELEM_SIZE)(sp)

# Now that we've saved all the registers, use temporaries to save sscratch and the old sp

# Get the old sscratch that was saved in the sp's spot on the array
ld t0, (CTX_REG_ARR_OFFST + 1 * CTX_REG_ARR_ELEM_SIZE)(sp)

# Swap sscratch (holds the old sp) with t0 (holds the old sscratch)
csrrw t0, sscratch, t0
# Put t0 (now holds the old sp) in the register array
sd t0, (CTX_REG_ARR_OFFST + 1 * CTX_REG_ARR_ELEM_SIZE)(sp)

# Now store the pc and privilege level
csrr t0, sepc
sd t0, (CTX_PC_OFFST)(sp)

# TODO: actual read privilege value
li t0, 1
sd t0, (CTX_PRIV_OFFST)(sp)

# a0: *const Context <- trap_handler(&regs, scause, stval)
mv a0, sp
csrr a1, scause
csrr a2, stval
call trap_handler

# sepc <- return pc
ld t0, (CTX_PC_OFFST)(a0)
csrrw t0, sepc, t0

# Configure trap return
# TODO: respect return privilege level
ld t0, (CTX_PRIV_OFFST)(a0)
csrs sstatus, t0

# Load next register context
ld x1,  (CTX_REG_ARR_OFFST + 0  * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x2,  (CTX_REG_ARR_OFFST + 1  * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x3,  (CTX_REG_ARR_OFFST + 2  * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x4,  (CTX_REG_ARR_OFFST + 3  * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x5,  (CTX_REG_ARR_OFFST + 4  * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x6,  (CTX_REG_ARR_OFFST + 5  * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x7,  (CTX_REG_ARR_OFFST + 6  * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x8,  (CTX_REG_ARR_OFFST + 7  * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x9,  (CTX_REG_ARR_OFFST + 8  * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x11, (CTX_REG_ARR_OFFST + 10 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x12, (CTX_REG_ARR_OFFST + 11 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x13, (CTX_REG_ARR_OFFST + 12 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x14, (CTX_REG_ARR_OFFST + 13 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x15, (CTX_REG_ARR_OFFST + 14 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x16, (CTX_REG_ARR_OFFST + 15 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x17, (CTX_REG_ARR_OFFST + 18 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x18, (CTX_REG_ARR_OFFST + 17 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x19, (CTX_REG_ARR_OFFST + 18 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x20, (CTX_REG_ARR_OFFST + 19 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x21, (CTX_REG_ARR_OFFST + 20 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x22, (CTX_REG_ARR_OFFST + 21 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x23, (CTX_REG_ARR_OFFST + 22 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x24, (CTX_REG_ARR_OFFST + 23 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x25, (CTX_REG_ARR_OFFST + 24 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x26, (CTX_REG_ARR_OFFST + 25 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x27, (CTX_REG_ARR_OFFST + 26 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x28, (CTX_REG_ARR_OFFST + 27 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x29, (CTX_REG_ARR_OFFST + 28 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x30, (CTX_REG_ARR_OFFST + 29 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld x31, (CTX_REG_ARR_OFFST + 30 * CTX_REG_ARR_ELEM_SIZE)(a0)
ld a0,  (CTX_REG_ARR_OFFST + 9  * CTX_REG_ARR_ELEM_SIZE)(a0)

# Return from the trap/interrupt/exception
sret
