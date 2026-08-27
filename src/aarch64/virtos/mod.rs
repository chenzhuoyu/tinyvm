pub mod commpage;
pub mod faults;
pub mod mem;
pub mod mmio;
pub mod pac;
pub mod shared_cache;
pub mod syscall;
pub mod tlb;

use crate::{
    aarch64::{paging::PAGE_SIZE, vm::Vm},
    mem::Protection,
    utils::{
        ptr::{Uintptr, VMA},
        size::align_to_page,
    },
};

pub const SP_EL1: VMA = VMA::new(0x3fff_ffff_fff0);
pub const EL1_STACK: VMA = VMA::new(0x3fff_ffff_c000);
pub const IRQ_STUBS: VMA = VMA::new(0x3fff_ffff_0000);

unsafe extern "C" {
    unsafe static virtos_end: u8;
    unsafe static virtos_start: u8;
}

/// EL1 stack top, host address.
static mut STACK_TOP: Uintptr = Uintptr::NIL;

#[inline]
fn virtos_code() -> &'static [u8] {
    unsafe {
        let end = &raw const virtos_end;
        let start = &raw const virtos_start;
        std::slice::from_raw_parts(start, end.offset_from_unsigned(start))
    }
}

pub fn init() {
    let code = virtos_code();
    let size = align_to_page(code.len());

    /* initialize the page table and memory manager */
    assert!(size <= PAGE_SIZE * 3, "virtos is too large");
    mem::VmMap::init();
    commpage::init();

    /* allocate memory for IRQ stubs and EL1 stack */
    let irq_stubs = Vm::alloc(size);
    let el1_stack = Vm::alloc(PAGE_SIZE);

    /* load the IRQ stubs into memory */
    unsafe {
        STACK_TOP = el1_stack + PAGE_SIZE - 16;
        std::ptr::write_bytes(irq_stubs.as_ptr::<u8>(), 0, size);
        std::ptr::copy_nonoverlapping(code.as_ptr(), irq_stubs.as_ptr(), code.len());
    }

    /* insert IRQ stubs into the VM map */
    mem::VmMap::insert(
        mem::VmKind::Regular,
        irq_stubs,
        IRQ_STUBS,
        size,
        Protection::RX,
        Protection::RX,
        true,
    );

    /* log the IRQ stubs range */
    tracing::debug!(
        "IRQ Stubs are loaded into {IRQ_STUBS:p}-{end:p}",
        end = IRQ_STUBS + code.len()
    );

    /* insert EL1 stack into the VM map */
    mem::VmMap::insert(
        mem::VmKind::Regular,
        el1_stack,
        EL1_STACK,
        PAGE_SIZE,
        Protection::RW,
        Protection::RW,
        true,
    );

    /* log the IRQ stubs range */
    tracing::debug!(
        "EL1 stack is loaded into {EL1_STACK:p}-{end:p}, SP_EL1={SP_EL1:p}",
        end = EL1_STACK + PAGE_SIZE
    );
}

#[inline]
pub fn stack_top() -> Uintptr {
    unsafe { STACK_TOP }
}
