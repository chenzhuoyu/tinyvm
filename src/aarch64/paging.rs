use parking_lot::Mutex;

use super::{
    regs::{SCTLR_EL1, TCR_EL1},
    vm::Vm,
};
use crate::{
    macros::define_bit_field,
    mem::{Addressable, Memory, Protection},
    utils::ptr::Uintptr,
};

define_bit_field! {
    /// Table descriptor for 16 KiB page.
    ///
    /// Valid at levels 0 through 2.
    struct TableDescriptor : u64 {
        /// Type Specifier, fixed to `0b11`.
        kind: 2,

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

    /// Page or Block descriptor for 16 KiB page.
    ///
    /// Block entries are valid at levels 0 through 2, only page entries are
    /// valid at level 3.
    struct PageOrBlockDescriptor : u64 {
        /// Type Specifier.
        ///
        /// Block entries at levels 0 through 2 have these bits set to `0b01`,
        /// page entries at level 3 are set to `0b11`.
        kind: 2,

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
        unused: 4,

        /// Implementation Defined.
        imp_def: 4,

        /// Reserved, set to zero.
        res0c: 1,
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

union Entry {
    table: TableDescriptor,
    block: PageOrBlockDescriptor,
}

impl Entry {
    #[inline]
    fn next_table(&mut self) -> &mut [Self; ENTRY_COUNT] {
        match unsafe { self.table.kind() } {
            0b00 => {
                let mem = Memory::alloc(PAGE_SIZE).map(Protection::RW);
                let tab = mem.addr();

                /* calculate next table address */
                let next = {
                    assert!(tab.is_aligned_to(PAGE_SIZE));
                    tab.as_u64() / (PAGE_SIZE as u64)
                };

                /* build the table entry */
                self.table = TableDescriptor::builder()
                    .kind(0b11)
                    .next_table_addr(next)
                    .build();

                /* make the page table permanent */
                std::mem::forget(mem);
                tab.as_mut()
            }
            0b11 => {
                let addr = unsafe { self.table.next_table_addr() };
                Uintptr::from(addr * (PAGE_SIZE as u64)).as_mut()
            }
            0b01 => unsafe { panic!("not a table entry: {:#?}", self.block) },
            0b10 => unsafe { panic!("invalid table entry: {:#?}", self.table) },
            _ => unreachable!(),
        }
    }

    #[inline]
    fn create_page(&mut self, addr: Uintptr, prot: Protection) {
        match unsafe { self.block.kind() } {
            0b00 => {}
            0b01 => unsafe { panic!("not a page entry: {:#?}", self.table) },
            0b10 => unsafe { panic!("invalid page entry: {:#?}", self.block) },
            0b11 => unsafe { panic!("overlapping page entry: {:#?}", self.block) },
            _ => unreachable!(),
        }
        let (ap, nx) = match prot {
            Protection::RW => (0b01, 1),
            Protection::RX => (0b11, 0),
            Protection::READ => (0b11, 0),
            Protection::NONE => (0b10, 1),
            _ => panic!("invalid selection of prot bits: {prot:?}"),
        };
        self.block = PageOrBlockDescriptor::builder()
            .kind(0b11)
            .AttrIndx(0)
            .NS(1)
            .AP(ap)
            .SH(0b11)
            .AF(1)
            .nG(0)
            .phys_addr(addr.as_u64() / (PAGE_SIZE as u64))
            .DBM(0)
            .Contig(0)
            .PXN(nx)
            .UXN(nx)
            .build();
    }
}

const ENTRY_SIZE: usize = std::mem::size_of::<Entry>();
const ENTRY_COUNT: usize = PAGE_SIZE / ENTRY_SIZE;

#[repr(transparent)]
pub struct PageTable([Entry; ENTRY_COUNT]);

static PAGE_LOCK: Mutex<()> = Mutex::new(());
static mut PAGE_TABLE: *mut PageTable = std::ptr::null_mut();

impl PageTable {
    pub(super) fn init(base: Uintptr) {
        unsafe {
            assert_eq!(libc::vm_page_size, PAGE_SIZE);
            PAGE_TABLE = base.as_ptr();
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
    fn add_region(&mut self, mut addr: Uintptr, mut size: usize, prot: Protection) {
        while size >= PAGE_SIZE {
            let (l1, l2, l3) = Self::index(addr);
            self.0[l1].next_table()[l2].next_table()[l3].create_page(addr, prot);
            addr += PAGE_SIZE;
            size -= PAGE_SIZE;
        }
        assert!(
            size == 0,
            "memory region size is not a multiple of page size: {size}"
        );
    }
}

impl Vm {
    #[inline]
    pub fn page_table(&self) -> Uintptr {
        unsafe { Uintptr::from(PAGE_TABLE) }
    }

    #[inline]
    pub fn register_pages(&self, addr: Uintptr, size: usize, prot: Protection) {
        unsafe {
            let _m = PAGE_LOCK.lock();
            assert!(!PAGE_TABLE.is_null());
            (*PAGE_TABLE).add_region(addr, size, prot);
        }
    }
}
