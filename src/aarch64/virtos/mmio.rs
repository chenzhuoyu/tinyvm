use std::collections::BTreeMap;

use parking_lot::RwLock;

use crate::utils::ptr::Uintptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmioKind {
    Read,
    ReadAtomic,
    Write,
    WriteAtomic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmioSize {
    Mem8,
    Mem16,
    Mem32,
    Mem64,
}

impl MmioSize {
    #[inline(always)]
    pub const fn bytes(self) -> usize {
        match self {
            Self::Mem8 => 1,
            Self::Mem16 => 2,
            Self::Mem32 => 4,
            Self::Mem64 => 8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MmioRequest {
    pub data: u64,
    pub addr: Uintptr,
    pub size: MmioSize,
    pub kind: MmioKind,
}

pub trait MmioHandler {
    fn handle(&self, pc: Uintptr, req: &mut MmioRequest);
}

impl<F: Fn(Uintptr, &mut MmioRequest)> MmioHandler for F {
    #[inline]
    fn handle(&self, pc: Uintptr, req: &mut MmioRequest) {
        self(pc, req)
    }
}

struct MmioRegion {
    size: usize,
    handler: Box<dyn MmioHandler>,
}

unsafe impl Send for MmioRegion {}
unsafe impl Sync for MmioRegion {}

/// MMIO memory regions registry.
static MMIO: RwLock<BTreeMap<Uintptr, MmioRegion>> = RwLock::new(BTreeMap::new());

pub fn dispatch(pc: Uintptr, req: &mut MmioRequest) -> Option<()> {
    let mmio = MMIO.read();
    let (&addr, region) = mmio.range(..=req.addr).next_back()?;

    /* check if the registered range covers the requested range completely */
    if addr <= req.addr && req.addr + req.size.bytes() <= addr + region.size {
        region.handler.handle(pc, req);
        Some(())
    } else {
        None
    }
}

pub fn register(addr: Uintptr, size: usize, handler: impl MmioHandler + 'static) {
    let mut mmio = MMIO.write();
    let region = mmio.range(..addr + size).next_back();

    /* check if the memory region overlaps */
    if let Some((&base, region)) = region {
        assert!(
            base + region.size <= addr,
            "MMIO region overlaps: {r1s:p}-{r1e:p} && {r2s:p}-{r2e:p}",
            r1s = base,
            r1e = base + region.size,
            r2s = addr,
            r2e = addr + size
        );
    }

    /* add a new handler */
    mmio.insert(
        addr,
        MmioRegion {
            size,
            handler: Box::new(handler),
        },
    );
}
