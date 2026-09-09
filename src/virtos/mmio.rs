use std::fmt::Debug;

use crate::utils::ptr::Uintptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmioKind {
    Read,
    ReadAtomic,
    Write,
    WriteAtomic,
    Execution,
}

#[derive(Debug, Clone, Copy)]
pub struct MmioRequest {
    pub data: u64,
    pub size: usize,
    pub addr: Uintptr,
    pub kind: MmioKind,
}

impl MmioRequest {
    #[inline(always)]
    pub const fn read_unsized(addr: Uintptr) -> Self {
        Self {
            addr,
            size: 0,
            data: 0,
            kind: MmioKind::Read,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmioResponse {
    Retry,
    Advance,
}

pub trait MmioHandler {
    fn handle(&self, pc: Uintptr, req: &mut MmioRequest) -> MmioResponse;
}

impl<F: Fn(Uintptr, &mut MmioRequest) -> MmioResponse> MmioHandler for F {
    #[inline]
    fn handle(&self, pc: Uintptr, req: &mut MmioRequest) -> MmioResponse {
        self(pc, req)
    }
}
