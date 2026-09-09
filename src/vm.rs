use crate::{
    ffi::*,
    hv_call,
    mem::Protection,
    paging::PAGE_SIZE,
    utils::ptr::{IPA, Uintptr},
};

pub enum Vm {}

impl Vm {
    pub fn init() {
        let vcfg = unsafe { hv_vm_config_create() };
        let mut max_ipa_bits = 0u32;
        hv_call!(hv_vm_config_set_el2_enabled(vcfg, false));
        hv_call!(hv_vm_config_set_ipa_granule(vcfg, HV_IPA_GRANULE_16KB));
        hv_call!(hv_vm_config_get_max_ipa_size(&raw mut max_ipa_bits));
        hv_call!(hv_vm_config_set_ipa_size(vcfg, max_ipa_bits));
        hv_call!(hv_vm_create(vcfg));
    }
}

impl Vm {
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
}

impl Vm {
    #[inline]
    pub fn map(addr: Uintptr, base: IPA, size: usize, prot: Protection) {
        debug_assert!(base.is_aligned_to(PAGE_SIZE) && size.is_multiple_of(PAGE_SIZE));
        hv_call!(hv_vm_map(addr.as_ptr(), base.addr(), size, prot.u64()));
    }

    #[inline]
    pub fn unmap(base: IPA, size: usize) {
        debug_assert!(base.is_aligned_to(PAGE_SIZE) && size.is_multiple_of(PAGE_SIZE));
        hv_call!(hv_vm_unmap(base.addr(), size));
    }

    #[inline]
    pub fn protect(base: IPA, size: usize, prot: Protection) {
        debug_assert!(base.is_aligned_to(PAGE_SIZE) && size.is_multiple_of(PAGE_SIZE));
        hv_call!(hv_vm_protect(base.addr(), size, prot.u64()));
    }
}
