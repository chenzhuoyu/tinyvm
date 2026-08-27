pub mod cpu;
pub mod disasm;
pub mod errors;
pub mod ffi;
pub mod paging;
pub mod regs;
pub mod syscall;
pub mod virtos;
pub mod vm;

use std::io::Error as IoError;

use cpu::Cpu;
use vm::Vm;

use crate::{
    Unit,
    aarch64::virtos::mem::{VmKind, VmMap},
    image::Image,
    mem::Protection,
    utils::ptr::{Uintptr, VMA},
};

const MAX_DYLD_ARGS: usize = 128;
const MAIN_STACK_SIZE: usize = 1048576;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct KernelArgs {
    exec: Uintptr,
    argc: usize,
    args: [Uintptr; MAX_DYLD_ARGS],
}

const KARGS_SIZE: usize = std::mem::size_of::<KernelArgs>();
const SZBUF_SIZE: usize = paging::PAGE_SIZE - KARGS_SIZE - 8;
const ZEROS_SIZE: usize = MAIN_STACK_SIZE - paging::PAGE_SIZE;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct StringBuf {
    buf: [u8; SZBUF_SIZE],
    len: usize,
}

impl StringBuf {
    fn add_cstr(&mut self, str: &str) -> Uintptr {
        assert!(
            self.len + str.len() < self.buf.len(),
            "insufficient space in string buffer"
        );
        let pos = self.len;
        let end = self.len + str.len();
        self.buf[self.len..end].copy_from_slice(str.as_bytes());
        self.buf[end] = 0;
        self.len = end + 1;
        Uintptr::from(self.buf.as_ptr()) + pos
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct InitStackFrame {
    zero: [u8; ZEROS_SIZE],
    args: KernelArgs,
    sbuf: StringBuf,
}

struct FrameBuilder<'f> {
    pos: usize,
    frame: &'f mut InitStackFrame,
}

impl<'f> FrameBuilder<'f> {
    #[inline]
    fn new(frame: &'f mut InitStackFrame, image: &Image) -> Self {
        frame.args.args.fill(Uintptr::NIL);
        frame.args.exec = image.data.addr();
        frame.args.argc = 0;
        Self { pos: 0, frame }
    }
}

impl FrameBuilder<'_> {
    #[inline]
    fn end(mut self) -> Self {
        self.frame.args.args[self.pos] = Uintptr::NIL;
        self.pos += 1;
        self
    }

    #[inline]
    fn add<S: AsRef<str>>(mut self, value: S) -> Self {
        if self.frame.args.argc == self.pos {
            self.frame.args.argc += 1;
        }
        self.frame.args.args[self.pos] = self.frame.sbuf.add_cstr(value.as_ref());
        self.pos += 1;
        self
    }
}

#[inline]
fn on_load(addr: Uintptr, size: usize, prot: Protection, max_prot: Protection) {
    VmMap::insert(
        VmKind::Regular,
        addr,
        VMA::new(addr.as_u64()),
        size,
        prot,
        max_prot,
        false,
    );
    if unsafe { libc::mprotect(addr.as_ptr(), size, libc::PROT_READ) } != 0 {
        panic!(
            "cannot make image segment read-only: {err}",
            err = IoError::last_os_error()
        );
    }
}

pub fn vm_exec() -> Unit {
    Vm::init();
    Image::set_load_handler(on_load);
    virtos::init();

    /* create the stack */
    let stack = Vm::alloc(MAIN_STACK_SIZE);
    let frame = stack.as_mut::<InitStackFrame>();

    /* insert the stack range into page table */
    VmMap::insert(
        VmKind::Regular,
        stack,
        VMA::new(stack.as_u64()),
        MAIN_STACK_SIZE,
        Protection::RW,
        Protection::RW,
        false,
    );

    /* load dyld and the target image */
    let dyld = Image::dyld().entry.as_u64();
    let image = Image::load("/bin/ls")?; // TODO: load actual image

    /*
    executable_path=/Users/chenzhuoyu/Sources/tests/test_dyld/syscall
    pfz=0xffff10000
    stack_guard=0xc50487d7ab460077
    malloc_entropy=0xd174f580181b9420,0xcf8cfc6f3a616c46
    ptr_munge=0x5398233f3075739f
    main_stack=0x16fe00000,0x7fc000,0x16be00000,0x4000000
    executable_file=0x1a01000010,0x175c3c7
    dyld_file=0x1a01000010,0xfffffff000a955c
    executable_cdhash=18dbaa193b6eeabb81814fadae14d049f7d13137
    executable_boothash=19944d88913ce237b3082b3ee642edd162999fe8
    th_port=0x103
    security_config=0x0
    */

    /* construct the initial stack frame */
    FrameBuilder::new(frame, &image)
        .add("ls")
        .end()
        .end()
        .add("executable_path=/bin/ls")
        .add("malloc_entropy=0x1,0x2df")
        .add("ptr_munge=0x1")
        .end();

    /* start the vCPU */
    tracing::debug!("CPU entry point is set to 0x{dyld:x}");
    Cpu::new(dyld, &raw const frame.args as u64).run();
    Ok(())
}
