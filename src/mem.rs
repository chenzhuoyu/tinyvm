use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{
        Deref, DerefMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
    },
};

use bytes::{Buf, BufMut, buf::UninitSlice};
use ffi::{HV_MEMORY_EXEC, HV_MEMORY_READ, HV_MEMORY_WRITE};

#[cfg(target_arch = "aarch64")]
use crate::aarch64::{ffi, vm::Vm};
use crate::utils::{
    ptr::Uintptr,
    size::{align_to_page, is_page_aligned},
};
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

pub trait MemoryExt: Addressable {
    fn protect(&self, prot: Protection);
}

impl<T: Addressable> MemoryExt for T {
    #[inline]
    fn protect(&self, prot: Protection) {
        assert!(is_page_aligned(self.size()));
        assert!(is_page_aligned(self.addr().addr()));
        Vm::protect(self.addr(), self.size(), prot);
    }
}

pub trait Addressable {
    fn size(&self) -> usize;
    fn addr(&self) -> Uintptr;
}

pub trait MemoryRange {
    fn view(self, mem: &Memory) -> MemoryView<'_>;
    fn view_mut(self, mem: &mut Memory) -> MemoryViewMut<'_>;
}

impl MemoryRange for RangeFull {
    #[inline]
    fn view(self, mem: &Memory) -> MemoryView<'_> {
        MemoryView {
            size: mem.size,
            addr: mem.addr,
            _ref: PhantomData,
        }
    }

    #[inline]
    fn view_mut(self, mem: &mut Memory) -> MemoryViewMut<'_> {
        MemoryViewMut {
            size: mem.size,
            addr: mem.addr,
            _ref: PhantomData,
        }
    }
}

impl MemoryRange for Range<usize> {
    #[inline]
    fn view(self, mem: &Memory) -> MemoryView<'_> {
        if self.end <= mem.size && self.start <= mem.size {
            MemoryView {
                size: self.end.saturating_sub(self.start),
                addr: mem.addr + self.start,
                _ref: PhantomData,
            }
        } else {
            panic!("memory view slice out of bounds: {self:?}")
        }
    }

    #[inline]
    fn view_mut(self, mem: &mut Memory) -> MemoryViewMut<'_> {
        if self.end <= mem.size && self.start <= mem.size {
            MemoryViewMut {
                size: self.end.saturating_sub(self.start),
                addr: mem.addr + self.start,
                _ref: PhantomData,
            }
        } else {
            panic!("memory view slice out of bounds: {self:?}")
        }
    }
}

impl MemoryRange for RangeTo<usize> {
    #[inline]
    fn view(self, mem: &Memory) -> MemoryView<'_> {
        if self.end <= mem.size {
            MemoryView {
                size: self.end,
                addr: mem.addr,
                _ref: PhantomData,
            }
        } else {
            panic!("memory view slice out of bounds: {self:?}")
        }
    }

    #[inline]
    fn view_mut(self, mem: &mut Memory) -> MemoryViewMut<'_> {
        if self.end <= mem.size {
            MemoryViewMut {
                size: self.end,
                addr: mem.addr,
                _ref: PhantomData,
            }
        } else {
            panic!("memory view slice out of bounds: {self:?}")
        }
    }
}

impl MemoryRange for RangeFrom<usize> {
    #[inline]
    fn view(self, mem: &Memory) -> MemoryView<'_> {
        if self.start <= mem.size {
            MemoryView {
                size: mem.size - self.start,
                addr: mem.addr + self.start,
                _ref: PhantomData,
            }
        } else {
            panic!("memory view slice out of bounds: {self:?}")
        }
    }

    #[inline]
    fn view_mut(self, mem: &mut Memory) -> MemoryViewMut<'_> {
        if self.start <= mem.size {
            MemoryViewMut {
                size: mem.size - self.start,
                addr: mem.addr + self.start,
                _ref: PhantomData,
            }
        } else {
            panic!("memory view slice out of bounds: {self:?}")
        }
    }
}

impl MemoryRange for RangeInclusive<usize> {
    #[inline]
    fn view(self, mem: &Memory) -> MemoryView<'_> {
        let (start, mut end) = self.into_inner();
        end += 1;
        MemoryRange::view(Range { start, end }, mem)
    }

    #[inline]
    fn view_mut(self, mem: &mut Memory) -> MemoryViewMut<'_> {
        let (start, mut end) = self.into_inner();
        end += 1;
        MemoryRange::view_mut(Range { start, end }, mem)
    }
}

