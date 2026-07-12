use std::{cell::Cell, io::ErrorKind};

use mach2::{
    kern_return::{
        KERN_FAILURE, KERN_INVALID_ADDRESS, KERN_MEMORY_FAILURE, KERN_NOT_SUPPORTED,
        KERN_PROTECTION_FAILURE, KERN_SUCCESS, kern_return_t,
    },
    port::{MACH_PORT_NULL, mach_port_name_t},
    traps::mach_task_self,
    vm::{mach_vm_allocate, mach_vm_deallocate, mach_vm_map, mach_vm_protect},
    vm_inherit::VM_INHERIT_DEFAULT,
    vm_prot::{VM_PROT_ALL, vm_prot_t},
    vm_types::{mach_vm_offset_t, mach_vm_size_t},
};

use super::Syscall;
use crate::{
    aarch64::{
        paging::{PAGE_SIZE, PageTable},
        regs::{PSTATE_V, Reg},
        vm::Vm,
    },
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

thread_local! {
    static TASK_SELF: Cell<u32> = const { Cell::new(0) };
}

impl Syscall<'_> {
    fn virtos_flush_tlb(&mut self, start: u64, count: u64) {
        let cpsr = self.cpu.read_reg(Reg::CPSR);
        self.cpu.write_reg(Reg::X16, start);
        self.cpu.write_reg(Reg::X17, count);
        self.cpu.write_reg(Reg::CPSR, cpsr | PSTATE_V);
    }
}

impl Syscall<'_> {
    pub(super) fn _kernelrpc_mach_vm_allocate_trap(
        &mut self,
        target: mach_port_name_t,
        address: *mut mach_vm_offset_t,
        size: mach_vm_size_t,
        flags: i32,
    ) -> kern_return_t {
        if target != TASK_SELF.get() {
            unsafe { mach_vm_allocate(target, address, size, flags) }
        } else {
            todo!()
        }
    }

    pub(super) fn _kernelrpc_mach_vm_deallocate_trap(
        &mut self,
        target: mach_port_name_t,
        address: Uintptr,
        size: mach_vm_size_t,
    ) -> kern_return_t {
        if target != TASK_SELF.get() {
            unsafe { mach_vm_deallocate(target, address.as_u64(), size) }
        } else {
            todo!()
        }
    }

    pub(super) fn _kernelrpc_mach_vm_protect_trap(
        &mut self,
        target: mach_port_name_t,
        address: Uintptr,
        size: mach_vm_size_t,
        set_maximum: i32,
        new_protection: vm_prot_t,
    ) -> kern_return_t {
        if target == TASK_SELF.get() {
            if let Some(prot) = Protection::from_bits(new_protection as u64) {
                if let Err(err) = PageTable::protect(address.as_u64(), size as usize, prot) {
                    match err.error.kind() {
                        ErrorKind::InvalidInput => KERN_INVALID_ADDRESS,
                        ErrorKind::Unsupported => KERN_NOT_SUPPORTED,
                        ErrorKind::OutOfMemory => KERN_MEMORY_FAILURE,
                        _ => KERN_FAILURE,
                    }
                } else {
                    Vm::protect(address, align_to_page(size as usize), prot);
                    self.virtos_flush_tlb(address.as_u64(), size.div_ceil(PAGE_SIZE as u64));
                    KERN_SUCCESS
                }
            } else {
                KERN_PROTECTION_FAILURE
            }
        } else {
            unsafe { mach_vm_protect(target, address.as_u64(), size, set_maximum, new_protection) }
        }
    }

    pub(super) fn _kernelrpc_mach_vm_map_trap(
        &mut self,
        target: mach_port_name_t,
        address: *mut mach_vm_offset_t,
        size: mach_vm_size_t,
        mask: mach_vm_offset_t,
        flags: i32,
        cur_protection: vm_prot_t,
    ) -> kern_return_t {
        if target != TASK_SELF.get() {
            unsafe {
                mach_vm_map(
                    target,
                    address,
                    size,
                    mask,
                    flags,
                    MACH_PORT_NULL,
                    0,
                    0,
                    cur_protection,
                    VM_PROT_ALL,
                    VM_INHERIT_DEFAULT,
                )
            }
        } else {
            todo!()
        }
    }

    pub(super) fn task_self_trap(&mut self) -> mach_port_name_t {
        let port = unsafe { mach_task_self() };
        TASK_SELF.set(port);
        port
    }
}
