use std::{
    error::{Error, Request as ErrorRequest},
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
};

use parking_lot::Mutex;

use super::regs::{SCTLR_EL1, TCR_EL1};
use crate::{
    macros::define_bit_field,
    mem::{Memory, Protection},
    utils::ptr::Uintptr,
};

define_bit_field! {
    /// Page or Block descriptor for 16 KiB page.
    ///
    /// Block entries are valid at levels 0 through 2, only page entries are
    /// valid at level 3.
    struct PageDescriptor : u64 {
        /// Valid bit.
        valid: 1,

        /// Type Specifier.
        ///
        ///   * `0`: Block descriptor for lookup levels less than 3.
        ///   * `1`: Page descriptor for lookup level 3.
        ty: 1,

        /// Memory attribute index.
        AttrIndx: 3,

        /// Non-secure.
        NS: 1,

        /// Access permissions.
        AP: 2,

        /// Sharability.
        SH: 2,

        /// Access flag.
        AF: 1,

        /// Non-global.
        nG: 1,

        /// Reserved, set to zero.
        res0a: 2,

        /// Output block address.
        phys_addr: 34,

        /// Reserved, set to zero.
        res0b: 3,

        /// Dirty bit modifier.
        DBM: 1,

        /// Contiguous.
        Contig: 1,

        /// Privileged execution never.
        PXN: 1,

        /// User execution never.
        UXN: 1,

        /// Reserved for software use.
        user_data: 4,

        /// Implementation Defined.
        imp_def: 4,

        /// Reserved, set to zero.
        res0c: 1,
    }

    /// Table descriptor for 16 KiB page.
    ///
    /// Valid at levels 0 through 2.
    struct TableDescriptor : u64 {
        /// Valid bit.
        valid: 1,

        /// Type Specifier, must be 1.
        ty: 1,

        /// Ignored.
        unused0: 10,

        /// Reserved, set to zero.
        res0a: 2,

        /// Next Table Address.
        next_table_addr: 34,

        /// Reserved, set to zero.
        res0b: 4,

        /// Ignored.
        unused1: 7,

        /// Hierarchical privileged execute never.
        PXNTable: 1,

        /// Hierarchical user execute never.
        XNTable: 1,

        /// Hierarchical access permissions.
        APTable: 2,

        /// Hierarchical "not secure" flag.
        NSTable: 1,
    }
}

impl TableDescriptor {
    fn new(next: u64) -> Self {
        TableDescriptor::builder()
            .valid(1)
            .ty(1)
            .next_table_addr(next)
            .build()
    }
}

pub const PAGE_SIZE: usize = 16384;
pub const MAIR_EL1_INIT: u64 = 0xff;

pub const TCR_EL1_INIT: u64 = {
    TCR_EL1::builder()
        .IPS(0b101) // 48-bit IPA
        .EPD1(1) // Disable TTBR1_EL1
        .TG0(0b10) // Page size is 16 KiB
        .SH0(0b11) // Inner sharable
        .ORGN0(0b01) // Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.
        .IRGN0(0b01) // Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable.
        .T0SZ(17) // 47-bit virtual address
        .build()
        .value()
};

pub const SCTLR_EL1_INIT: u64 = {
    SCTLR_EL1::builder()
        .I(1) // Enable instruction cache
        .SED(1) // Disbale SETEND instruction
        .ITD(1) // Disable IT instruction
        .C(1) // Enable data cache
        .M(1) // Enable MMU
        .build()
        .value()
};

trait AsPageAttrs {
    fn ap(self) -> u64;
    fn nx(self) -> u64;
}

impl AsPageAttrs for Protection {
    #[inline]
    fn ap(self) -> u64 {
        match self {
            Protection::RW => 0b01,
            Protection::RX => 0b11,
            Protection::READ => 0b11,
            Protection::NONE => 0b10,
            _ => panic!("invalid selection of protection bits: {self:?}"),
        }
    }

