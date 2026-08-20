use crate::{
    aarch64::{
        cpu::Cpu,
        regs::{PSTATE_V, Reg},
        virtos::STACK_TOP,
    },
    utils::ptr::Uintptr,
};

impl Cpu {
    #[inline]
    pub fn flush_tlb(&self, addr: Uintptr, num_pages: usize) {
        STACK_TOP.write([addr.addr(), num_pages]);
        self.write_reg(Reg::CPSR, self.read_reg(Reg::CPSR) | PSTATE_V);
    }
}
