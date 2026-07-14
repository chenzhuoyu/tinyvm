.global _virtos_end
.global _virtos_start
.balign 128

_virtos_start:
.set idx, 0
.rept 16
.balign 128
    hvc     #idx
    b.vs    .L_flush_tlb
    eret
.set idx, idx + 1
.endr

.balign 128
.L_flush_tlb:
    dsb     ishst
    cmp     x17, #0x200000
    b.ls    .L_flush_16
    tlbi    vmalle1is
    b       .L_done

.macro tlbi_rv mask, scale, shift, target
    ands    x11, x17, #(~\mask)
    b.eq    \target
    mov     x10, #((0b10 << 46) | (\scale << 44))
    orr     x10, x10, x16, lsr #14
    add     x16, x16, x11, lsl #14
    and     x17, x17, #\mask
    sub     x11, x11, #(\mask + 1)
    orr     x10, x10, x11, lsl #\shift
    tlbi    rvae1is, x10
.endmacro

.L_flush_16:
    tlbi_rv 0xffff, 3, 23, .L_flush_11

.L_flush_11:
    tlbi_rv 0x07ff, 2, 28, .L_flush_6

.L_flush_6:
    tlbi_rv 0x3f, 1, 33, .L_flush_2

.L_flush_2:
    tlbi_rv 1, 0, 38, .L_flush_1

.L_flush_1:
    lsr     x16, x16, #12
    tlbi    vae1is, x16

.L_done:
    dsb     ish
    isb
    eret
_virtos_end:
