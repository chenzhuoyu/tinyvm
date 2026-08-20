use crate::{
    aarch64::{
        regs::{SCTLR_EL1, TCR_EL1},
        vm::Vm,
    },
    macros::define_bit_field,
    mem::Protection,
    utils::ptr::Uintptr,
};

define_bit_field! {
    struct L3Ptr : usize {
        offs: 14,
        idx3: 11,
        idx2: 11,
        idx1: 11,
        sign: 17,
    }

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

impl Protection {
    #[inline]
    fn from_ap_nx(ap: u64, nx: u64) -> Self {
        match (ap, nx) {
            (0b01, 0) => Self::all(),
            (0b01, 1) => Self::RW,
            (0b10, 0) => Self::EXEC,
            (0b10, 1) => Self::NONE,
            (0b11, 0) => Self::RX,
            (0b11, 1) => Self::READ,
            _ => panic!("invalid selectop of AP & NX bits: {ap:02b}:{nx}"),
        }
    }
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
    const INVALID: Self = Self { bits: 0 };
}

impl Entry {
    #[inline]
    fn page_mut(&mut self) -> Option<&mut PageDescriptor> {
        unsafe {
            if self.page.valid() == 1 {
                assert!(self.page.ty() == 1, "invalid L3 page entry");
                Some(&mut self.page)
            } else {
                None
            }
        }
    }

    #[inline]
    fn table_mut(&mut self) -> Option<&mut [Self; ENTRY_COUNT]> {
        unsafe {
            if self.table.valid() == 1 {
                Some(self.table_mut_unchecked())
            } else {
                None
            }
        }
    }

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

impl Entry {
    #[inline]
    fn set_page(&mut self, phys: Uintptr, prot: Protection, max_prot: Protection) {
        self.page = PageDescriptor::builder()
            .valid(1)
            .ty(1)
            .AttrIndx(0)
            .NS(1)
            .AP(prot.ap())
            .SH(0b11)
            .AF(1)
            .nG(0)
            .phys_addr(phys.as_u64() / (PAGE_SIZE as u64))
            .DBM(0)
            .Contig(0)
            .PXN(prot.nx())
            .UXN(prot.nx())
            .user_data(max_prot.bits())
            .build();
    }

    #[inline]
    fn set_table(&mut self) -> &mut [Self; ENTRY_COUNT] {
        unsafe {
            if self.table.valid() == 0 {
                let next = Vm::alloc(PAGE_SIZE);
                Vm::map(next, PAGE_SIZE, Protection::RW);
                self.table = TableDescriptor::new(next.as_u64() / (PAGE_SIZE as u64));
                next.as_mut()
            } else {
                assert!(self.table.ty() == 1, "invalid table descriptor");
                self.table_mut_unchecked()
            }
        }
    }
}

const ENTRY_SIZE: usize = std::mem::size_of::<Entry>();
const ENTRY_COUNT: usize = PAGE_SIZE / ENTRY_SIZE;

#[repr(transparent)]
pub struct PageTable([Entry; ENTRY_COUNT]);

impl PageTable {
    #[inline(always)]
    const fn index(addr: Uintptr) -> (usize, usize, usize) {
        let ptr = L3Ptr(addr.addr());
        assert!(ptr.sign() == 0);
        (ptr.idx1(), ptr.idx2(), ptr.idx3())
    }
}

impl PageTable {
    fn walk_pages_mut(&mut self, addr: Uintptr, num_pages: usize, mut f: impl FnMut(&mut [Entry])) {
        let (p1, p2, p3) = Self::index(addr + num_pages * PAGE_SIZE - 1);
        let (mut l1, mut l2, mut l3) = Self::index(addr);

        /* unset all pages */
        while l1 <= p1 && l2 <= p2 && l3 <= p3 {
            if let Some(t1) = self.0[l1].table_mut() {
                if let Some(t2) = t1[l2].table_mut() {
                    if l1 == p1 && l2 == p2 {
                        f(&mut t2[l3..=p3]);
                    } else {
                        f(&mut t2[l3..]);
                    }
                }
                l2 += 1;
                l1 += l2 / ENTRY_COUNT;
                l2 %= ENTRY_COUNT;
                l3 = 0;
            } else {
                l1 += 1;
                l2 = 0;
                l3 = 0;
            }
        }
    }
}

impl PageTable {
    pub fn lookup(&self, addr: Uintptr) -> Protection {
        unsafe {
            let (l1, l2, l3) = Self::index(addr);
            let page = self.0[l1].table_unchecked()[l2].table_unchecked()[l3].page;
            Protection::from_ap_nx(page.AP(), page.UXN())
        }
    }
}

impl PageTable {
    pub fn set(&mut self, addr: Uintptr, prot: Protection, max_prot: Protection) {
        let (l1, l2, l3) = Self::index(addr);
        let next = self.0[l1].set_table();
        let next = next[l2].set_table();
        next[l3].set_page(addr, prot, max_prot);
    }

    pub fn unset(&mut self, addr: Uintptr, num_pages: usize) {
        self.walk_pages_mut(addr, num_pages, |entries| entries.fill(Entry::INVALID));
    }

    pub fn protect(&mut self, addr: Uintptr, num_pages: usize, prot: Protection) {
        self.walk_pages_mut(addr, num_pages, |entries| {
            for entry in entries {
                if let Some(page) = entry.page_mut() {
                    page.set_AP(prot.ap());
                    page.set_PXN(prot.nx());
                    page.set_UXN(prot.nx());
                }
            }
        });
    }

    pub fn prefault(
        &mut self,
        mut addr: Uintptr,
        end: Uintptr,
        prot: Protection,
        max_prot: Protection,
    ) {
        while addr < end {
            self.set(addr, prot, max_prot);
            addr += PAGE_SIZE;
        }
    }
}
