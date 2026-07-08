pub mod cpu;
pub mod disasm;
pub mod ffi;
pub mod paging;
pub mod regs;
pub mod syscall;
pub mod virtos;
pub mod vm;

use cpu::Cpu;
use paging::PageTable;
use vm::Vm;

use crate::{
    Unit,
    image::Image,
    mem::{Addressable, Memory, Protection},
    utils::ptr::Uintptr,
};

const MAX_DYLD_ARGS: usize = 128;
const DYLD_ARGS_SIZE: usize = 4096;
const INIT_STACK_SIZE: usize = 1048576;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct KernelArgs {
    main: Uintptr,
    argc: usize,
    args: [Uintptr; MAX_DYLD_ARGS],
    sbuf: [u8; DYLD_ARGS_SIZE],
    slen: usize,
}

impl KernelArgs {
    fn add_cstr(&mut self, str: &str) -> Uintptr {
        assert!(
            self.slen + str.len() < DYLD_ARGS_SIZE,
            "insufficient space in string buffer"
        );
        let pos = self.slen;
        let end = self.slen + str.len();
        self.sbuf[self.slen..end].copy_from_slice(str.as_bytes());
        self.sbuf[end] = 0;
        self.slen = end + 1;
        Uintptr::from(self.sbuf.as_ptr()) + pos
    }
}

const KARGS_SIZE: usize = std::mem::size_of::<KernelArgs>();
const ZEROS_SIZE: usize = INIT_STACK_SIZE - KARGS_SIZE;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct InitStackFrame {
    zero: [u8; ZEROS_SIZE],
    args: KernelArgs,
}

#[inline]
fn on_load(addr: Uintptr, size: usize, prot: Protection) {
    PageTable::insert(addr, addr.as_u64(), size, prot);
}

pub fn vm_exec() -> Unit {
    Vm::init();
    Image::set_load_handler(on_load);

    /* create the stack */
    let stack = Memory::alloc(INIT_STACK_SIZE).map(Protection::RW);
    let frame = stack.addr().as_mut::<InitStackFrame>();

    /* add the stack into page table */
    PageTable::insert(
        stack.addr(),
        stack.addr().as_u64(),
        stack.size(),
        Protection::RW,
    );

    /* log the stack range */
    tracing::debug!(
        "Stack is mapped at {:p}-{:p}",
        stack.addr(),
        stack.addr() + stack.size()
    );

    /* load dyld and the target image */
    let dyld = Image::dyld().entry.as_u64();
    let image = Image::load("/bin/ls")?; // TODO: load actual image

    /* construct the initial stack frame */
    frame.args.main = image.data.addr();
    frame.args.argc = 1;
    frame.args.args[0] = frame.args.add_cstr("ls");
    frame.args.args[1] = Uintptr::NIL;
    frame.args.args[2] = Uintptr::NIL;
    frame.args.args[3] = frame.args.add_cstr("executable_path=/bin/ls");
    frame.args.args[4] = Uintptr::NIL;

    /* start the vCPU */
    Cpu::new(dyld, &raw const frame.args as u64).run();
    Ok(())
}
