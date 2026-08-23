use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    io::{Error as IoError, Result as IoResult},
    ops::{Deref, DerefMut},
    sync::Arc,
};

use parking_lot::Mutex;
use smallvec::SmallVec;

use crate::{
    aarch64::{
        cpu::Cpu,
        disasm::disasm,
        paging::{PAGE_SIZE, PageTable},
        virtos::mmio::{MmioHandler, MmioKind, MmioRequest, MmioResponse},
        vm::Vm,
    },
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
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
    populated: bool,
}

impl VmRegion {
    fn map(&self, addr: Uintptr) {
        if self.kind.is_regular() {
            Vm::map(addr, self.next - addr, self.prot);
        }
    }

    fn unmap(&self, start: Uintptr, end: Uintptr) {
        assert!(
            end <= self.next,
            "unmapping addresses beyound the current region: {self:#?}"
        );
        if self.populated || self.kind.is_regular() {
            Vm::unmap(start, end - start);
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
        prefault: bool,
    ) {
        let region = VmRegion {
            kind,
            prot,
            next: addr + size,
            max_prot,
            populated: prefault,
        };

        /* remove existing mappings before adding new one */
        self.unmap_range(addr, size);
        region.map(addr);
        self.maps.insert(addr, region);
        eprintln!("map(): {addr:p}-{end:p}", end = addr + size);

        /* prefault the pages if needed */
        if prefault {
            self.page.prefault(addr, addr + size, prot, max_prot);
        }
    }

    fn unmap_range(&mut self, addr: Uintptr, size: usize) {
        eprintln!("unmap(): {addr:p}-{end:p}", end = addr + size);
        let keys = self
            .maps
            .range(..addr + size)
            .rev()
            .take_while(|&(.., r)| r.next > addr)
            .map(|(&p, ..)| p)
            .collect::<SmallVec<[Uintptr; 16]>>();
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
                        populated: region.populated,
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
                        populated: region.populated,
                    }
                });
            }
            region.unmap(base.max(addr), region.next.min(addr + size));
        }
        self.page.unset(addr, size / PAGE_SIZE);
    }

    fn protect_range(
        &mut self,
        cpu: &Cpu,
        addr: Uintptr,
        size: usize,
        prot: Protection,
        set_max: bool,
    ) -> IoResult<()> {
        let mut tlbi = false;
        let mut iter = self.maps.range(..addr + size);
        let mut rbuf = SmallVec::<[(Uintptr, VmRegion); 2]>::new();

        /* get the first covered region, make sure the address range doesn't have trailing gaps */
        let mut head = {
            if let Some((&head, region)) = iter.next_back() {
                if region.next < addr + size {
                    return Err(IoError::from_raw_os_error(libc::ENOMEM));
                } else {
                    head
                }
            } else {
                return Err(IoError::from_raw_os_error(libc::ENOMEM));
            }
        };

        /* perform validations before actually make changes */
        for (&base, region) in iter.rev() {
            if region.next <= addr {
                break;
            }
            if region.next != head {
                return Err(IoError::from_raw_os_error(libc::ENOMEM));
            }
            if !prot.difference(region.max_prot).is_empty() {
                return Err(IoError::from_raw_os_error(libc::EACCES));
            }
            head = base;
        }

        /* scan through the regions, split regions as needed */
        for (&base, region) in self.maps.range_mut(head..addr + size).rev() {
            if region.next <= addr {
                break;
            }
            if region.populated {
                tlbi = true;
            }
            if addr + size < region.next {
                let tail = VmRegion {
                    kind: region.kind.clone(),
                    next: region.next,
                    prot: region.prot,
                    max_prot: region.max_prot,
                    populated: region.populated,
                };
                rbuf.push((addr + size, tail));
                region.next = addr + size;
            }
            if base < addr {
                let lead = VmRegion {
                    kind: region.kind.clone(),
                    next: region.next,
                    prot,
                    max_prot: if set_max { prot } else { region.max_prot },
                    populated: region.populated,
                };
                rbuf.push((addr, lead));
                region.next = addr;
            } else {
                if set_max {
                    region.max_prot = prot;
                }
                region.prot = prot;
                assert_eq!(addr, base);
            }
        }

        /* handle block splitting */
        for (base, region) in rbuf {
            self.maps.insert(base, region);
        }

        /* check if we need to flush TLB */
        if !tlbi {
            return Ok(());
        }

        /* modify page table, and flush the TLB */
        self.page.protect(addr, size / PAGE_SIZE, prot);
        cpu.flush_tlb(addr, size / PAGE_SIZE);
        Ok(())
    }
}

impl VmMap {
    #[inline]
    fn find_mmio(&mut self, pc: Uintptr, req: &MmioRequest) -> Option<Arc<dyn MmioHandler>> {
        let (.., region) = self
            .maps
            .range_mut(..=req.addr)
            .next_back()
            .filter(|(.., r)| r.next > req.addr)?;
        let VmKind::Mmio(mmio) = &region.kind else {
            panic!(
                "unexpected MMIO fault\nAddress:\n  {addr:p}\nInstruction:\n  {insn}",
                addr = req.addr,
                insn = disasm(pc)
            );
        };
        region.populated = true;
        Some(mmio.clone())
    }

    #[inline]
    fn page_fault_in(&mut self, addr: Uintptr, kind: MmioKind) -> bool {
        let Some((.., region)) = self.maps.range_mut(..=addr).next_back() else {
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
        region.prot.contains(mask) && {
            self.page.set(addr, region.prot, region.max_prot);
            region.populated = true;
            true
        }
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
        prefault: bool,
    ) {
        assert!(
            addr.is_aligned_to(PAGE_SIZE),
            "unaligned map base address: {addr:p}"
        );
        VMM.lock().map_range(
            kind.into(),
            addr,
            align_to_page(size),
            prot,
            max_prot,
            prefault,
        );
    }

    #[inline]
    pub fn unmap(addr: Uintptr, size: usize) {
        assert!(
            addr.is_aligned_to(PAGE_SIZE),
            "unaligned unmap base address: {addr:p}"
        );
        VMM.lock().unmap_range(addr, align_to_page(size));
    }

    #[inline]
    pub fn protect(
        cpu: &Cpu,
        addr: Uintptr,
        size: usize,
        prot: Protection,
        set_max: bool,
    ) -> IoResult<()> {
        assert!(
            addr.is_aligned_to(PAGE_SIZE),
            "unaligned protect address: {addr:p}"
        );
        VMM.lock()
            .protect_range(cpu, addr, align_to_page(size), prot, set_max)
    }
}

impl VmMap {
    #[inline]
    pub fn lookup(addr: Uintptr) -> Protection {
        unsafe { (*VMM.data_ptr()).page.lookup(addr) }
    }
}

impl VmMap {
    #[inline]
    pub fn handle_mmio(pc: Uintptr, req: &mut MmioRequest) -> Option<MmioResponse> {
        let mmio = VMM.lock().find_mmio(pc, req)?;
        Some(mmio.handle(pc, req))
    }

    #[inline]
    pub fn handle_page_fault(addr: Uintptr, kind: MmioKind) -> bool {
        VMM.lock().page_fault_in(addr, kind)
    }
}
