use super::ffi::*;
use crate::{
    aarch64::paging::PageTable,
    hv_call,
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

unsafe extern "C" {
    unsafe static irq_stub_end: u8;
    unsafe static irq_stub_start: u8;
}

#[inline]
fn irq_stubs() -> &'static [u8] {
    unsafe {
        let end = &raw const irq_stub_end;
        let data = &raw const irq_stub_start;
        std::slice::from_raw_parts(data, end.offset_from_unsigned(data))
    }
}

pub enum Vm {}
static mut IRQ_STUBS: Uintptr = Uintptr::NIL;

impl Vm {
    pub fn init() {
        let mut code = std::ptr::null_mut();
        let mut ptable = std::ptr::null_mut();

        /* find the IRQ stubs */
        let stub = irq_stubs();
        let size = align_to_page(stub.len());
        let hcfg = unsafe { hv_vm_config_create() };

        /* create VM without EL2 */
        hv_call!(hv_vm_config_set_el2_enabled(hcfg, false));
        hv_call!(hv_vm_create(hcfg));

        /* allocate L1 page table */
        hv_call!(hv_vm_allocate(
            &raw mut ptable,
            libc::vm_page_size,
            HV_ALLOCATE_DEFAULT
        ));

        /* map the L1 page table into guest space */
        hv_call!(hv_vm_map(
            ptable,
            ptable as u64,
            size,
            Protection::RW.bits()
        ));

        /* and allocate memory for IRQ stubs */
        hv_call!(hv_vm_allocate(&raw mut code, size, HV_ALLOCATE_DEFAULT));
        hv_call!(hv_vm_map(code, code as u64, size, Protection::RX.bits()));

        /* load IRQ stubs into memory */
        unsafe {
            std::ptr::copy_nonoverlapping(stub.as_ptr(), code as *mut u8, stub.len());
            IRQ_STUBS = code.into();
        }

        /* initialize the page table */
        PageTable::init(Uintptr::from(ptable));
        PageTable::register(code.into(), size, Protection::RX);
    }
}

impl Vm {
    #[inline]
    pub fn irq_stubs() -> Uintptr {
        unsafe { IRQ_STUBS }
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
