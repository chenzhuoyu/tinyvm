#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod bsd_mman;
pub mod commpage;
pub mod mach_vm;
pub mod mmio;
pub mod shared_cache;
pub mod task;
pub mod tlb;

use super::regs::{Reg, SysReg};

pub trait HalProvider {
    fn read_reg(&self, reg: Reg) -> u64;
    fn write_reg(&self, reg: Reg, value: u64);
    fn read_sys_reg(&self, sys_reg: SysReg) -> u64;
    fn write_sys_reg(&self, sys_reg: SysReg, value: u64);
}

pub fn init() {
    commpage::init();
}
