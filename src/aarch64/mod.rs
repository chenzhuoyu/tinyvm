pub mod consts;
pub mod cpu;
pub mod ffi;
pub mod vm;

use cpu::{Reg, SysReg};
use vm::VM;

use crate::{
    Unit,
    aarch64::cpu::Cpu,
    image::DYLD,
    mem::{Addressable, Memory, Protection},
};

pub fn vm_exec() -> Unit {
    let cpu = Cpu::new();
    let stack = Memory::alloc(0x10000, Protection::RW)?;
    let stack_top = stack.addr() + (stack.size() - 16);

    /* initialize the vCPU and set it to EL0 */
    cpu.write_reg(Reg::PC, dbg!(DYLD.entry).as_u64());
    cpu.write_reg(Reg::CPSR, 0);
    cpu.write_sys_reg(SysReg::SP_EL0, stack_top.as_u64());
    cpu.write_sys_reg(SysReg::VBAR_EL1, dbg!(VM.irq_stubs()).as_u64());
    cpu.run();
    Ok(())
}
