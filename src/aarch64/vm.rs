use std::{collections::BTreeMap, fmt::Debug, sync::LazyLock};

use parking_lot::RwLock;

use super::{cpu::Cpu, ffi::*};
use crate::{Addressable, Memory, Protection, hv_call, ptr::Uintptr};

#[derive(Debug)]
pub struct Vm {
    mmap: RwLock<BTreeMap<Uintptr, (Uintptr, usize)>>,
}

pub static VM: LazyLock<Vm> = LazyLock::new(|| {
    let cfg = unsafe { hv_vm_config_create() };
    hv_call!(hv_vm_config_set_el2_enabled(cfg, false));
    hv_call!(hv_vm_create(cfg));
    Vm::new()
});

impl Vm {
    fn new() -> Self {
        let mmap = RwLock::new(BTreeMap::new());
        Self { mmap }
    }
}

impl Vm {
    #[inline]
    pub fn map(&self, base: Uintptr, mem: &Memory, prot: Protection) {
        let size = mem.size();
        let addr = mem.addr();
        hv_call!(hv_vm_map(addr.as_ptr(), base.as_u64(), size, prot.bits()));
        self.mmap.write().insert(base, (addr, size));
    }

    #[inline]
    pub fn protect(&self, base: Uintptr, size: usize, prot: Protection) {
        hv_call!(hv_vm_protect(base.as_u64(), size, prot.bits()))
    }

    #[inline]
    pub fn translate(&self, addr: Uintptr) -> Option<Uintptr> {
        let (&base, &(host, size)) = self.mmap.read().range(..=addr).next_back()?;
        (addr < base + size).then_some(host + (addr - base))
    }
}

impl Vm {
    #[inline]
    pub fn new_vcpu(&self, pc: Uintptr, sp: Uintptr) -> Cpu {
        Cpu::new(pc, sp)
    }
}

impl Drop for Vm {
    #[inline]
    fn drop(&mut self) {
        unsafe { hv_vm_destroy() };
    }
}
