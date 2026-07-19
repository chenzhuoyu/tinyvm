#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod bsd_mman;
pub mod commpage;
pub mod faults;
pub mod mach_vm;
pub mod mem;
pub mod mmio;
pub mod shared_cache;
pub mod task;
pub mod tlb;

use super::{
    paging::PageTable,
    regs::{Reg, SysReg},
};
use crate::{
    mem::{Memory, Protection},
    utils::ptr::Uintptr,
};

pub trait HalProvider {
    fn read_reg(&self, reg: Reg) -> u64;
    fn write_reg(&self, reg: Reg, value: u64);
    fn read_sys_reg(&self, sys_reg: SysReg) -> u64;
    fn write_sys_reg(&self, sys_reg: SysReg, value: u64);
}

unsafe extern "C" {
    unsafe static virtos_end: u8;
    unsafe static virtos_start: u8;
}

/// The guest address of IRQ stubs.
static mut IRQ_STUBS: Uintptr = Uintptr::NIL;

#[inline]
fn virtos_code() -> &'static [u8] {
    unsafe {
        let end = &raw const virtos_end;
        let start = &raw const virtos_start;
        std::slice::from_raw_parts(start, end.offset_from_unsigned(start))
    }
}

#[inline]
pub fn irq_stubs() -> u64 {
    unsafe { IRQ_STUBS.as_u64() }
}

pub fn init() {
    let code = virtos_code();
    let page = Memory::from_data(code).map(Protection::RX);
    let (phys, size) = page.into_parts();

    /* set the IRQ stubs address */
    unsafe {
        IRQ_STUBS = phys;
        tracing::debug!("IRQ Stubs are loaded into {:p}-{:p}", phys, phys + size);
    }

    /* initialize the page table */
    PageTable::init();
    PageTable::insert(phys, size, Protection::RX, Protection::RX);
    commpage::init();
}
