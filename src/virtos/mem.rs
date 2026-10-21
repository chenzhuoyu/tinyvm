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
    cpu::Cpu,
    disasm::disasm,
    mem::Protection,
    paging::{PAGE_SIZE, PageTable},
    utils::{
        ptr::{IPA, Uintptr},
        size::align_to_page,
    },
    virtos::mmio::{MmioContext, MmioHandler, MmioKind},
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
    #[inline]
    fn end(&self) -> Uintptr {
        self.base.virt + self.size
    }

    #[inline]
    fn init(&self) {
        if self.kind.is_regular() {
            assert_eq!(self.base.phys, self.base.virt);
            Vm::map(self.base.virt, self.base.phys, self.size, self.prot);
        }
    }
}

impl VmRegion {
    #[inline]
    fn update(&mut self, prot: Protection, set_max: bool) {
        if set_max {
            self.max_prot = prot;
        }
        self.prot = prot;
    }

    #[inline]
    fn split_to(&mut self, offs: usize) -> Self {
        assert!(
            offs < self.size,
            "splitting at an offset past it's current boundary"
        );
        let region = Self {
            size: offs,
            kind: self.kind.clone(),
            base: self.base,
            prot: self.prot,
            max_prot: self.max_prot,
            populated: self.populated,
        };
        self.size -= offs;
        self.base.virt += offs;
        self.base.phys += offs;
        region
    }

    #[inline]
    fn split_off(&mut self, offs: usize) -> Self {
        assert!(
            offs < self.size,
            "splitting off more spaces than it currently have"
        );
        let new_base = VmBase {
            phys: self.base.phys + offs,
            virt: self.base.virt + offs,
        };
        let region = Self {
            size: self.size - offs,
            kind: self.kind.clone(),
            base: new_base,
            prot: self.prot,
            max_prot: self.max_prot,
            populated: self.populated,
        };
        self.size -= offs;
        region
    }
}

impl Drop for VmRegion {
    #[inline]
    fn drop(&mut self) {
        if self.kind.is_regular() {
            Vm::unmap(self.base.phys, self.size);
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
        let item = VmRegion {
            base,
            kind,
            size,
            prot,
            max_prot,
            populated: prefault,
        };

        /* remove existing mappings before adding new one */
        self.unmap_range(addr, size);
        item.init();
        self.maps.insert(addr, item);

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
            .take_while(|&(.., r)| r.end() > addr)
            .map(|(&p, ..)| p)
            .collect::<SmallVec<[Uintptr; 16]>>();
        for base in keys {
            let mut region = self
                .maps
                .remove(&base)
                .unwrap_or_else(|| unsafe { std::intrinsics::unreachable() });
            if base < addr {
                let head = region.split_to(addr - base);
                self.maps.insert(base, head);
            }
            if region.end() > addr + size {
                let tail = region.split_off(addr + size - region.base.virt);
                self.maps.insert(addr + size, tail);
            }
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
        let mut rbuf = SmallVec::<[VmRegion; 2]>::new();
        let mut iter = self.maps.range(..addr + size);

        /* get the first covered region, make sure the address range doesn't have trailing gaps */
        let mut head = {
            if let Some((&head, region)) = iter.next_back() {
                if region.end() < addr + size {
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
            if region.end() <= addr {
                break;
            } else if region.end() != head {
                return Err(IoError::from_raw_os_error(libc::ENOMEM));
            } else if !prot.difference(region.max_prot).is_empty() {
                return Err(IoError::from_raw_os_error(libc::EACCES));
            } else {
                head = base;
            }
        }

        /* scan through the regions, split regions as needed */
        for (&base, region) in self.maps.range_mut(head..addr + size).rev() {
            if region.end() <= addr {
                break;
            }
            if region.populated {
                tlbi = true;
            }
            if region.end() > addr + size {
                rbuf.push(region.split_off(addr + size - base));
            }
            if base < addr {
                let mut head = region.split_off(addr - base);
                head.update(prot, set_max);
                rbuf.push(head);
            } else {
                region.update(prot, set_max);
            }
        }

        /* handle block splitting */
        for region in rbuf {
            self.maps.insert(region.base.virt, region);
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
    fn find_mmio(&mut self, pc: Uintptr, addr: Uintptr) -> Option<Arc<dyn MmioHandler>> {
        let (.., region) = self
            .maps
            .range_mut(..=addr)
            .next_back()
            .filter(|(.., r)| r.end() > addr)?;
        let VmKind::Mmio(mmio) = &region.kind else {
            panic!(
                "unexpected MMIO fault\nAddress:\n  {addr:p}\nInstruction:\n  {insn}",
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
        if region.end() <= addr {
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
        let phys = region.base.phys + (addr - region.base.virt);
        self.page.set(addr, phys, region.prot, region.max_prot);
        region.populated = true;
        true
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
            "unaligned map addresses: {addr:p}:{phys:p}"
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
    pub fn handle_mmio(mcx: MmioContext<'_>) -> bool {
        match { VMM.lock().find_mmio(mcx.pc, mcx.addr) } {
            Some(mmio) => mmio.handle(mcx),
            None => false,
        }
    }

    #[inline]
    pub fn handle_page_fault(addr: Uintptr, kind: MmioKind) -> bool {
        VMM.lock().page_fault_in(addr, kind)
    }
}
