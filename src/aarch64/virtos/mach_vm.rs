use std::io::ErrorKind;

use mach2::{
    kern_return::{
        KERN_FAILURE, KERN_INVALID_ADDRESS, KERN_MEMORY_FAILURE, KERN_NOT_SUPPORTED,
        KERN_PROTECTION_FAILURE, KERN_SUCCESS, kern_return_t,
    },
    port::{MACH_PORT_NULL, mach_port_name_t},
    vm::{mach_vm_allocate, mach_vm_deallocate, mach_vm_map, mach_vm_protect},
    vm_inherit::VM_INHERIT_DEFAULT,
    vm_prot::{VM_PROT_ALL, VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE, vm_prot_t},
    vm_types::{mach_vm_offset_t, mach_vm_size_t},
};

use super::{HalProvider, task::TASK_SELF, tlb::TlbProvider};
use crate::{
    aarch64::{
        paging::{PAGE_SIZE, PageTable},
        vm::Vm,
    },
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

pub fn _kernelrpc_mach_vm_allocate_trap(
    _hal: &impl HalProvider,
    target: mach_port_name_t,
    address: *mut mach_vm_offset_t,
    size: mach_vm_size_t,
    flags: i32,
) -> kern_return_t {
    if target != *TASK_SELF {
        unsafe { mach_vm_allocate(target, address, size, flags) }
    } else {
        // TODO (chenzhuoyu): implement this
        todo!()
    }
}

pub fn _kernelrpc_mach_vm_deallocate_trap(
    hal: &impl HalProvider,
    target: mach_port_name_t,
    address: Uintptr,
    size: mach_vm_size_t,
) -> kern_return_t {
    if target == *TASK_SELF {
        if let Err(err) = PageTable::unmap(address.as_u64(), size as usize) {
            return match err.error.kind() {
                ErrorKind::InvalidInput => KERN_INVALID_ADDRESS,
                ErrorKind::Unsupported => KERN_NOT_SUPPORTED,
                ErrorKind::OutOfMemory => KERN_MEMORY_FAILURE,
                _ => KERN_FAILURE,
            };
        }
        Vm::unmap(address.as_u64(), align_to_page(size as usize));
        hal.flush_tlb_range(address.as_u64(), (size as usize).div_ceil(PAGE_SIZE));
    }
    unsafe { mach_vm_deallocate(target, address.as_u64(), size) }
}

pub fn _kernelrpc_mach_vm_protect_trap(
    hal: &impl HalProvider,
    target: mach_port_name_t,
    address: Uintptr,
    size: mach_vm_size_t,
    set_maximum: i32,
    new_protection: vm_prot_t,
) -> kern_return_t {
    if target == *TASK_SELF {
        if let Some(prot) = Protection::from_bits(new_protection as u64) {
            if let Err(err) = PageTable::protect(address.as_u64(), size as usize, prot) {
                match err.error.kind() {
                    ErrorKind::InvalidInput => KERN_INVALID_ADDRESS,
                    ErrorKind::Unsupported => KERN_NOT_SUPPORTED,
                    ErrorKind::OutOfMemory => KERN_MEMORY_FAILURE,
                    _ => KERN_FAILURE,
                }
            } else {
                let size = size as usize;
                Vm::protect(address, align_to_page(size), prot);
                hal.flush_tlb_range(address.as_u64(), size.div_ceil(PAGE_SIZE));
                KERN_SUCCESS
            }
        } else {
            KERN_PROTECTION_FAILURE
        }
    } else {
        unsafe { mach_vm_protect(target, address.as_u64(), size, set_maximum, new_protection) }
    }
}

pub fn _kernelrpc_mach_vm_map_trap(
    hal: &impl HalProvider,
    target: mach_port_name_t,
    address: *mut mach_vm_offset_t,
    size: mach_vm_size_t,
    mask: mach_vm_offset_t,
    flags: i32,
    cur_protection: vm_prot_t,
) -> kern_return_t {
    macro_rules! set_prot {
        ($prot:ident, $flag:ident, $name:ident) => {
            if cur_protection & $flag != 0 {
                $prot |= Protection::$name;
            }
        };
    }

    /* make a copy of the desired protection */
    let mut prot = Protection::NONE;
    let mut map_protectiion = cur_protection;

    /* always map as read-write at host side when targeting self */
    if target == *TASK_SELF {
        map_protectiion = VM_PROT_READ | VM_PROT_WRITE;
    }

    /* forward the syscall */
    let result = unsafe {
        mach_vm_map(
            target,
            address,
            size,
            mask,
            flags,
            MACH_PORT_NULL,
            0,
            0,
            map_protectiion,
            VM_PROT_ALL,
            VM_INHERIT_DEFAULT,
        )
    };

    /* handle the syscall only if it's self-targeting and successful */
    if target != *TASK_SELF || result != KERN_SUCCESS {
        return result;
    }

    /* convert protection flags */
    set_prot!(prot, VM_PROT_READ, READ);
    set_prot!(prot, VM_PROT_WRITE, WRITE);
    set_prot!(prot, VM_PROT_EXECUTE, EXEC);

    /* get the map address & size */
    let size = align_to_page(size as usize);
    let addr = unsafe { Uintptr::from(*address) };

    /* map to guest address space & insert into page table, then flush TLB */
    Vm::map(addr, addr.as_u64(), size, prot);
    PageTable::insert(addr, addr.as_u64(), size, prot);
    hal.flush_tlb_range(addr.as_u64(), size / PAGE_SIZE);
    KERN_SUCCESS
}
