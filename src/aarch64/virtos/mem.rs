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
    utils::{
        ptr::{Uintptr, VMA},
        size::align_to_page,
    },
};

const VMA_MIN: VMA = VMA::new(0x0001_0000_0000);
const VMA_MAX: VMA = VMA::new(0x4000_0000_0000);

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
    next: VMA,
    kind: VmKind,
    base: Uintptr,
    prot: Protection,
    max_prot: Protection,
    populated: bool,
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

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct VmFlags : u32 {
        const FIXED    = 1 << 0;
        const PREFAULT = 1 << 1;
    }
}

#[derive(Debug, Clone, Copy)]
struct ProtAndFlags {
    prot: Protection,
    max_prot: Protection,
    map_flags: VmFlags,
}

pub struct VmMap {
    free: BTreeMap<VMA, VMA>,
    used: BTreeMap<VMA, VmRegion>,
    page: PageTableRef,
}

unsafe impl Send for VmMap {}
unsafe impl Sync for VmMap {}

static VMM: Mutex<VmMap> = Mutex::new(VmMap {
    free: BTreeMap::new(),
    used: BTreeMap::new(),
    page: PageTableRef(std::ptr::null_mut()),
});

impl VmMap {
    pub fn init() {
        unsafe {
            assert!(
                libc::vm_page_size == PAGE_SIZE,
                "page size went bananas: {size} != {PAGE_SIZE}",
                size = libc::vm_page_size,
            );
        }

        /* allocate memory for page table */
        let tab = Vm::alloc(PAGE_SIZE);
        let vmm = unsafe { &mut *VMM.data_ptr() };

        /* initialize the virtual memory allocator */
        Vm::map(tab, PAGE_SIZE, Protection::READ);
        vmm.free.insert(VMA_MIN, VMA_MAX);
        vmm.page.0 = tab.as_ptr();
    }

    pub fn base() -> Uintptr {
        let vmm = unsafe { &(*VMM.data_ptr()) };
        assert!(!vmm.page.0.is_null(), "VM is not initialized");
        Uintptr::from(vmm.page.0)
    }
}

impl VmMap {
    fn pop_free(&mut self, size: usize) -> Option<VMA> {
        let (&base, &next) = self
            .free
            .iter()
            .find(|&(&base, &next)| next - base >= size)?;
        assert!(
            self.free.remove(&base).is_some(),
            "cannot remove items just found"
        );
        if base + size < next {
            self.push_free(base + size, next);
        }
        Some(base)
    }

    fn pop_hint(&mut self, hint: VMA, size: usize) -> Option<VMA> {
        let (&base, &next) = self
            .free
            .range(..=hint)
            .next_back()
            .filter(|&(.., &q)| q >= hint + size)?;
        assert!(
            self.free.remove(&base).is_some(),
            "cannot remove items just found"
        );
        if base < hint {
            self.push_free(base, hint);
        }
        if hint + size < next {
            self.push_free(hint + size, next);
        }
        Some(hint)
    }

    fn push_free(&mut self, start: VMA, mut end: VMA) {
        if let Some(next) = self.free.remove(&end) {
            end = next;
        }
        let prev = self
            .free
            .range_mut(..start)
            .next_back()
            .filter(|(.., next)| **next == start);
        if let Some((.., next)) = prev {
            *next = end;
        } else {
            self.free.insert(start, end);
        }
    }
}

impl VmMap {
    fn map_range(
        &mut self,
        kind: VmKind,
        base: Uintptr,
        hint: VMA,
        size: usize,
        meta: ProtAndFlags,
    ) -> IoResult<VMA> {
        let fixed = meta.map_flags.contains(VmFlags::FIXED);
        let mut addr = self.pop_hint(hint, size);

        /* if user does not require mapping at fixed address, try pick one on our own */
        if !fixed && addr.is_none() {
            addr = self.pop_free(size);
        }

        /* should have got an available virtual address */
        let Some(addr) = addr else {
            return Err(IoError::from_raw_os_error(libc::ENOMEM));
        };

        /* construct the region */
        let region = VmRegion {
            next: addr + size,
            kind,
            base,
            prot: meta.prot,
            max_prot: meta.max_prot,
            populated: meta.map_flags.contains(VmFlags::PREFAULT),
        };

        /* map the memory region into VM if it's regular memory */
        if region.kind.is_regular() {
            Vm::map(base, size, meta.prot);
        }

        /* prefault the pages if needed */
        if region.populated {
            self.page
                .prefault(addr, base, addr + size, meta.prot, meta.max_prot);
        }

        /* add to used mappings */
        self.used.insert(addr, region);
        Ok(addr)
    }

    fn unmap_range(&mut self, addr: VMA, size: usize) {
        let keys = self
            .used
            .range(..addr + size)
            .rev()
            .take_while(|&(.., r)| r.next > addr)
            .map(|(&p, ..)| p)
            .collect::<SmallVec<[VMA; 16]>>();
        for base in keys {
            let region = self
                .used
                .remove(&base)
                .unwrap_or_else(|| unsafe { std::intrinsics::unreachable() });
            if base < addr {
                self.used.insert(base, {
                    VmRegion {
                        next: addr,
                        kind: region.kind.clone(),
                        base: region.base,
                        prot: region.prot,
                        max_prot: region.max_prot,
                        populated: region.populated,
                    }
                });
            }
            if region.next > addr + size {
                self.used.insert(addr + size, {
                    VmRegion {
                        next: region.next,
                        kind: region.kind.clone(),
                        base: region.base + (addr + size - base),
                        prot: region.prot,
                        max_prot: region.max_prot,
                        populated: region.populated,
                    }
                });
            }
            if region.populated || region.kind.is_regular() {
                let (addr, size) = (
                    region.base + (addr - base),
                    region.next.min(addr + size) - addr,
                );
                let should_dealloc = {
                    Vm::unmap(addr, size);
                    region.kind.is_regular()
                };
                if should_dealloc {
                    Vm::dealloc(addr, size);
                }
            }
        }
        self.page.unset(addr, size / PAGE_SIZE);
        self.push_free(addr, addr + size);
    }

