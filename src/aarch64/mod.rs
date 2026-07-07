pub mod cpu;
pub mod ffi;
pub mod paging;
pub mod regs;
pub mod syscall;
pub mod vm;

use cpu::{COMMPAGE_BEGIN, COMMPAGE_END, COMMPAGE_RO_BEGIN, COMMPAGE_RO_END, Cpu};
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

pub fn vm_exec() -> Unit {
    Vm::init();
    Image::set_load_handler(PageTable::register);

    /* create the stack */
    let stack = Memory::alloc(INIT_STACK_SIZE).map(Protection::RW);
    let frame = stack.addr().as_mut::<InitStackFrame>();

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
    frame.args.main = image.entry;
    frame.args.argc = 1;
    frame.args.args[0] = frame.args.add_cstr("ls");
    frame.args.args[1] = Uintptr::NIL;
    frame.args.args[2] = Uintptr::NIL;
    frame.args.args[3] = frame.args.add_cstr("executable_path=/bin/ls");
    frame.args.args[4] = Uintptr::NIL;

    /* mark the Commpage as read-only in page table */
    PageTable::register(
        COMMPAGE_BEGIN,
        COMMPAGE_END - COMMPAGE_BEGIN,
        Protection::READ,
    );

    /* there seems to be two Commpages, don't ask me why, I genuinely don't know */
    PageTable::register(
        COMMPAGE_RO_BEGIN,
        COMMPAGE_RO_END - COMMPAGE_RO_BEGIN,
        Protection::READ,
    );

    /* mark the stack as read-write and start the vCPU */
    PageTable::register(stack.addr(), stack.size(), Protection::RW);
    Cpu::new(dyld, &raw const frame.args as u64).run();
    Ok(())
}
