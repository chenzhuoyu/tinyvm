pub mod cpu;
pub mod ffi;
pub mod paging;
pub mod regs;
pub mod vm;

use cpu::{COMMPAGE_BEGIN, COMMPAGE_END, Cpu};
use vm::VM;

use crate::{
    Unit,
    image::Image,
    mem::{Addressable, Memory, Protection},
    utils::{ptr::Uintptr, str::Sz},
};

const MAX_KERNEL_ARGS: usize = 128;
const INIT_STACK_SIZE: usize = 1048576;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct KernelArgs {
    main: Uintptr,
    argc: usize,
    args: [Sz; MAX_KERNEL_ARGS],
}

const KARGS_SIZE: usize = std::mem::size_of::<KernelArgs>();
const ZEROS_SIZE: usize = INIT_STACK_SIZE - KARGS_SIZE;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct InitStackFrame {
    zero: [u8; ZEROS_SIZE],
    args: KernelArgs,
}

pub fn vm_exec() -> Unit {
    let stack = Memory::alloc(INIT_STACK_SIZE).map(Protection::RW);
    let frame = stack.addr().as_mut::<InitStackFrame>();

    /* mark the Commpage as read-only in page table */
    VM.register_pages(
        COMMPAGE_BEGIN,
        COMMPAGE_END - COMMPAGE_BEGIN,
        Protection::READ,
    );

    /* register the stack into page table, and bind the load handler */
    VM.register_pages(stack.addr(), stack.size(), Protection::RW);
    Image::set_load_handler(|addr, size, prot| VM.register_pages(addr, size, prot));

    /* load dyld and the target image */
    let dyld = Image::dyld().entry.as_u64();
    let image = Image::load("/bin/ls")?; // TODO: load actual image

    /* construct the initial stack frame */
    frame.args.main = image.entry;
    frame.args.argc = 1;
    frame.args.args[0] = Sz::from(c"ls");
    frame.args.args[1] = Sz::NIL;
    frame.args.args[2] = Sz::NIL;
    frame.args.args[3] = Sz::from(c"executable_path=/bin/ls");
    frame.args.args[4] = Sz::NIL;

    /* initialize the vCPU and set it to EL0, and start the vCPU */
    Cpu::new(dyld, &raw const frame.args as u64).run();
    Ok(())
}
