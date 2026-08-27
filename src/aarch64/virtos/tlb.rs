use crate::{
    aarch64::{
        cpu::Cpu,
        regs::{PSTATE_V, Reg},
        virtos::stack_top,
    },
    utils::ptr::VMA,
};

impl Cpu {
    #[inline]
    pub fn flush_tlb(&self, addr: VMA, num_pages: usize) {
        stack_top().write([addr.addr(), num_pages as u64]);
        self.write_reg(Reg::CPSR, self.read_reg(Reg::CPSR) | PSTATE_V);
    }
}
