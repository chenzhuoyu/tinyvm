#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod bsd_mman;
pub mod commpage;
pub mod faults;
pub mod mach_msg;
pub mod mach_vm;
pub mod mem;
pub mod mmio;
pub mod pac;
pub mod shared_cache;
pub mod task;
pub mod tlb;

use super::{
    paging::PageTable,
    regs::{Reg, SysReg},
};
use crate::{
    mem::{Memory, Protection},
    utils::{ptr::Uintptr, size::align_to_page},
};

/// The guest address of IRQ stubs.
pub const IRQ_STUBS: Uintptr = Uintptr::new(0x7fff_0000_0000);

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

#[inline]
fn virtos_code() -> &'static [u8] {
    unsafe {
        let end = &raw const virtos_end;
        let start = &raw const virtos_start;
        std::slice::from_raw_parts(start, end.offset_from_unsigned(start))
    }
}

pub fn init() {
    let code = virtos_code();
    let size = align_to_page(code.len());
    let addr = Memory::map(size, Protection::RX);

    /* load the IRQ stubs into memory */
    unsafe {
        std::ptr::write_bytes(addr.as_ptr::<u8>(), 0, size);
        std::ptr::copy_nonoverlapping(code.as_ptr(), addr.as_ptr(), code.len());
    }

    /* log the IRQ stubs range */
    tracing::debug!(
        "IRQ Stubs are loaded into {addr:p}-{end:p}",
        end = addr + code.len()
    );

    /* initialize the page table */
    PageTable::init();
    PageTable::map(addr, IRQ_STUBS, size, Protection::RX, Protection::RX);
    commpage::init();
}