    #[inline]
    fn nx(self) -> u64 {
        !self.contains(Protection::EXEC) as u64
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
union Entry {
    bits: u64,
    page: PageDescriptor,
    table: TableDescriptor,
}

impl Entry {
    #[inline]
    fn set_page(&mut self, addr: Uintptr, prot: Protection, max_prot: Protection) {
        unsafe {
            assert!(
                self.page.valid() == 0,
                "overlapping page entry at {:p}: {:#?}",
                addr,
                self.page,
            );
        }
        self.page = PageDescriptor::builder()
            .valid(1)
            .ty(1)
            .AttrIndx(0)
            .NS(1)
            .AP(prot.ap())
            .SH(0b11)
            .AF(1)
            .nG(0)
            .phys_addr(addr.as_u64() / (PAGE_SIZE as u64))
            .DBM(0)
            .Contig(0)
            .PXN(prot.nx())
            .UXN(prot.nx())
            .user_data(max_prot.bits())
            .build();
    }

    #[inline]
    fn set_table(&mut self, level: FaultLevel) -> &mut [Self; ENTRY_COUNT] {
        if self.try_as_table_mut(level).is_err() {
            let next = Memory::alloc(PAGE_SIZE).map(Protection::RW).into_parts().0;
            self.table = TableDescriptor::new(next.as_u64() / (PAGE_SIZE as u64));
            next.as_mut()
        } else {
            unsafe { self.table_mut_unchecked() }
        }
    }
}

impl Entry {
    #[inline]
    fn try_as_page(&self) -> PageResult<PageDescriptor> {
        unsafe {
            if self.page.valid() == 1 {
                assert!(self.page.ty() == 1, "invalid L3 page entry");
                Ok(self.page)
            } else {
                Err(PageFault::enomem(FaultLevel::L3))
            }
        }
    }

    #[inline]
    fn try_as_table(&self, level: FaultLevel) -> PageResult<&[Self; ENTRY_COUNT]> {
        unsafe {
            if self.table.valid() == 1 {
                assert!(self.table.ty() == 1, "invalid {level:?} table descriptor");
                Ok(self.table_unchecked())
            } else {
                Err(PageFault::enomem(level))
            }
        }
    }

    #[inline]
    fn try_as_table_mut(&mut self, level: FaultLevel) -> PageResult<&mut [Self; ENTRY_COUNT]> {
        unsafe {
            if self.table.valid() == 1 {
                assert!(self.table.ty() == 1, "invalid {level:?} table descriptor");
                Ok(self.table_mut_unchecked())
            } else {
                Err(PageFault::enomem(level))
            }
        }
    }
}

impl Entry {
    #[inline]
    unsafe fn table_unchecked(&self) -> &[Self; ENTRY_COUNT] {
        let addr = unsafe { self.table.next_table_addr() };
        Uintptr::from(addr * (PAGE_SIZE as u64)).as_ref()
    }

    #[inline]
    unsafe fn table_mut_unchecked(&mut self) -> &mut [Self; ENTRY_COUNT] {
        let addr = unsafe { self.table.next_table_addr() };
        Uintptr::from(addr * (PAGE_SIZE as u64)).as_mut()
    }
}

const ENTRY_SIZE: usize = std::mem::size_of::<Entry>();
const ENTRY_COUNT: usize = PAGE_SIZE / ENTRY_SIZE;

pub type PageUnit = PageResult<()>;
pub type PageResult<T> = Result<T, PageFault>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultLevel {
    L1,
    L2,
    L3,
}

#[derive(Debug)]
pub struct PageFault {
    pub error: IoError,
    pub level: FaultLevel,
}

impl PageFault {
    #[inline]
    fn new(code: i32, level: FaultLevel) -> Self {
        Self {
            error: IoError::from_raw_os_error(code),
            level,
        }
    }

    #[inline]
    fn eacces(level: FaultLevel) -> Self {
        Self::new(libc::EACCES, level)
    }

    #[inline]
    fn eexist(level: FaultLevel) -> Self {
        Self::new(libc::EEXIST, level)
    }

    #[inline]
    fn einval(level: FaultLevel) -> Self {
        Self::new(libc::EINVAL, level)
    }

    #[inline]
    fn enomem(level: FaultLevel) -> Self {
        Self::new(libc::ENOMEM, level)
    }
}

impl Error for PageFault {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }

    #[inline]
    fn provide<'a>(&'a self, req: &mut ErrorRequest<'a>) {
        req.provide_ref(&self.error);
    }
}

impl Display for PageFault {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "page fault at level {:?}: {}", self.level, self.error)
    }
}

impl From<PageFault> for IoError {
    #[inline]
    fn from(error: PageFault) -> Self {
        error.error
    }
}

#[repr(transparent)]
pub struct PageTable([Entry; ENTRY_COUNT]);

static PAGE_LOCK: Mutex<()> = Mutex::new(());
static mut PAGE_TABLE: *mut PageTable = std::ptr::null_mut();

impl PageTable {
    pub(super) fn init() {
        unsafe {
            assert!(
                libc::vm_page_size == PAGE_SIZE,
                "page size went bananas: {} != {}",
                libc::vm_page_size,
                PAGE_SIZE,
            );
            PAGE_TABLE = Memory::alloc(PAGE_SIZE)
                .map(Protection::RW)
                .into_parts()
                .0
                .as_ptr();
        }
    }
}

impl PageTable {
    #[inline(always)]
    const fn index(addr: Uintptr) -> (usize, usize, usize) {
        define_bit_field! {
            struct L3Ptr : usize {
                offs: 14,
                idx3: 11,
                idx2: 11,
                idx1: 11,
                sign: 17,
            }
        }
        let ptr = L3Ptr(addr.addr());
        assert!(ptr.sign() == 0);
        (ptr.idx1(), ptr.idx2(), ptr.idx3())
    }
}

