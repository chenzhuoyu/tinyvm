#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod bsd_mman;
pub mod commpage;
pub mod faults;
pub mod mach_msg;
pub mod mach_vm;
pub mod mem;
pub mod mmio;
pub mod pac;
pub mod shared_cache;
pub mod task;
pub mod tlb;

use std::io::Error as IoError;

use crate::{
    aarch64::paging::PAGE_SIZE,
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

/// The guest address of IRQ stubs.
pub const IRQ_STUBS: Uintptr = Uintptr::new(0xffffff0000);

/// The stack top address for IRQ stubs.
pub const STACK_TOP: Uintptr = Uintptr::new(0xfffffffff0);

unsafe extern "C" {
    unsafe static virtos_end: u8;
    unsafe static virtos_start: u8;
}

#[inline]
fn virtos_code() -> &'static [u8] {
    unsafe {
        let end = &raw const virtos_end;
        let start = &raw const virtos_start;
        std::slice::from_raw_parts(start, end.offset_from_unsigned(start))
    }
}

fn map_fixed(addr: Uintptr, size: usize) {
    let ret = unsafe {
        libc::mmap(
            addr.as_ptr(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANON | libc::MAP_PRIVATE | libc::MAP_FIXED,
            -1,
            0,
        )
    };
    assert!(
        ret != libc::MAP_FAILED,
        "cannot allocate memory at {addr:p}: {err}",
        err = IoError::last_os_error(),
    );
}

pub fn init() {
    let code = virtos_code();
    let size = align_to_page(code.len());
    let base = STACK_TOP.align_down(PAGE_SIZE);

    /* initialize the page table and memory manager */
    assert!(size <= PAGE_SIZE * 3, "virtos is too large");
    mem::VmMap::init();
    commpage::init();

    /* allocate memory for IRQ stubs and EL1 stack */
    map_fixed(IRQ_STUBS, size);
    map_fixed(base, PAGE_SIZE);

    /* load the IRQ stubs into memory */
    unsafe {
        std::ptr::write_bytes(IRQ_STUBS.as_ptr::<u8>(), 0, size);
        std::ptr::copy_nonoverlapping(code.as_ptr(), IRQ_STUBS.as_ptr(), code.len());
    }

    /* insert IRQ stubs into the VM map */
    mem::VmMap::map(
        mem::VmKind::Regular,
        IRQ_STUBS,
        size,
        Protection::RX,
        Protection::RX,
        true,
    );

    /* log the IRQ stubs range */
    tracing::debug!(
        "IRQ Stubs are loaded into {IRQ_STUBS:p}-{end:p}",
        end = IRQ_STUBS + code.len()
    );

    /* insert EL1 stack into the VM map */
    mem::VmMap::map(
        mem::VmKind::Regular,
        base,
        PAGE_SIZE,
        Protection::RW,
        Protection::RW,
        true,
    );

    /* log the IRQ stubs range */
    tracing::debug!(
        "EL1 stack is loaded into {base:p}-{end:p}, SP_EL1={STACK_TOP:p}",
        end = base + PAGE_SIZE
    );
}
