.global _bl_end
.global _bl_start
.p2align 4

_bl_start:
    mov     x0, #1
    adr     x1, msg
    mov     x2, #14
    mov     x16, #4
    svc     #0x80

    mov     x0, #0
    mov     x16, #1
    svc     #0x80

msg:
    .ascii "Hello, World!\n"

_bl_end:

.global _irq_stub_end
.global _irq_stub_start
.balign 128

_irq_stub_start:
.set idx, 0
.rept 16
.balign 128
    hvc #idx
    eret
.set idx, idx + 1
.endr
_irq_stub_end:
