use super::{ffi::*, paging::PageTable};
use crate::{
    hv_call,
    mem::{Memory, Protection},
    utils::ptr::Uintptr,
};

unsafe extern "C" {
    unsafe static virtos_end: u8;
    unsafe static virtos_start: u8;
}

#[inline]
fn virtos_code() -> &'static [u8] {
    unsafe {
        let end = &raw const virtos_end;
        let start = &raw const virtos_start;
        std::slice::from_raw_parts(start, end.offset_from_unsigned(start))
    }
}

pub enum Vm {}
pub const IRQ_STUBS: Uintptr = Uintptr::new(0x7fff_0000_0000);

impl Vm {
    pub fn init() {
        let code = virtos_code();
        let vcfg = unsafe { hv_vm_config_create() };

        /* create VM without EL2 */
        hv_call!(hv_vm_config_set_el2_enabled(vcfg, false));
        hv_call!(hv_vm_create(vcfg));

        /* allocate memory for IRQ stubs */
        let page = Memory::from_data(code).map(Protection::RX);
        let (phys, size) = page.into_parts();

        /* initialize the page table */
        PageTable::init();
        PageTable::insert(phys, IRQ_STUBS.as_u64(), size, Protection::RX);

        /* log the IRQ stubs range */
        tracing::debug!(
            "IRQ Stubs are loaded into {:p}-{:p}",
            IRQ_STUBS,
            IRQ_STUBS + size,
        );
    }
}

impl Vm {
    #[inline]
    pub const fn irq_stubs() -> Uintptr {
        IRQ_STUBS
    }
}

impl Vm {
    #[inline]
    pub fn map(addr: Uintptr, base: u64, size: usize, prot: Protection) {
        hv_call!(hv_vm_map(addr.as_ptr(), base, size, prot.bits()));
    }

    #[inline]
    pub fn unmap(base: u64, size: usize) {
        hv_call!(hv_vm_unmap(base, size));
    }

    #[inline]
    pub fn alloc(size: usize) -> Uintptr {
        let mut ret = std::ptr::null_mut();
        hv_call!(hv_vm_allocate(&raw mut ret, size, HV_ALLOCATE_DEFAULT));
        Uintptr::from(ret)
    }

    #[inline]
    pub fn dealloc(addr: Uintptr, size: usize) {
        hv_call!(hv_vm_deallocate(addr.as_ptr(), size));
    }

    #[inline]
    pub fn protect(base: Uintptr, size: usize, prot: Protection) {
        hv_call!(hv_vm_protect(base.as_u64(), size, prot.bits()));
    }
}
