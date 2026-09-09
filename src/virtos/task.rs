use mach2::{port::mach_port_name_t, traps::mach_task_self};

use crate::cpu::Cpu;

#[inline]
pub fn task_self_trap(_cpu: &Cpu) -> mach_port_name_t {
    unsafe { mach_task_self() }
}