    fn protect_range(
        &mut self,
        cpu: &Cpu,
        addr: VMA,
        size: usize,
        prot: Protection,
        set_max: bool,
    ) -> IoResult<()> {
        let mut tlbi = false;
        let mut iter = self.used.range(..addr + size);
        let mut rbuf = SmallVec::<[(VMA, VmRegion); 2]>::new();

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
        for (&base, region) in self.used.range_mut(head..addr + size).rev() {
            if region.next <= addr {
                break;
            }
            if region.populated {
                tlbi = true;
            }
            if addr + size < region.next {
                let tail = VmRegion {
                    next: region.next,
                    kind: region.kind.clone(),
                    base: region.base + (addr + size - base),
                    prot: region.prot,
                    max_prot: region.max_prot,
                    populated: region.populated,
                };
                rbuf.push((addr + size, tail));
                region.next = addr + size;
            }
            if base < addr {
                let lead = VmRegion {
                    next: region.next,
                    kind: region.kind.clone(),
                    base: region.base,
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
            self.used.insert(base, region);
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

    fn populate_range(&mut self, cpu: &Cpu, virt: VMA, size: usize) -> Option<Uintptr> {
        let (phys, prot) = self
            .page
            .lookup(virt)
            .or_else(|| self.touch_phys_address(cpu, virt))?;
        Some(phys)
    }

    #[cold]
    fn touch_phys_address(&mut self, cpu: &Cpu, virt: VMA) -> Option<(Uintptr, Protection)> {
        let (&base, region) = self
            .used
            .range_mut(..=virt)
            .next_back()
            .filter(|(.., r)| r.next > virt)?;
        let phys = {
            assert!(virt.is_aligned_to(PAGE_SIZE));
            region.base + (virt - base)
        };
        self.page.set(virt, phys, region.prot, region.max_prot);
        cpu.flush_tlb(virt, 1);
        region.populated = true;
        Some((phys, region.prot))
    }
}

impl VmMap {
    #[inline]
    fn find_mmio(&mut self, pc: VMA, req: &MmioRequest) -> Option<Arc<dyn MmioHandler>> {
        let (.., region) = self
            .used
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
    fn page_fault_in(&mut self, addr: VMA, kind: MmioKind) -> bool {
        let Some((&base, region)) = self.used.range_mut(..=addr).next_back() else {
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
        self.page.set(
            addr,
            region.base + (addr - base),
            region.prot,
            region.max_prot,
        );
        region.populated = true;
        true
    }
}

impl VmMap {
    #[inline]
    pub fn map(
        kind: impl Into<VmKind>,
        base: Uintptr,
        hint: VMA,
        size: usize,
        prot: Protection,
        max_prot: Protection,
        map_flags: VmFlags,
    ) -> IoResult<VMA> {
        assert!(
            hint.is_aligned_to(PAGE_SIZE),
            "unaligned map hint address: {hint:p}"
        );
        let meta = ProtAndFlags {
            prot,
            max_prot,
            map_flags,
        };
        VMM.lock()
            .map_range(kind.into(), base, hint, align_to_page(size), meta)
    }

    #[inline]
    pub fn unmap(addr: VMA, size: usize) {
        assert!(
            addr.is_aligned_to(PAGE_SIZE),
            "unaligned unmap base address: {addr:p}"
        );
        VMM.lock().unmap_range(addr, align_to_page(size));
    }

    #[inline]
    pub fn protect(
        cpu: &Cpu,
        addr: VMA,
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

    #[inline]
    pub fn populate(cpu: &Cpu, virt: VMA, size: usize) -> Option<Uintptr> {
        VMM.lock()
            .populate_range(cpu, virt.align_down(PAGE_SIZE), align_to_page(size))
    }
}

impl VmMap {
    #[inline]
    pub fn insert(
        kind: impl Into<VmKind>,
        base: Uintptr,
        addr: VMA,
        size: usize,
        prot: Protection,
        max_prot: Protection,
        prefault: bool,
    ) {
        let map_flags = {
            if prefault {
                VmFlags::FIXED | VmFlags::PREFAULT
            } else {
                VmFlags::FIXED
            }
        };
        Self::map(kind, base, addr, size, prot, max_prot, map_flags)
            .expect("cannot insert VM mappings");
    }

    #[inline]
    pub fn translate(virt: VMA) -> Option<Uintptr> {
        Some(VMM.lock().page.lookup(virt)?.0)
    }
}

impl VmMap {
    #[inline]
    pub fn handle_mmio(pc: VMA, req: &mut MmioRequest) -> Option<MmioResponse> {
        let mmio = VMM.lock().find_mmio(pc, req)?;
        Some(mmio.handle(pc, req))
    }

    #[inline]
    pub fn handle_page_fault(addr: VMA, kind: MmioKind) -> bool {
        VMM.lock().page_fault_in(addr, kind)
    }
}
