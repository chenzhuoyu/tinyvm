use mach2::{
    kern_return::{
        KERN_FAILURE, KERN_INVALID_ADDRESS, KERN_INVALID_ARGUMENT, KERN_MEMORY_ERROR,
        KERN_NO_SPACE, KERN_PROTECTION_FAILURE, KERN_SUCCESS, kern_return_t,
    },
    port::{MACH_PORT_NULL, mach_port_name_t},
    vm::{mach_vm_allocate, mach_vm_deallocate, mach_vm_map, mach_vm_protect},
    vm_inherit::VM_INHERIT_DEFAULT,
    vm_prot::{VM_PROT_ALL, VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE, vm_prot_t},
    vm_types::{mach_vm_offset_t, mach_vm_size_t},
};

use super::{HalProvider, mmio, task::TASK_SELF, tlb::TlbProvider};
use crate::{
    aarch64::{
        paging::{PAGE_SIZE, PageFault, PageTable},
        vm::Vm,
    },
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

trait AsKernReturn {
    fn as_kern_return(&self) -> kern_return_t;
}

impl AsKernReturn for PageFault {
    #[inline]
    fn as_kern_return(&self) -> kern_return_t {
        if let Some(errno) = self.error.raw_os_error() {
            match errno {
                libc::EACCES => KERN_PROTECTION_FAILURE,
                libc::ENOMEM => KERN_INVALID_ADDRESS,
                libc::EEXIST => KERN_NO_SPACE,
                libc::EINVAL => KERN_INVALID_ARGUMENT,
                _ => KERN_MEMORY_ERROR,
            }
        } else {
            KERN_FAILURE
        }
    }
}

pub fn _kernelrpc_mach_vm_allocate_trap(
    hal: &impl HalProvider,
    target: mach_port_name_t,
    address: *mut mach_vm_offset_t,
    size: mach_vm_size_t,
    flags: i32,
) -> kern_return_t {
    let task = *TASK_SELF;
    let result = unsafe { mach_vm_allocate(target, address, size, flags) };

    /* not targeting self, just forward the result */
    if target != task {
        return result;
    }

    /* get the mapped address */
    let size = align_to_page(size as usize);
    let addr = unsafe { Uintptr::from(*address) };

    /* insert into guest address space and page table */
    PageTable::insert(addr, size, Protection::RW, Protection::all());
    hal.flush_tlb(addr.as_u64(), size / PAGE_SIZE);
    Vm::map(addr, size, Protection::RW);
    KERN_SUCCESS
}

pub fn _kernelrpc_mach_vm_deallocate_trap(
    hal: &impl HalProvider,
    target: mach_port_name_t,
    address: Uintptr,
    size: mach_vm_size_t,
) -> kern_return_t {
    if target == *TASK_SELF {
        if let Err(err) = PageTable::unmap(address, size as usize) {
            return err.as_kern_return();
        }
        let size = align_to_page(size as usize);
        Vm::unmap(address.as_u64(), size);
        hal.flush_tlb(address.as_u64(), size / PAGE_SIZE);
        mmio::unregister(address, size);
    }
    unsafe { mach_vm_deallocate(target, address.as_u64(), size) }
}

pub fn _kernelrpc_mach_vm_protect_trap(
    hal: &impl HalProvider,
    target: mach_port_name_t,
    address: Uintptr,
    size: mach_vm_size_t,
    set_maximum: i32,
    mut new_protection: vm_prot_t,
) -> kern_return_t {
    if target == *TASK_SELF {
        let Some(prot) = Protection::from_bits(new_protection as u64) else {
            return KERN_INVALID_ARGUMENT;
        };
        if let Err(err) = PageTable::protect(address, size as usize, prot, set_maximum != 0) {
            return err.as_kern_return();
        }
        let size = align_to_page(size as usize);
        Vm::protect(address, size, prot);
        hal.flush_tlb(address.as_u64(), size / PAGE_SIZE);
        new_protection &= !VM_PROT_EXECUTE;
    }
    unsafe { mach_vm_protect(target, address.as_u64(), size, set_maximum, new_protection) }
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
    let mut map_protection = cur_protection;

    /* never map as executable at host side when targeting self */
    if target == *TASK_SELF {
        map_protection &= !VM_PROT_EXECUTE;
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
            map_protection,
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
    PageTable::insert(addr, size, prot, Protection::all());
    hal.flush_tlb(addr.as_u64(), size / PAGE_SIZE);
    Vm::map(addr, size, prot);
    KERN_SUCCESS
}
