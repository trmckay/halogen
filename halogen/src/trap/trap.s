// For a list of GNU assembler directives:
// https://ftp.gnu.org/old-gnu/Manuals/gas-2.9.1/html_chapter/as_7.html

// Frame saving logic derived from:
// https://osblog.stephenmarz.com/ch4.html

.option norvc

.altmacro
.set NUM_GP_REGS, 32
.set REG_SIZE, 8

.macro save_gp i, basereg=t6
    sd  x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro load_gp i, basereg=t6
    ld  x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro save_fp i, basereg=t6
    fsd  f\i, ((NUM_GP_REGS+(\i))*REG_SIZE)(\basereg)
.endm
.macro load_fp i, basereg=t6
    fld  f\i, ((NUM_GP_REGS+(\i))*REG_SIZE)(\basereg)
.endm


.global _mtrap_vector
_mtrap_vector:
    csrrw  t6, mscratch, t6

    call  mtrap_vector
    mret

    .set  i, 1
    .rept  30
        save_gp  %i
       .set  i, i+1
    .endr

    mv  t5, t6
    csrr  t6, mscratch
    save_gp  31, t5

    csrw  mscratch, t5

    csrr  a0, mepc
    csrr  a1, mtval
    csrr  a2, mcause
    csrr  a3, mhartid
    csrr  a4, mstatus

    mv  a5, t5
    ld  sp, 520(a5)
    call  mtrap_vector

    csrw  mepc, a0
    csrr  t6, mscratch

    .set  i, 1
    .rept  31
        load_gp  %i
       .set  i, i+1
    .endr

    mret
