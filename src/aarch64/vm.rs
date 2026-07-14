use super::ffi::*;
use crate::{hv_call, mem::Protection, utils::ptr::Uintptr};

pub enum Vm {}

impl Vm {
    pub fn init() {
        let vcfg = unsafe { hv_vm_config_create() };
        hv_call!(hv_vm_config_set_el2_enabled(vcfg, false));
        hv_call!(hv_vm_create(vcfg));
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
