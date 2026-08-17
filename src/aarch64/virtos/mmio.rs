use std::{
    collections::BTreeMap,
    io::{Error as IoError, Result as IoResult},
    sync::Arc,
};

use parking_lot::RwLock;

use crate::{
    aarch64::{paging::PAGE_SIZE, vm::Vm},
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

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

struct MmioRegion {
    end: Uintptr,
    handler: Arc<dyn MmioHandler>,
}

unsafe impl Send for MmioRegion {}
unsafe impl Sync for MmioRegion {}

/// MMIO memory regions registry.
static MMIO: RwLock<BTreeMap<Uintptr, MmioRegion>> = RwLock::new(BTreeMap::new());

pub fn map(addr: Uintptr, size: usize, handler: impl MmioHandler + 'static) {
    let mut mmio = MMIO.write();
    let region = mmio.range(..addr + size).next_back();

    /* check the address alignment */
    assert!(
        addr.is_aligned_to(PAGE_SIZE) && size.is_multiple_of(PAGE_SIZE),
        "MMIO region should be page aligned"
    );

    /* check if the memory region overlaps */
    if let Some((&base, region)) = region {
        assert!(
            region.end <= addr,
            "MMIO region overlaps: {r1s:p}-{r1e:p} && {r2s:p}-{r2e:p}",
            r1s = base,
            r1e = region.end,
            r2s = addr,
            r2e = addr + size
        );
    }

    /* add a new handler */
    mmio.insert(
        addr,
        MmioRegion {
            end: addr + size,
            handler: Arc::new(handler),
        },
    );
}

pub fn unmap(addr: Uintptr, size: usize) {
    let mut keys = vec![];
    let mut mmio = MMIO.write();

    /* collect memory regions covered by the specified range */
    for (&base, region) in mmio.range(..addr + size).rev() {
        if region.end > addr {
            keys.push(base);
        } else {
            break;
        }
    }

    /* adjust the interval, split if needed */
    for base in keys {
        if let Some(item) = mmio.remove(&base) {
            if base < addr {
                mmio.insert(base, {
                    MmioRegion {
                        end: addr,
                        handler: item.handler.clone(),
                    }
                });
            }
            if item.end > addr + size {
                mmio.insert(addr + size, {
                    MmioRegion {
                        end: item.end,
                        handler: item.handler,
                    }
                });
            }
        } else {
            unsafe { std::intrinsics::unreachable() }
        }
    }
}

pub fn protect(addr: Uintptr, size: usize, prot: Protection) -> IoResult<()> {
    let size = align_to_page(size);
    let mut last = addr + size;

    /* address should align to page */
    if !addr.is_aligned_to(PAGE_SIZE) {
        return Err(IoError::from_raw_os_error(libc::EINVAL));
    }

    /* scan all regions backwards, protect all gaps */
    for (&base, region) in MMIO.read().range(..last).rev() {
        if region.end > addr {
            if region.end < last {
                Vm::protect(region.end, last - region.end, prot);
            }
            last = base;
        } else {
            break;
        }
    }

    /* the range is entirely covered by MMIO regions */
    if last <= addr {
        return Ok(());
    }

    /* there are still remaining pages */
    Vm::protect(addr, last - addr, prot);
    Ok(())
}

pub fn dispatch(pc: Uintptr, req: &mut MmioRequest) -> Option<MmioResponse> {
    let mmio = MMIO.read();
    let (&addr, region) = mmio.range(..=req.addr).next_back()?;

    /* check if the registered range covers the requested range completely */
    if addr <= req.addr && req.addr + req.size <= region.end {
        Some(region.handler.handle(pc, req))
    } else {
        None
    }
}
