use std::io::{Error as IoError, Result as IoResult};

use super::{HalProvider, mmio, tlb::TlbProvider};
use crate::{
    aarch64::{
        paging::{PAGE_SIZE, PageTable},
        vm::Vm,
    },
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

pub fn munmap(hal: &impl HalProvider, addr: Uintptr, len: usize) -> IoResult<()> {
    let base = addr.as_u64();
    let size = align_to_page(len);
    let num_pages = size / PAGE_SIZE;

    /* remove from page table and such */
    PageTable::unmap(addr, len)?;
    mmio::unregister(addr, size);
    hal.flush_tlb(base, num_pages);
    Vm::unmap(base, size);

    /* actually remove from host address space */
    if unsafe { libc::munmap(addr.as_ptr(), len) } != 0 {
        Err(IoError::last_os_error())
    } else {
        Ok(())
    }
}

pub fn mprotect(hal: &impl HalProvider, addr: Uintptr, len: usize, prot: i32) -> IoResult<()> {
    if let Some(prot) = Protection::from_bits(prot as u64) {
        let size = align_to_page(len);
        PageTable::protect(addr, size, prot, false)?;
        hal.flush_tlb(addr.as_u64(), size / PAGE_SIZE);
        Vm::protect(addr, size, prot);
        Ok(())
    } else {
        Err(IoError::from_raw_os_error(libc::EINVAL))
    }
}

pub fn mmap(
    _hal: &impl HalProvider,
    addr: Uintptr,
    len: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    pos: libc::off_t,
) -> IoResult<Uintptr> {
    eprintln!("mmap(addr={addr:p}, len={len}, prot={prot}, flags={flags}, fd={fd}, pos={pos})");
    std::intrinsics::breakpoint();
    todo!()
}
