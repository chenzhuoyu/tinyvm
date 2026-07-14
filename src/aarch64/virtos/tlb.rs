use super::HalProvider;
use crate::aarch64::regs::{PSTATE_V, Reg};

pub trait TlbProvider {
    fn flush_tlb_range(&self, start: u64, num_pages: usize);
}

impl<P: HalProvider> TlbProvider for P {
    fn flush_tlb_range(&self, start: u64, num_pages: usize) {
        let cpsr = self.read_reg(Reg::CPSR);
        self.write_reg(Reg::X16, start);
        self.write_reg(Reg::X17, num_pages as u64);
        self.write_reg(Reg::CPSR, cpsr | PSTATE_V);
    }
}