impl MemoryRange for RangeToInclusive<usize> {
    #[inline]
    fn view(self, mem: &Memory) -> MemoryView<'_> {
        let end = self.end + 1;
        MemoryRange::view(RangeTo { end }, mem)
    }

    #[inline]
    fn view_mut(self, mem: &mut Memory) -> MemoryViewMut<'_> {
        let end = self.end + 1;
        MemoryRange::view_mut(RangeTo { end }, mem)
    }
}

pub struct UnmappedMemory {
    addr: Uintptr,
    size: usize,
}

impl UnmappedMemory {
    #[inline]
    pub fn map(self, prot: Protection) -> Memory {
        let (addr, size) = self.into_parts();
        Vm::map(addr, addr.as_u64(), size, prot);
        Memory { addr, size }
    }
}

impl UnmappedMemory {
    #[inline]
    pub fn into_parts(self) -> (Uintptr, usize) {
        let this = ManuallyDrop::new(self);
        (this.addr, this.size)
    }
}

impl Drop for UnmappedMemory {
    fn drop(&mut self) {
        Vm::dealloc(self.addr, self.size);
    }
}

impl Debug for UnmappedMemory {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let end = self.addr + self.size;
        write!(f, "unmapped_memory({:p}-{:p})", self.addr, end)
    }
}

pub struct Memory {
    addr: Uintptr,
    size: usize,
}

impl Memory {
    pub fn alloc(size: usize) -> UnmappedMemory {
        let size = align_to_page(size);
        let addr = Vm::alloc(size);
        UnmappedMemory { addr, size }
    }

    pub fn from_data(data: &[u8]) -> UnmappedMemory {
        let dest = Self::alloc(data.len());
        let addr = dest.addr.as_ptr::<u8>();
        unsafe { addr.copy_from_nonoverlapping(data.as_ptr(), data.len()) }
        dest
    }
}

impl Memory {
    #[inline]
    pub fn view<R: MemoryRange>(&self, range: R) -> MemoryView<'_> {
        range.view(self)
    }

    #[inline]
    pub fn view_mut<R: MemoryRange>(&mut self, range: R) -> MemoryViewMut<'_> {
        range.view_mut(self)
    }
}

impl Memory {
    #[inline]
    pub fn into_parts(self) -> (Uintptr, usize) {
        let this = ManuallyDrop::new(self);
        (this.addr, this.size)
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        Vm::unmap(self.addr.as_u64(), self.size);
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

impl Addressable for Memory {
    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    #[inline]
    fn addr(&self) -> Uintptr {
        self.addr
    }
}

#[derive(Clone, Copy)]
pub struct MemoryView<'m> {
    size: usize,
    addr: Uintptr,
    _ref: PhantomData<&'m Memory>,
}

impl Buf for MemoryView<'_> {
    #[inline]
    fn remaining(&self) -> usize {
        self.size
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.addr.as_ptr(), self.size) }
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        self.size = self.size.checked_sub(cnt).expect("advance past end");
        self.addr += cnt;
    }
}

impl Debug for MemoryView<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:p}-{:p}", self.addr, self.addr + self.size)
    }
}

impl Addressable for MemoryView<'_> {
    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    #[inline]
    fn addr(&self) -> Uintptr {
        self.addr
    }
}

pub struct MemoryViewMut<'m> {
    size: usize,
    addr: Uintptr,
    _ref: PhantomData<&'m mut Memory>,
}

impl Debug for MemoryViewMut<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:p}-{:p}", self.addr, self.addr + self.size)
    }
}

unsafe impl BufMut for MemoryViewMut<'_> {
    #[inline]
    fn remaining_mut(&self) -> usize {
        self.size
    }

    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.size = self.size.checked_sub(cnt).expect("advance past end");
        self.addr += cnt;
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        unsafe {
            UninitSlice::new(std::slice::from_raw_parts_mut(
                self.addr.as_ptr(),
                self.size,
            ))
        }
    }
}

impl Addressable for MemoryViewMut<'_> {
    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    #[inline]
    fn addr(&self) -> Uintptr {
        self.addr
    }
}
