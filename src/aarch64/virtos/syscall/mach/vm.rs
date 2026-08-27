use mach2::{
    kern_return::{KERN_INVALID_ARGUMENT, KERN_SUCCESS, kern_return_t},
    port::MACH_PORT_NULL,
    traps::mach_task_self,
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
        virtos::mem::VmMap,
    },
    mem::Protection,
};

pub fn _kernelrpc_mach_vm_allocate_trap(
    cpu: &Cpu,
    args: ARG__kernelrpc_mach_vm_allocate_trap,
) -> kern_return_t {
    if args.target == unsafe { mach_task_self() } {
        _kernelrpc_mach_vm_map_trap(
            cpu,
            ARG__kernelrpc_mach_vm_map_trap {
                target: args.target,
                address: args.addr,
                size: args.size,
                mask: 0,
                flags: args.flags,
                cur_protection: VM_PROT_READ | VM_PROT_WRITE,
            },
        )
    } else {
        unsafe { mach_vm_allocate(args.target, args.addr, args.size, args.flags) }
    }
}

pub fn _kernelrpc_mach_vm_deallocate_trap(
    cpu: &Cpu,
    args: ARG__kernelrpc_mach_vm_deallocate_trap,
) -> kern_return_t {
    if args.target != unsafe { mach_task_self() } {
        return unsafe { mach_vm_deallocate(args.target, args.address.addr(), args.size) };
    }
    VmMap::unmap(args.address, args.size as usize);
    cpu.flush_tlb(args.address, (args.size as usize).div_ceil(PAGE_SIZE));
    KERN_SUCCESS
}

pub fn _kernelrpc_mach_vm_protect_trap(
    cpu: &Cpu,
    args: ARG__kernelrpc_mach_vm_protect_trap,
) -> kern_return_t {
    if args.target != unsafe { mach_task_self() } {
        return unsafe {
            mach_vm_protect(
                args.target,
                args.address.addr(),
                args.size,
                args.set_maximum,
                args.new_protection,
            )
        };
    }
    let Some(prot) = Protection::from_bits(args.new_protection as u64) else {
        return KERN_INVALID_ARGUMENT;
    };
    AsKernReturn::as_kern_return(&VmMap::protect(
        cpu,
        args.address,
        args.size as usize,
        prot,
        args.set_maximum != 0,
    ))
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

    /* only handle self-targeting VM maps */
    if args.target != unsafe { mach_task_self() } {
        return unsafe {
            mach_vm_map(
                args.target,
                args.address,
                args.size,
                args.mask,
                args.flags,
                MACH_PORT_NULL,
                0,
                0,
                args.cur_protection,
                VM_PROT_ALL,
                VM_INHERIT_DEFAULT,
            )
        };
    }

    /* make a copy of the desired protection */
    let mut cur_prot = Protection::NONE;
    let mut max_prot = Protection::NONE;

    /* convert protection flags */
    set_prot!(cur_prot, VM_PROT_READ, READ);
    set_prot!(cur_prot, VM_PROT_WRITE, WRITE);
    set_prot!(cur_prot, VM_PROT_EXECUTE, EXEC);

    /* convert max protection flags */
    set_prot!(max_prot, VM_PROT_READ, READ);
    set_prot!(max_prot, VM_PROT_WRITE, WRITE);
    set_prot!(max_prot, VM_PROT_EXECUTE, EXEC);

    // /* get the allocated address */
    // let size = args.size as usize;
    // let addr = unsafe { Uintptr::from(*args.address) };
    // let flags = VmFlags::empty();

    // /* insert into page table */
    // VmMap::map(VmKind::Regular, addr, size, cur_prot, max_prot, flags);
    // KERN_SUCCESS
    todo!()
}
