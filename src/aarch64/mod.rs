pub mod consts;
pub mod cpu;
pub mod ffi;
pub mod vm;

use cpu::{Reg, SysReg};
use vm::VM;

use crate::{Addressable, Memory, Protection, Unit, aarch64::cpu::Cpu};

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

pub fn vm_main() -> Unit {
    let cpu = Cpu::new();
    let irq = Memory::copy_from_slice(irq_stubs())?;
    let code = Memory::copy_from_slice(bootloader())?;
    let stack = Memory::mmap(0x10000)?;

    /* map memory regions */
    VM.map(&irq, Protection::RX);
    VM.map(&code, Protection::RX);
    VM.map(&stack, Protection::RW);

    /* calculate initial PC & SP */
    let pc = code.addr();
    let sp = stack.addr() + (stack.size() - 16);

    /* initialize the vCPU and set it to EL0 */
    cpu.write_reg(Reg::PC, pc.as_u64());
    cpu.write_reg(Reg::CPSR, 0);
    cpu.write_sys_reg(SysReg::SP_EL0, sp.as_u64());
    cpu.write_sys_reg(SysReg::VBAR_EL1, irq.addr().as_u64());
    cpu.run();
    Ok(())
}
