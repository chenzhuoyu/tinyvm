use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    io::Result as IoResult,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use parking_lot::Mutex;
use smallvec::SmallVec;

use crate::{
    aarch64::{
        paging::{PAGE_SIZE, PageTable},
        virtos::mmio::{MmioHandler, MmioKind, MmioRequest, MmioResponse},
        vm::Vm,
    },
    mem::Protection,
    utils::ptr::Uintptr,
};

#[derive(Clone)]
pub enum VmKind {
    Mmio(Arc<dyn MmioHandler>),
    Regular,
}

impl VmKind {
    #[inline]
    const fn is_regular(&self) -> bool {
        matches!(self, Self::Regular)
    }
}

impl Debug for VmKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Mmio(..) => write!(f, "Mmio"),
            Self::Regular => write!(f, "Regular"),
        }
    }
}

impl<H: MmioHandler + 'static> From<H> for VmKind {
    #[inline]
    fn from(handler: H) -> Self {
        Self::Mmio(Arc::new(handler))
    }
}

#[derive(Debug)]
struct VmRegion {
    kind: VmKind,
    next: Uintptr,
    prot: Protection,
    max_prot: Protection,
}

impl VmRegion {
    fn map(&self, addr: Uintptr) -> IoResult<()> {
        if self.kind.is_regular() {
            Vm::map(addr, self.next - addr, self.prot);
            Ok(())
        } else {
            // TODO
            Ok(())
        }
    }

    fn unmap(&self, start: Uintptr, end: Uintptr) -> IoResult<()> {
        assert!(
            end <= self.next,
            "unmapping addresses beyound the current region: {self:#?}"
        );
        if self.kind.is_regular() {
            Vm::unmap(start, end - start);
            Ok(())
        } else {
            todo!()
        }
    }
}

#[repr(transparent)]
struct PageTableRef(*mut PageTable);

impl Deref for PageTableRef {
    type Target = PageTable;

    #[inline(always)]
    fn deref(&self) -> &PageTable {
        unsafe { &*self.0 }
    }
}

impl DerefMut for PageTableRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut PageTable {
        unsafe { &mut *self.0 }
    }
}

pub struct VmMap {
    page: PageTableRef,
    maps: BTreeMap<Uintptr, VmRegion>,
}

unsafe impl Send for VmMap {}
unsafe impl Sync for VmMap {}

static VMM: Mutex<VmMap> = Mutex::new(VmMap {
    page: PageTableRef(std::ptr::null_mut()),
    maps: BTreeMap::new(),
});

impl VmMap {
    pub fn init() {
        unsafe {
            assert!(
                libc::vm_page_size == PAGE_SIZE,
                "page size went bananas: {size} != {PAGE_SIZE}",
                size = libc::vm_page_size,
            );
            let tab = Vm::alloc(PAGE_SIZE);
            Vm::map(tab, PAGE_SIZE, Protection::RW);
            (*VMM.data_ptr()).page.0 = tab.as_ptr();
        }
    }

    pub fn base() -> Uintptr {
        let vmm = unsafe { &(*VMM.data_ptr()) };
        assert!(!vmm.page.0.is_null(), "VM is not initialized");
        Uintptr::from(vmm.page.0)
    }
}

impl VmMap {
    fn map_range(
        &mut self,
        kind: VmKind,
        addr: Uintptr,
        size: usize,
        prot: Protection,
        max_prot: Protection,
    ) -> IoResult<()> {
        let region = VmRegion {
            kind,
            prot,
            next: addr + size,
            max_prot,
        };
        self.unmap_range(addr, size)?;
        region.map(addr)?;
        self.maps.insert(addr, region);
        Ok(())
    }

    fn unmap_range(&mut self, addr: Uintptr, size: usize) -> IoResult<()> {
        let keys = self
            .maps
            .range(..addr + size)
            .rev()
            .take_while(|&(.., r)| r.next > addr)
            .map(|(&p, ..)| p)
            .collect::<SmallVec<[Uintptr; 8]>>();
        for base in keys {
            let region = self
                .maps
                .remove(&base)
                .unwrap_or_else(|| unsafe { std::intrinsics::unreachable() });
            if base < addr {
                self.maps.insert(base, {
                    VmRegion {
                        next: addr,
                        prot: region.prot,
                        kind: region.kind.clone(),
                        max_prot: region.max_prot,
                    }
                });
            }
            if region.next > addr + size {
                self.maps.insert(addr + size, {
                    VmRegion {
                        next: region.next,
                        prot: region.prot,
                        kind: region.kind.clone(),
                        max_prot: region.max_prot,
                    }
                });
            }
            region.unmap(base.max(addr), region.next.min(addr + size))?;
        }
        self.page.unset(addr, size.div_ceil(PAGE_SIZE));
        Ok(())
    }

    fn prefault_range(&mut self, addr: Uintptr, size: usize) {
        for (&base, region) in self.maps.range(..addr + size).rev() {
            if region.next > addr {
                self.page.prefault(
                    base.max(addr),
                    region.next.min(addr + size),
                    region.prot,
                    region.max_prot,
                );
            } else {
                break;
            }
        }
    }
}

impl VmMap {
    #[inline]
    fn on_page_fault(&mut self, addr: Uintptr, kind: MmioKind) -> bool {
        let Some((.., region)) = self.maps.range(..=addr).next_back() else {
            return false;
        };
        if region.next <= addr {
            return false;
        }
        let mask = {
            match kind {
                MmioKind::Read => Protection::READ,
                MmioKind::Write => Protection::WRITE,
                MmioKind::Execution => Protection::EXEC,
                _ => unreachable!(),
            }
        };
        if !region.prot.contains(mask) {
            return false;
        }
        self.page.set(addr, region.prot, region.max_prot);
        true
    }
}

impl VmMap {
    #[inline]
    pub fn map(
        kind: impl Into<VmKind>,
        addr: Uintptr,
        size: usize,
        prot: Protection,
        max_prot: Protection,
    ) -> IoResult<()> {
        VMM.lock()
            .map_range(kind.into(), addr, size, prot, max_prot)
    }

    #[inline]
    pub fn unmap(addr: Uintptr, size: usize) -> IoResult<()> {
        VMM.lock().unmap_range(addr, size)
    }

    #[inline]
    pub fn protect(addr: Uintptr, size: usize, prot: Protection, set_max: bool) -> IoResult<()> {
        todo!()
    }

    #[inline]
    pub fn prefault(addr: Uintptr, size: usize) {
        VMM.lock().prefault_range(addr, size);
    }
}

impl VmMap {
    #[inline]
    pub fn lookup(addr: Uintptr) -> Protection {
        unsafe { (*VMM.data_ptr()).page.lookup(addr) }
    }

    #[inline]
    pub fn insert(
        kind: impl Into<VmKind>,
        addr: Uintptr,
        size: usize,
        prot: Protection,
        max_prot: Protection,
    ) {
        if let Err(err) = Self::map(kind, addr, size, prot, max_prot) {
            panic!("cannot add VM map entry: {err}");
        }
    }
}

impl VmMap {
    pub fn handle_mmio(pc: Uintptr, req: &mut MmioRequest) -> Option<MmioResponse> {
        todo!()
    }

    #[inline]
    pub fn handle_page_fault(addr: Uintptr, kind: MmioKind) -> bool {
        VMM.lock().on_page_fault(addr, kind)
    }
}
