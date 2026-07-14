use std::io::{Error as IoError, Result as IoResult};

use super::{HalProvider, tlb::TlbProvider};
use crate::{
    aarch64::{
        paging::{PAGE_SIZE, PageTable},
        vm::Vm,
    },
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

pub fn munmap(hal: &impl HalProvider, addr: Uintptr, len: usize) -> IoResult<()> {
    let ret = unsafe {
        PageTable::unmap(addr.as_u64(), len)?;
        Vm::unmap(addr.as_u64(), align_to_page(len));
        hal.flush_tlb_range(addr.as_u64(), len.div_ceil(PAGE_SIZE));
        libc::munmap(addr.as_ptr(), len)
    };
    if ret != 0 {
        Err(IoError::last_os_error())
    } else {
        Ok(())
    }
}

pub fn mprotect(hal: &impl HalProvider, addr: Uintptr, len: usize, prot: i32) -> IoResult<()> {
    if let Some(prot) = Protection::from_bits(prot as u64) {
        PageTable::protect(addr.as_u64(), len, prot)?;
        Vm::protect(addr, align_to_page(len), prot);
        hal.flush_tlb_range(addr.as_u64(), len.div_ceil(PAGE_SIZE));
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
