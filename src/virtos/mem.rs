use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    io::{Error as IoError, Result as IoResult},
    ops::{Add, Deref, DerefMut},
    sync::Arc,
};

use parking_lot::Mutex;
use smallvec::SmallVec;

use crate::{
    cpu::Cpu,
    disasm::disasm,
    mem::Protection,
    paging::{PAGE_SIZE, PageTable},
    utils::{
        ptr::{IPA, Uintptr},
        size::align_to_page,
    },
    virtos::mmio::{MmioHandler, MmioKind, MmioRequest, MmioResponse},
    vm::Vm,
};

#[derive(Clone)]
enum VmKind {
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

#[derive(Debug, Clone, Copy)]
struct VmBase {
    phys: IPA,
    virt: Uintptr,
}

impl VmBase {
    #[inline]
    fn identity(addr: Uintptr) -> Self {
        Self {
            phys: addr.into(),
            virt: addr,
        }
    }
}

impl Add<usize> for VmBase {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self {
            phys: self.phys + rhs,
            virt: self.virt + rhs,
        }
    }
}

#[derive(Debug)]
struct VmRegion {
    size: usize,
    kind: VmKind,
    base: VmBase,
    prot: Protection,
    max_prot: Protection,
    populated: bool,
}

impl VmRegion {
    fn map(&self, addr: Uintptr) {
        if self.kind.is_regular() {
            assert_eq!(self.base.phys, self.base.virt);
            Vm::map(addr, self.base.phys, self.size, self.prot);
        }
    }

    fn unmap(&self, addr: Uintptr, size: usize) {
        if self.populated || self.kind.is_regular() {
            Vm::unmap(addr.into(), size);
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
            Vm::map(tab, tab.into(), PAGE_SIZE, Protection::RW);
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
        base: VmBase,
        size: usize,
        prot: Protection,
        max_prot: Protection,
        prefault: bool,
    ) {
        let addr = base.virt;
        let next = addr + size;

        /* construct the VM region */
        let region = VmRegion {
            base,
            kind,
            size,
            prot,
            max_prot,
            populated: prefault,
        };

        /* remove existing mappings before adding new one */
        self.unmap_range(addr, size);
        region.map(addr);
        self.maps.insert(addr, region);

        /* prefault the pages if needed */
        if prefault {
            self.page.prefault(base.phys, addr, next, prot, max_prot);
        }
    }

    fn unmap_range(&mut self, addr: Uintptr, size: usize) {
        let keys = self
            .maps
            .range(..addr + size)
            .rev()
            .take_while(|&(&p, r)| p + r.size > addr)
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
                        base: region.base,
                        size: addr - base,
                        kind: region.kind.clone(),
                        prot: region.prot,
                        max_prot: region.max_prot,
                        populated: region.populated,
                    }
                });
            }
            if base + region.size > addr + size {
                self.maps.insert(addr + size, {
                    VmRegion {
                        base: region.base + (addr + size - base),
                        size: base + region.size - (addr + size),
                        kind: region.kind.clone(),
                        prot: region.prot,
                        max_prot: region.max_prot,
                        populated: region.populated,
                    }
                });
            }
            // region.unmap(base.max(addr), region.next.min(addr + size));
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
                if head + region.size < addr + size {
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
            if base + region.size <= addr {
                break;
            } else if base + region.size != head {
                return Err(IoError::from_raw_os_error(libc::ENOMEM));
            } else if !prot.difference(region.max_prot).is_empty() {
                return Err(IoError::from_raw_os_error(libc::EACCES));
            } else {
                head = base;
            }
        }

        /* scan through the regions, split regions as needed */
        for (&base, region) in self.maps.range_mut(head..addr + size).rev() {
            if base + region.size <= addr {
                break;
            }
            if region.populated {
                tlbi = true;
            }
            if addr + size < base + region.size {
                let tail = VmRegion {
                    base: region.base + (addr + size - base),
                    size: (base + region.size) - (addr + size),
                    kind: region.kind.clone(),
                    prot: region.prot,
                    max_prot: region.max_prot,
                    populated: region.populated,
                };
                rbuf.push((addr + size, tail));
                region.size = addr + size - base;
            }
            if base < addr {
                let lead = VmRegion {
                    base: region.base + (addr - base),
                    size: base + region.size - addr,
                    kind: region.kind.clone(),
                    prot,
                    max_prot: if set_max { prot } else { region.max_prot },
                    populated: region.populated,
                };
                rbuf.push((addr, lead));
                region.size = addr - base;
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
            .filter(|&(&p, ref r)| p + r.size > req.addr)?;
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
        let Some((&base, region)) = self.maps.range_mut(..=addr).next_back() else {
            return false;
        };
        if base + region.size <= addr {
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
            self.page
                .set(addr, region.base.phys, region.prot, region.max_prot);
            region.populated = true;
            true
        }
    }
}

impl VmMap {
    #[inline]
    pub fn mmap(
        addr: Uintptr,
        size: usize,
        prot: Protection,
        max_prot: Protection,
        prefault: bool,
    ) {
        assert!(
            addr.is_aligned_to(PAGE_SIZE),
            "unaligned map address: {addr:p}"
        );
        VMM.lock().map_range(
            VmKind::Regular,
            VmBase::identity(addr),
            align_to_page(size),
            prot,
            max_prot,
            prefault,
        );
    }

    #[inline]
    pub fn iomap(
        mmio: impl MmioHandler + 'static,
        addr: Uintptr,
        phys: IPA,
        size: usize,
        prot: Protection,
        max_prot: Protection,
        prefault: bool,
    ) {
        assert!(
            addr.is_aligned_to(PAGE_SIZE) && phys.is_aligned_to(PAGE_SIZE),
            "unaligned map addresses: {addr:p}"
        );
        VMM.lock().map_range(
            VmKind::Mmio(Arc::new(mmio)),
            VmBase { phys, virt: addr },
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
            "unaligned unmap address: {addr:p}"
        );
        let size = align_to_page(size);
        VMM.lock().unmap_range(addr, size);
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
        let size = align_to_page(size);
        VMM.lock().protect_range(cpu, addr, size, prot, set_max)
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
