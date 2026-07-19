use std::io::{Error as IoError, Result as IoResult};

use crate::{mem::Protection, utils::ptr::Uintptr};

pub fn protect(addr: Uintptr, size: usize, prot: Protection) -> IoResult<()> {
    if unsafe { libc::mprotect(addr.as_ptr(), size, prot.bits() as i32) } != 0 {
        Err(IoError::last_os_error())
    } else {
        Ok(())
    }
}
