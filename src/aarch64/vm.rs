use std::{fmt::Debug, sync::LazyLock};

use super::ffi::*;
use crate::{Addressable, Memory, Protection, hv_call, ptr::Uintptr};

#[derive(Debug)]
pub struct Vm {
    ipa_bits: u32,
    max_vcpu_count: usize,
}

pub static VM: LazyLock<Vm> = LazyLock::new(|| {
    let cfg = unsafe { hv_vm_config_create() };
    let mut ipa_bits = 0u32;
    let mut max_vcpu_count = 0u32;
    hv_call!(hv_vm_get_max_vcpu_count(&raw mut max_vcpu_count));
    hv_call!(hv_vm_config_get_default_ipa_size(&raw mut ipa_bits));
    hv_call!(hv_vm_config_set_el2_enabled(cfg, false));
    hv_call!(hv_vm_create(cfg));
    Vm::new(ipa_bits, max_vcpu_count)
});

impl Vm {
    #[inline]
    fn new(ipa_bits: u32, max_vcpu_count: u32) -> Self {
        Self {
            ipa_bits,
            max_vcpu_count: max_vcpu_count as usize,
        }
    }
}

impl Vm {
    #[inline]
    pub const fn ipa_size(&self) -> usize {
        1usize << self.ipa_bits
    }

    #[inline]
    pub const fn max_vcpu_count(&self) -> usize {
        self.max_vcpu_count
    }
}

impl Vm {
    #[inline]
    pub fn map(&self, mem: &Memory, prot: Protection) {
        let size = mem.size();
        let addr = mem.addr();
        hv_call!(hv_vm_map(addr.as_ptr(), addr.as_u64(), size, prot.bits()));
    }

    #[inline]
    pub fn protect(&self, base: Uintptr, size: usize, prot: Protection) {
        hv_call!(hv_vm_protect(base.as_u64(), size, prot.bits()))
    }
}

impl Drop for Vm {
    #[inline]
    fn drop(&mut self) {
        unsafe { hv_vm_destroy() };
    }
}
