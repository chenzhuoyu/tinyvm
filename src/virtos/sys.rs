use std::io::{Error as IoError, Result as IoResult};

use crate::{mem::Protection, utils::ptr::Uintptr};

pub fn sys_map(
    addr: Uintptr,
    size: usize,
    prot: Protection,
    flags: i32,
    fd: i32,
    offset: i64,
) -> IoResult<Uintptr> {
    unsafe {
        match libc::mmap(addr.as_ptr(), size, prot.bits(), flags, fd, offset) {
            libc::MAP_FAILED => Err(IoError::last_os_error()),
            mem => Ok(Uintptr::from(mem)),
        }
    }
}

pub fn sys_unmap(addr: Uintptr, size: usize) -> IoResult<()> {
    if unsafe { libc::munmap(addr.as_ptr(), size) } < 0 {
        Err(IoError::last_os_error())
    } else {
        Ok(())
    }
}

pub fn sys_protect(addr: Uintptr, size: usize, prot: Protection) -> IoResult<()> {
    if unsafe { libc::mprotect(addr.as_ptr(), size, prot.bits()) } != 0 {
        Err(IoError::last_os_error())
    } else {
        Ok(())
    }
}
