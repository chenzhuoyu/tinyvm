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
