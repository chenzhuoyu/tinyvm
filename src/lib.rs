#![feature(debug_closure_helpers)]

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    io::{Error as IoError, Result as IoResult},
    ops::{Deref, DerefMut},
};

use ffi::{HV_MEMORY_EXEC, HV_MEMORY_READ, HV_MEMORY_WRITE};
use libc::{MAP_ANON, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE, c_void};
#[cfg(target_arch = "x86_64")]
pub use x86_64::{Cpu, Vm, ffi};

#[macro_export]
macro_rules! hv_call {
    ($invoke:expr) => {{
        #[allow(clippy::macro_metavars_in_unsafe)]
        match unsafe { $invoke } {
            $crate::ffi::HV_SUCCESS => Ok(()),
            $crate::ffi::HV_ERROR => Err($crate::io_error!(Other, "hypervisor error")),
            $crate::ffi::HV_BUSY => Err($crate::io_error!(ResourceBusy, "hypervisor is busy")),
            $crate::ffi::HV_BAD_ARGUMENT => Err($crate::io_error!(InvalidInput, "bad arguments")),
            $crate::ffi::HV_NO_RESOURCES => Err($crate::io_error!(Other, "insufficient resources")),
            $crate::ffi::HV_NO_DEVICE => Err($crate::io_error!(Other, "no devices")),
            $crate::ffi::HV_UNSUPPORTED => Err($crate::io_error!(Unsupported, "unsupported")),
            err => Err($crate::io_error!(Other, "unknown error: {err}")),
        }
    }};
}

#[macro_export]
macro_rules! io_error {
    ($kind:ident, $msg:literal) => {
        std::io::Error::new(std::io::ErrorKind::$kind, format!($msg))
    };
    ($kind:ident, $expr:expr) => {
        std::io::Error::new(std::io::ErrorKind::$kind, $expr)
    };
    ($kind:ident, $msg:literal, $($arg:tt)*) => {
        std::io::Error::new(std::io::ErrorKind::$kind, format!($msg, $($arg)*))
    };
}

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct Protection : u64 {
        const EXEC  = HV_MEMORY_EXEC;
        const READ  = HV_MEMORY_READ;
        const WRITE = HV_MEMORY_WRITE;
    }
}

impl Protection {
    pub const RW: Self = Self::READ.union(Self::WRITE);
    pub const WX: Self = Self::WRITE.union(Self::EXEC);
}

impl Protection {
    fn as_mprotect(self) -> i32 {
        macro_rules! select_flags {
            ($name:ident) => {
                if self.contains(Self::$name) {
                    paste::paste! { [< PROT_ $name >]}
                } else {
                    0
                }
            };
        }
        select_flags!(READ) | select_flags!(WRITE) | select_flags!(EXEC)
    }
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
    pub(crate) base: *mut c_void,
    pub(crate) size: usize,
    pub(crate) prot: Protection,
}

impl Memory {
    pub fn mmap(size: usize, prot: Protection) -> IoResult<Self> {
        let base = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                size,
                prot.as_mprotect(),
                MAP_ANON | MAP_PRIVATE,
                -1,
                0,
            )
        };
        if std::ptr::eq(base, MAP_FAILED) {
            tracing::error!("Cannot map memory with size {size} as {prot:?}");
            return Err(IoError::last_os_error());
        }
        Ok(Self { base, size, prot })
    }
}

impl Memory {
    #[inline]
    pub fn write(&mut self, offs: usize, data: &[u8]) {
        self[offs..offs + data.len()].copy_from_slice(data);
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        if unsafe { libc::munmap(self.base, self.size) } != 0 {
            tracing::error!(
                "Cannot unmap memory block at {:p} of size {}: {}",
                self.base,
                self.size,
                IoError::last_os_error(),
            );
        }
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let end = unsafe { self.base.add(self.size) };
        write!(f, "memory({:p}-{:p}:{:?})", self.base, end, self.prot)
    }
}

impl Deref for Memory {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.base as *const u8, self.size) }
    }
}

impl DerefMut for Memory {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.base as *mut u8, self.size) }
    }
}
