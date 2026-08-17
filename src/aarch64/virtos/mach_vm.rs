use std::io::Error as IoError;

use mach2::{
    kern_return::{
        KERN_FAILURE, KERN_INVALID_ADDRESS, KERN_INVALID_ARGUMENT, KERN_MEMORY_ERROR,
        KERN_PROTECTION_FAILURE, KERN_SUCCESS, kern_return_t,
    },
    port::MACH_PORT_NULL,
    vm::{mach_vm_allocate, mach_vm_deallocate, mach_vm_map, mach_vm_protect},
    vm_inherit::VM_INHERIT_DEFAULT,
    vm_prot::{VM_PROT_ALL, VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE},
    vm_statistics::VM_FLAGS_ANYWHERE,
};

use super::{HalProvider, mmio, task::TASK_SELF, tlb::TlbProvider};
use crate::{
    aarch64::{
        paging::{PAGE_SIZE, PageTable},
        syscall::mach::{
            ARG__kernelrpc_mach_vm_allocate_trap, ARG__kernelrpc_mach_vm_deallocate_trap,
            ARG__kernelrpc_mach_vm_map_trap, ARG__kernelrpc_mach_vm_protect_trap,
        },
        vm::Vm,
    },
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

trait AsKernReturn {
    fn as_kern_return(&self) -> kern_return_t;
}

impl AsKernReturn for IoError {
    #[inline]
    fn as_kern_return(&self) -> kern_return_t {
        if let Some(errno) = self.raw_os_error() {
            match errno {
                libc::EACCES => KERN_PROTECTION_FAILURE,
                libc::ENOMEM => KERN_INVALID_ADDRESS,
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
    args: ARG__kernelrpc_mach_vm_allocate_trap,
) -> kern_return_t {
    let task = *TASK_SELF;
    let result = unsafe { mach_vm_allocate(args.target, args.addr, args.size, args.flags) };

    /* not targeting self, just forward the result */
    if args.target != task {
        return result;
    }

    /* get the mapped address */
    let size = align_to_page(args.size as usize);
    let addr = unsafe { Uintptr::from(*args.addr) };

    /* insert into guest address space and page table */
    PageTable::map(addr, addr, size, Protection::RW, Protection::all());
    hal.flush_tlb(addr.as_u64(), size / PAGE_SIZE);
    Vm::map(addr, size, Protection::RW);
    KERN_SUCCESS
}

pub fn _kernelrpc_mach_vm_deallocate_trap(
    hal: &impl HalProvider,
    args: ARG__kernelrpc_mach_vm_deallocate_trap,
) -> kern_return_t {
    if args.target == *TASK_SELF {
        if let Err(err) = PageTable::unmap(args.address, args.size as usize) {
            return err.error.as_kern_return();
        }
        let size = align_to_page(args.size as usize);
        Vm::unmap(args.address, size);
        mmio::unmap(args.address, size);
        hal.flush_tlb(args.address.as_u64(), size / PAGE_SIZE);
    }
    unsafe { mach_vm_deallocate(args.target, args.address.as_u64(), args.size) }
}

pub fn _kernelrpc_mach_vm_protect_trap(
    hal: &impl HalProvider,
    mut args: ARG__kernelrpc_mach_vm_protect_trap,
) -> kern_return_t {
    let size = align_to_page(args.size as usize);
    let task_self = *TASK_SELF;

    /* handle the memory protection for self-targeting maps */
    if args.target == task_self {
        let Some(prot) = Protection::from_bits(args.new_protection as u64) else {
            return KERN_INVALID_ARGUMENT;
        };
        if let Err(err) = PageTable::protect(args.address, size, prot, args.set_maximum != 0) {
            return err.error.as_kern_return();
        }
        if let Err(err) = mmio::protect(args.address, size, prot) {
            return err.as_kern_return();
        }
        hal.flush_tlb(args.address.as_u64(), size / PAGE_SIZE);
        args.new_protection &= !VM_PROT_EXECUTE;
        args.set_maximum = 0;
    }

    /* forward the mach trap */
    unsafe {
        mach_vm_protect(
            args.target,
            args.address.as_u64(),
            size as u64,
            args.set_maximum,
            args.new_protection,
        )
    }
}

pub fn _kernelrpc_mach_vm_map_trap(
    hal: &impl HalProvider,
    args: ARG__kernelrpc_mach_vm_map_trap,
) -> kern_return_t {
    macro_rules! set_prot {
        ($prot:ident, $flag:ident, $name:ident) => {
            if args.cur_protection & $flag != 0 {
                $prot |= Protection::$name;
            }
        };
    }

    /* make a copy of the desired protection */
    let mut prot = Protection::NONE;
    let mut map_protection = args.cur_protection;

    /* never map as executable at host side when targeting self */
    if args.target == *TASK_SELF {
        map_protection &= !VM_PROT_EXECUTE;
    }

    /* check for fixed address mappings */
    if args.flags & VM_FLAGS_ANYWHERE == 0 {
        let addr = unsafe { Uintptr::from(*args.address) };
        Vm::unmap(addr, args.size as usize);
        PageTable::unmap(addr, args.size as usize).expect("cannot unmap fixed range");
    }

    /* forward the syscall */
    let result = unsafe {
        mach_vm_map(
            args.target,
            args.address,
            args.size,
            args.mask,
            args.flags,
            MACH_PORT_NULL,
            0,
            0,
            map_protection,
            VM_PROT_ALL,
            VM_INHERIT_DEFAULT,
        )
    };

    /* handle the syscall only if it's self-targeting and successful */
    if args.target != *TASK_SELF || result != KERN_SUCCESS {
        return result;
    }

    /* convert protection flags */
    set_prot!(prot, VM_PROT_READ, READ);
    set_prot!(prot, VM_PROT_WRITE, WRITE);
    set_prot!(prot, VM_PROT_EXECUTE, EXEC);

    /* get the map address & size */
    let size = align_to_page(args.size as usize);
    let addr = unsafe { Uintptr::from(*args.address) };

    /* insert into page table, map to guest address space, then flush TLB */
    PageTable::map(addr, addr, size, prot, Protection::all());
    hal.flush_tlb(addr.as_u64(), size / PAGE_SIZE);
    Vm::map(addr, size, prot);
    KERN_SUCCESS
}
