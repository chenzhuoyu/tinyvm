pub mod cpu;
pub mod ffi;
pub mod regs;
pub mod vm;

use cpu::Cpu;

use crate::{
    Unit,
    image::{DYLD, Image},
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

    /* construct the initial stack frame */
    let image = Image::load("/bin/ls")?;
    frame.args.main = image.entry;
    frame.args.argc = 1;
    frame.args.args[0] = Sz::from(c"ls");
    frame.args.args[1] = Sz::NIL;
    frame.args.args[2] = Sz::NIL;
    frame.args.args[3] = Sz::from(c"executable_path=/bin/ls");
    frame.args.args[4] = Sz::NIL;

    /* initialize the vCPU and set it to EL0, and start the vCPU */
    Cpu::new(DYLD.entry.as_u64(), &raw const frame.args as u64).run();
    Ok(())
}
