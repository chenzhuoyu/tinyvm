#![feature(debug_closure_helpers)]
#![feature(macro_metavar_expr)]
#![cfg_attr(target_arch = "aarch64", feature(portable_simd))]
#![cfg_attr(target_arch = "aarch64", feature(simd_ffi))]

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
pub(crate) mod macros;
pub mod ptr;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    io::Error as IoError,
    ops::{Deref, DerefMut},
};

#[cfg(target_arch = "aarch64")]
use aarch64::ffi;
use anyhow::Context;
use bytes::{Buf, BufMut, buf::UninitSlice};
use ffi::{HV_MEMORY_EXEC, HV_MEMORY_READ, HV_MEMORY_WRITE};
use libc::{MAP_ANON, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use ptr::Uintptr;
#[cfg(target_arch = "x86_64")]
use x86_64::ffi;

pub type Unit = Maybe<()>;
pub type Maybe<T> = Result<T, anyhow::Error>;

#[macro_export]
macro_rules! hv_call {
    ($name:ident ( $($arg:expr),* $(,)? )) => {{
        #[allow(clippy::macro_metavars_in_unsafe)]
        match unsafe { $name($($arg),*) } {
            $crate::ffi::HV_SUCCESS => {}
            $crate::ffi::HV_ERROR => panic!(concat!(stringify!($name), ": generic hypervisor error")),
            $crate::ffi::HV_BUSY => panic!(concat!(stringify!($name), ": hypervisor is busy")),
            $crate::ffi::HV_BAD_ARGUMENT => panic!(concat!(stringify!($name), ": bad arguments")),
            $crate::ffi::HV_ILLEGAL_GUEST_STATE => panic!(concat!(stringify!($name), ": illegal guest state")),
            $crate::ffi::HV_NO_RESOURCES => panic!(concat!(stringify!($name), ": insufficient resources")),
            $crate::ffi::HV_NO_DEVICE => panic!(concat!(stringify!($name), ": no devices")),
            $crate::ffi::HV_DENIED => panic!(concat!(stringify!($name), ": denied")),
            #[cfg(target_arch = "x86_64")]
            $crate::ffi::HV_FAULT => panic!(concat!(stringify!($name), ": fault")),
            #[cfg(target_arch = "aarch64")]
            $crate::ffi::HV_EXISTS => panic!(concat!(stringify!($name), ": exists")),
            $crate::ffi::HV_UNSUPPORTED => panic!(concat!(stringify!($name), ": unsupported operation")),
            err => panic!("{}: unknown error: {}", stringify!($name), err),
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
    pub const RX: Self = Self::READ.union(Self::EXEC);
    pub const RW: Self = Self::READ.union(Self::WRITE);
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

pub trait Addressable {
    fn size(&self) -> usize;
    fn addr(&self) -> Uintptr;
}

pub struct Memory {
    base: Uintptr,
    size: usize,
}

impl Memory {
    pub fn mmap(size: usize) -> Maybe<Self> {
        let aligned_size = unsafe {
            let vm_page_size = libc::vm_page_size;
            (size + vm_page_size - 1) & !(vm_page_size - 1)
        };
        let base = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                aligned_size,
                PROT_READ | PROT_WRITE,
                MAP_ANON | MAP_PRIVATE,
                -1,
                0,
            )
        };
        if std::ptr::eq(base, MAP_FAILED) {
            tracing::error!("Cannot map memory with size {size}");
            return Err(IoError::last_os_error()).context("cannot map memory");
        }
        Ok(Self {
            base: base.into(),
            size: aligned_size,
        })
    }

    pub fn copy_from_slice(data: &[u8]) -> Maybe<Self> {
        let mut mem = Self::mmap(data.len())?;
        mem.view_mut(0).put_slice(data);
        Ok(mem)
    }
}

impl Memory {
    #[inline(always)]
    pub fn view(&self, pos: usize) -> MemoryView<'_> {
        MemoryView { pos, mem: self }
    }

    #[inline(always)]
    pub fn view_mut(&mut self, pos: usize) -> MemoryViewMut<'_> {
        MemoryViewMut { pos, mem: self }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        if unsafe { libc::munmap(self.base.as_ptr(), self.size) } != 0 {
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
        let end = self.base + self.size;
        write!(f, "memory({:p}-{:p})", self.base, end)
    }
}

impl Deref for Memory {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.base.as_ptr(), self.size) }
    }
}

impl DerefMut for Memory {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.base.as_ptr(), self.size) }
    }
}

impl Addressable for Memory {
    #[inline(always)]
    fn size(&self) -> usize {
        self.size
    }

    #[inline(always)]
    fn addr(&self) -> Uintptr {
        self.base
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryView<'m> {
    pos: usize,
    mem: &'m Memory,
}

impl Buf for MemoryView<'_> {
    #[inline(always)]
    fn remaining(&self) -> usize {
        self.mem.len() - self.pos
    }

    #[inline(always)]
    fn chunk(&self) -> &[u8] {
        &self.mem[self.pos..]
    }

    #[inline(always)]
    fn advance(&mut self, cnt: usize) {
        self.pos += cnt;
        assert!(self.pos <= self.mem.len());
    }
}

impl Addressable for MemoryView<'_> {
    #[inline(always)]
    fn size(&self) -> usize {
        self.mem.size() - self.pos
    }

    #[inline(always)]
    fn addr(&self) -> Uintptr {
        self.mem.addr() + self.pos
    }
}

#[derive(Debug)]
pub struct MemoryViewMut<'m> {
    pos: usize,
    mem: &'m mut Memory,
}

unsafe impl BufMut for MemoryViewMut<'_> {
    #[inline(always)]
    fn remaining_mut(&self) -> usize {
        self.mem.len() - self.pos
    }

    #[inline(always)]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.pos += cnt;
        assert!(self.pos <= self.mem.len());
    }

    #[inline(always)]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        UninitSlice::new(&mut self.mem[self.pos..])
    }
}

impl Addressable for MemoryViewMut<'_> {
    #[inline(always)]
    fn size(&self) -> usize {
        self.mem.size() - self.pos
    }

    #[inline(always)]
    fn addr(&self) -> Uintptr {
        self.mem.addr() + self.pos
    }
}
