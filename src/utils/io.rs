use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    fs::File,
    io::{Error as IoError, ErrorKind, Result as IoResult},
    ops::Deref,
    os::fd::AsRawFd,
    path::Path,
};

use anyhow::Context;

use crate::{Maybe, utils::ptr::Uintptr};

pub trait AsUsize: Copy {
    fn as_usize(self) -> usize;
}

macro_rules! impl_as_usize {
    ($($ty:ty),* $(,)?) => {
        $(
            impl AsUsize for $ty {
                #[allow(dead_code)]
                #[inline(always)]
                fn as_usize(self) -> usize {
                    self as usize
                }
            }
        )*
    };
}

impl_as_usize! {
    i8,
    i16,
    i32,
    i64,
    isize,
    u8,
    u16,
    u32,
    u64,
    usize,
}

pub trait ValueExt<T: AsUsize> {
    fn value(self) -> T;
    fn usize(self) -> usize;
}

macro_rules! impl_get_int {
    ($( $endian:ty { $( $ty:ty ),* $(,)? } )*) => {
        paste::paste! {
            $($(
                impl ValueExt<$ty> for object::[< $ty:upper >]<object::$endian> {
                    #[allow(dead_code)]
                    #[inline(always)]
                    fn value(self) -> $ty {
                        self.get(object::$endian)
                    }

                    #[allow(dead_code)]
                    #[inline(always)]
                    fn usize(self) -> usize {
                        self.value().as_usize()
                    }
                }
            )*)*
        }
    };
}

impl_get_int! {
    BigEndian    { u16, u32, u64 }
    LittleEndian { u16, u32, u64 }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryIo<'s>(pub &'s [u8]);

impl MemoryIo<'_> {
    #[inline]
    pub fn new(data: Uintptr, size: usize) -> Self {
        unsafe { Self(std::slice::from_raw_parts(data.as_ptr(), size)) }
    }
}

impl MemoryIo<'_> {
    #[inline]
    pub fn read<T: Copy>(&mut self) -> IoResult<T> {
        let ret = self.read_at(0)?;
        self.0 = &self.0[std::mem::size_of::<T>()..];
        Ok(ret)
    }

    #[inline]
    pub fn read_at<T: Copy>(&self, offs: usize) -> IoResult<T> {
        if self.0.len() >= std::mem::size_of::<T>() + offs {
            unsafe { Ok(self.0.as_ptr().add(offs).cast::<T>().read()) }
        } else {
            Err(IoError::from(ErrorKind::UnexpectedEof))
        }
    }
}

pub struct MappedFile {
    size: usize,
    base: *mut u8,
}

unsafe impl Send for MappedFile {}
unsafe impl Sync for MappedFile {}

impl MappedFile {
    pub fn map<F: AsRawFd>(file: F, size: usize, offset: usize) -> Maybe<Self> {
        let base = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                offset as libc::off_t,
            )
        };
        if std::ptr::eq(base, libc::MAP_FAILED) {
            return Err(IoError::last_os_error()).context("cannot map file");
        }
        Ok(Self {
            size,
            base: base as *mut u8,
        })
    }

    pub fn map_from<P: AsRef<Path>>(path: P) -> Maybe<Self> {
        let file = File::open(path)?;
        let size = file.metadata()?.len() as usize;
        Self::map(file, size, 0)
    }
}

impl MappedFile {
    #[inline]
    pub fn data(&self) -> &'static [u8] {
        unsafe { std::slice::from_raw_parts(self.base, self.size) }
    }
}

impl Drop for MappedFile {
    fn drop(&mut self) {
        if !self.base.is_null() {
            unsafe { libc::munmap(self.base as *mut libc::c_void, self.size) };
        }
    }
}

impl Deref for MappedFile {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.data()
    }
}

impl Debug for MappedFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        unsafe {
            write!(
                f,
                "MappedFile({:p}-{:p})",
                self.base,
                self.base.add(self.size)
            )
        }
    }
}
