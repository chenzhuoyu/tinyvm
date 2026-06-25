use std::{fmt::Debug, sync::LazyLock};

use super::ffi::*;
use crate::{
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

#[derive(Debug)]
pub struct Vm {
    ipa_bits: u32,
    irq_stubs: Uintptr,
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
        let stub = irq_stubs();
        let size = align_to_page(stub.len());
        let mut code = std::ptr::null_mut();

        /* allocate, copy & map IRQ stubs */
        unsafe {
            hv_call!(hv_vm_allocate(&raw mut code, size, HV_ALLOCATE_DEFAULT));
            std::ptr::copy_nonoverlapping(stub.as_ptr(), code as *mut u8, stub.len());
            hv_call!(hv_vm_map(code, code as u64, size, Protection::RX.bits()));
        }

        /* construct the VM */
        Self {
            ipa_bits,
            irq_stubs: Uintptr::from(code),
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
    pub const fn irq_stubs(&self) -> Uintptr {
        self.irq_stubs
    }

    #[inline]
    pub const fn max_vcpu_count(&self) -> usize {
        self.max_vcpu_count
    }
}

impl Vm {
    #[inline]
    pub fn map(&self, addr: Uintptr, base: u64, size: usize, prot: Protection) {
        hv_call!(hv_vm_map(addr.as_ptr(), base, size, prot.bits()));
    }

    #[inline]
    pub fn unmap(&self, base: u64, size: usize) {
        hv_call!(hv_vm_unmap(base, size));
    }

    #[inline]
    pub fn alloc(&self, size: usize) -> Uintptr {
        let mut ret = std::ptr::null_mut();
        hv_call!(hv_vm_allocate(&raw mut ret, size, HV_ALLOCATE_DEFAULT));
        Uintptr::from(ret)
    }

    #[inline]
    pub fn dealloc(&self, addr: Uintptr, size: usize) {
        hv_call!(hv_vm_deallocate(addr.as_ptr(), size));
    }

    #[inline]
    pub fn protect(&self, base: Uintptr, size: usize, prot: Protection) {
        hv_call!(hv_vm_protect(base.as_u64(), size, prot.bits()));
    }
}

impl Drop for Vm {
    #[inline]
    fn drop(&mut self) {
        unsafe { hv_vm_destroy() };
    }
}
