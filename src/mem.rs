use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    ops::{Deref, DerefMut},
};

use ffi::{HV_MEMORY_EXEC, HV_MEMORY_READ, HV_MEMORY_WRITE};

#[cfg(target_arch = "aarch64")]
use crate::aarch64::{ffi, vm::Vm};
use crate::utils::{ptr::Uintptr, size::align_to_page};
#[cfg(target_arch = "x86_64")]
use crate::x86_64::ffi;

bitflags::bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Protection : u64 {
        const EXEC  = HV_MEMORY_EXEC;
        const READ  = HV_MEMORY_READ;
        const WRITE = HV_MEMORY_WRITE;
    }
}

impl Protection {
    pub const RX: Self = Self::READ.union(Self::EXEC);
    pub const RW: Self = Self::READ.union(Self::WRITE);
    pub const NONE: Self = Self::empty();
}

impl Debug for Protection {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        macro_rules! write_flags {
            ($name:ident, $repr:literal) => {
                if self.contains(Self::$name) {
                    write!(f, $repr)?;
                } else {
                    write!(f, "-")?;
                }
            };
        }
        write_flags!(READ, "r");
        write_flags!(WRITE, "w");
        write_flags!(EXEC, "x");
        Ok(())
    }
}

pub struct Memory {
    addr: Uintptr,
    size: usize,
}

impl Memory {
    pub fn alloc(size: usize) -> Self {
        let size = align_to_page(size);
        let addr = Vm::alloc(size);
        Self { addr, size }
    }
}

impl Memory {
    #[inline]
    pub const fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub const fn addr(&self) -> Uintptr {
        self.addr
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        Vm::dealloc(self.addr, self.size);
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:p}-{:p}", self.addr, self.addr + self.size)
    }
}

impl Deref for Memory {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.addr.as_ptr(), self.size) }
    }
}

impl DerefMut for Memory {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.addr.as_ptr(), self.size) }
    }
}