impl PageTable {
    #[inline]
    fn page_of(&self, addr: Uintptr) -> Result<PageDescriptor, PageFault> {
        let (l1, l2, l3) = Self::index(addr);
        let next = self.0[l1].try_as_table(FaultLevel::L1)?;
        let next = next[l2].try_as_table(FaultLevel::L2)?;
        next[l3].try_as_page()
    }
}

impl PageTable {
    fn do_map(
        &mut self,
        mut addr: Uintptr,
        num_pages: usize,
        prot: Protection,
        max_prot: Protection,
    ) -> PageUnit {
        for i in 0..num_pages {
            if self.page_of(addr + i * PAGE_SIZE).is_ok() {
                return Err(PageFault::eexist(FaultLevel::L3));
            }
        }
        for _ in 0..num_pages {
            let (l1, l2, l3) = Self::index(addr);
            let next = self.0[l1].set_table(FaultLevel::L1);
            let next = next[l2].set_table(FaultLevel::L2);
            next[l3].set_page(addr, prot, max_prot);
            addr += PAGE_SIZE;
        }
        Ok(())
    }

    fn do_unmap(&mut self, mut addr: Uintptr, num_pages: usize) -> PageUnit {
        for i in 0..num_pages {
            self.page_of(addr + i * PAGE_SIZE)?;
        }
        for _ in 0..num_pages {
            let (l1, l2, l3) = Self::index(addr);
            unsafe { self.0[l1].table_mut_unchecked()[l2].table_mut_unchecked()[l3].bits = 0 }
            addr += PAGE_SIZE as u64;
        }
        Ok(())
    }

    fn do_protect(
        &mut self,
        mut addr: Uintptr,
        num_pages: usize,
        set_max: bool,
        prot: Protection,
    ) -> PageUnit {
        for i in 0..num_pages {
            self.page_of(addr + i * PAGE_SIZE)?;
        }
        for _ in 0..num_pages {
            let page = unsafe {
                let (l1, l2, l3) = Self::index(addr);
                &mut self.0[l1].table_mut_unchecked()[l2].table_mut_unchecked()[l3].page
            };
            if prot.bits() & !page.user_data() != 0 {
                return Err(PageFault::eacces(FaultLevel::L3));
            }
            if set_max {
                page.set_user_data(prot.bits());
            }
            page.set_AP(prot.ap());
            page.set_PXN(prot.nx());
            page.set_UXN(prot.nx());
            addr += PAGE_SIZE as u64;
        }
        Ok(())
    }
}

impl PageTable {
    #[inline]
    pub fn base() -> Uintptr {
        unsafe {
            assert!(!PAGE_TABLE.is_null(), "VM is not initialized");
            Uintptr::from(PAGE_TABLE)
        }
    }

    #[inline]
    pub fn remove(addr: Uintptr, size: usize) {
        if let Err(err) = Self::unmap(addr, size) {
            panic!("cannot remove page table entry: {err}");
        }
    }

    #[inline]
    pub fn insert(phys: Uintptr, size: usize, prot: Protection, max_prot: Protection) {
        if let Err(err) = Self::map(phys, size, prot, max_prot) {
            panic!("cannot insert page table entry: {err}");
        }
    }
}

impl PageTable {
    pub fn map(addr: Uintptr, size: usize, prot: Protection, max_prot: Protection) -> PageUnit {
        if let Some(tab) = unsafe { PAGE_TABLE.as_mut() } {
            if addr.addr().is_multiple_of(PAGE_SIZE) {
                let _lock = PAGE_LOCK.lock();
                tab.do_map(addr, size.div_ceil(PAGE_SIZE), prot, max_prot)
            } else {
                Err(PageFault::einval(FaultLevel::L1))
            }
        } else {
            panic!("Page Table is not initialized")
        }
    }

    pub fn unmap(addr: Uintptr, size: usize) -> PageUnit {
        if let Some(tab) = unsafe { PAGE_TABLE.as_mut() } {
            if addr.addr().is_multiple_of(PAGE_SIZE) {
                let _lock = PAGE_LOCK.lock();
                tab.do_unmap(addr, size.div_ceil(PAGE_SIZE))
            } else {
                Err(PageFault::einval(FaultLevel::L1))
            }
        } else {
            panic!("Page Table is not initialized")
        }
    }

    pub fn protect(addr: Uintptr, size: usize, prot: Protection, set_max: bool) -> PageUnit {
        if let Some(tab) = unsafe { PAGE_TABLE.as_mut() } {
            if addr.addr().is_multiple_of(PAGE_SIZE) {
                let _lock = PAGE_LOCK.lock();
                tab.do_protect(addr, size.div_ceil(PAGE_SIZE), set_max, prot)
            } else {
                Err(PageFault::einval(FaultLevel::L1))
            }
        } else {
            panic!("Page Table is not initialized")
        }
    }
}
