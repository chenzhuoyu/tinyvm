use crate::{aarch64::ffi::*, hv_call, mem::Protection, utils::ptr::Uintptr};

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
    pub fn map(addr: Uintptr, size: usize, prot: Protection) {
        hv_call!(hv_vm_map(addr.as_ptr(), addr.as_u64(), size, prot.bits()));
    }

    #[inline]
    pub fn unmap(addr: Uintptr, size: usize) {
        hv_call!(hv_vm_unmap(addr.as_u64(), size));
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
    pub fn protect(addr: Uintptr, size: usize, prot: Protection) {
        hv_call!(hv_vm_protect(addr.as_u64(), size, prot.bits()));
    }
}
