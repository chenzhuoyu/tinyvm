use mach2::{
    kern_return::{KERN_INVALID_ARGUMENT, KERN_SUCCESS, kern_return_t},
    port::MACH_PORT_NULL,
    vm::{mach_vm_allocate, mach_vm_deallocate, mach_vm_map, mach_vm_protect},
    vm_inherit::VM_INHERIT_DEFAULT,
    vm_prot::{VM_PROT_ALL, VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE},
};

use crate::{
    aarch64::{
        cpu::Cpu,
        errors::AsKernReturn,
        paging::PAGE_SIZE,
        syscall::mach::{
            ARG__kernelrpc_mach_vm_allocate_trap, ARG__kernelrpc_mach_vm_deallocate_trap,
            ARG__kernelrpc_mach_vm_map_trap, ARG__kernelrpc_mach_vm_protect_trap,
        },
        virtos::{
            mem::{VmKind, VmMap},
            task::TASK_SELF,
        },
    },
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

pub fn _kernelrpc_mach_vm_allocate_trap(
    _cpu: &Cpu,
    args: ARG__kernelrpc_mach_vm_allocate_trap,
) -> kern_return_t {
    let task = *TASK_SELF;
    let result = unsafe { mach_vm_allocate(args.target, args.addr, args.size, args.flags) };

    /* not targeting self, just forward the result */
    if args.target != task || result != KERN_SUCCESS {
        return result;
    }

    /* insert into guest address space and page table */
    let ret = unsafe {
        VmMap::map(
            VmKind::Regular,
            Uintptr::from(*args.addr),
            align_to_page(args.size as usize),
            Protection::RW,
            Protection::all(),
        )
    };

    /* deallocate memory if map failed */
    if let Err(err) = ret {
        unsafe {
            mach_vm_deallocate(args.target, *args.addr, args.size);
            err.as_kern_return()
        }
    } else {
        KERN_SUCCESS
    }
}

pub fn _kernelrpc_mach_vm_deallocate_trap(
    cpu: &Cpu,
    args: ARG__kernelrpc_mach_vm_deallocate_trap,
) -> kern_return_t {
    if args.target == *TASK_SELF {
        if let Err(err) = VmMap::unmap(args.address, args.size as usize) {
            return err.as_kern_return();
        }
        cpu.flush_tlb(args.address, (args.size as usize).div_ceil(PAGE_SIZE));
    }
    unsafe { mach_vm_deallocate(args.target, args.address.as_u64(), args.size) }
}

pub fn _kernelrpc_mach_vm_protect_trap(
    cpu: &Cpu,
    mut args: ARG__kernelrpc_mach_vm_protect_trap,
) -> kern_return_t {
    let size = align_to_page(args.size as usize);
    let task_self = *TASK_SELF;

    /* handle the memory protection for self-targeting maps */
    if args.target == task_self {
        let Some(prot) = Protection::from_bits(args.new_protection as u64) else {
            return KERN_INVALID_ARGUMENT;
        };
        if let Err(err) = VmMap::protect(cpu, args.address, size, prot, args.set_maximum != 0) {
            return err.as_kern_return();
        }
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
    _cpu: &Cpu,
    args: ARG__kernelrpc_mach_vm_map_trap,
) -> kern_return_t {
    macro_rules! set_prot {
        ($prot:ident, $flag:ident, $name:ident) => {
            if args.cur_protection & $flag != 0 {
                $prot |= Protection::$name;
            }
        };
    }

    /* check output address */
    if args.address.is_null() {
        return KERN_INVALID_ARGUMENT;
    }

    /* make a copy of the desired protection */
    let mut prot = Protection::NONE;
    let mut map_protection = args.cur_protection;

    /* never map as executable at host side when targeting self */
    if args.target == *TASK_SELF {
        map_protection &= !VM_PROT_EXECUTE;
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

    /* insert into page table */
    let ret = unsafe {
        VmMap::map(
            VmKind::Regular,
            Uintptr::from(*args.address),
            align_to_page(args.size as usize),
            prot,
            Protection::all(),
        )
    };

    /* unmap the memory on page table failure */
    if let Err(err) = ret {
        unsafe {
            mach_vm_deallocate(args.target, *args.address, args.size);
            err.as_kern_return()
        }
    } else {
        KERN_SUCCESS
    }
}
