pub mod consts;
pub mod cpu;
pub mod ffi;
pub mod vm;

use bytes::BufMut;
use vm::VM;

use crate::{Memory, Protection, Unit, aarch64::cpu::SysReg, ptr::Uintptr};

unsafe extern "C" {
    unsafe static bl_end: u8;
    unsafe static bl_start: u8;
    unsafe static irq_stub_end: u8;
    unsafe static irq_stub_start: u8;
}

#[inline]
fn irq_stubs() -> &'static [u8] {
    unsafe {
        let end = &raw const irq_stub_end;
        let data = &raw const irq_stub_start;
        std::slice::from_raw_parts(data, end.offset_from_unsigned(data))
    }
}

#[inline]
fn bootloader() -> &'static [u8] {
    unsafe {
        let end = &raw const bl_end;
        let data = &raw const bl_start;
        std::slice::from_raw_parts(data, end.offset_from_unsigned(data))
    }
}

const PAGE_SIZE: usize = 0x4000;
const STACK_SIZE: usize = 0x10000;

const BL_BASE: Uintptr = Uintptr::new(0x10000);
const STUBS_BASE: Uintptr = Uintptr::new(0x4000);
const STACK_BASE: Uintptr = Uintptr::new(0x20000);

pub fn vm_main() -> Unit {
    let cpu = VM.new_vcpu(BL_BASE, STACK_BASE + STACK_SIZE - 16);
    let size = bootloader().len().div_ceil(PAGE_SIZE) * PAGE_SIZE;

    /* map memory for bootloader & stack */
    let stack = Memory::mmap(STACK_SIZE)?;
    let mut code = Memory::mmap(size)?;
    let mut stubs = Memory::mmap(PAGE_SIZE)?;

    /* inject the bootloader & SVC handler */
    code.view_mut(0).put_slice(bootloader());
    stubs.view_mut(0).put_slice(irq_stubs());

    /* set the CPU to execute the bootloader, and start the virtual CPU */
    VM.map(BL_BASE, &code, Protection::RX);
    VM.map(STUBS_BASE, &stubs, Protection::RX);
    VM.map(STACK_BASE, &stack, Protection::RW);

    /* set the interrupt table at address 0 */
    cpu.write_sys_reg(SysReg::VBAR_EL1, 0x4000);
    cpu.run();
    Ok(())
}
