mod bsd;
mod platform;

use bsd::BsdSyscall;
use platform::*;

use super::{
    cpu::Cpu,
    regs::{PSTATE_NZCV, Reg, SysReg},
};

pub struct SvcResult {
    pub x0: u64,
    pub x1: u64,
    pub spsr: u64,
}

#[derive(Debug)]
pub struct Syscall<'p> {
    num: i64,
    cpu: &'p Cpu,
    args: [u64; 9],
    spsr: u64,
    nzcv: u64,
}

impl<'p> Syscall<'p> {
    pub fn read(cpu: &'p Cpu) -> Self {
        let num = cpu.read_reg(Reg::X16);
        let spsr = cpu.read_sys_reg(SysReg::SPSR_EL1);

        /* read syscall args */
        let mut args = [
            cpu.read_reg(Reg::X0),
            cpu.read_reg(Reg::X1),
            cpu.read_reg(Reg::X2),
            cpu.read_reg(Reg::X3),
            cpu.read_reg(Reg::X4),
            cpu.read_reg(Reg::X5),
            cpu.read_reg(Reg::X6),
            cpu.read_reg(Reg::X7),
            cpu.read_reg(Reg::X8),
        ];

        /* indirect syscall, use X0 as the syscall number */
        let num = {
            if num == 0 {
                let [first] = args.shift_left([0]);
                first as i64
            } else {
                num as i64
            }
        };

        /* construct the syscall */
        Self {
            num,
            cpu,
            args,
            spsr,
            nzcv: 0,
        }
    }
}

impl Syscall<'_> {
    fn forward(&mut self) {
        unsafe {
            std::arch::asm!(
                "svc #0x80",
                "mrs {}, NZCV",
                out(reg) self.nzcv,
                in("x16") self.num,
                inout("x0") self.args[0],
                inout("x1") self.args[1],
                in("x2") self.args[2],
                in("x3") self.args[3],
                in("x4") self.args[4],
                in("x5") self.args[5],
                in("x6") self.args[6],
                in("x7") self.args[7],
                in("x8") self.args[8],
            );
        }
    }
}

impl Syscall<'_> {
    fn dispatch_bsd(&mut self, id: u64) {
        let bsd = BsdSyscall::decode(id, &self.args);
        tracing::trace!("SYSCALL   :: [{id:3?}] {bsd:?}");
        self.forward();
    }

    fn dispatch_mach(&mut self, id: u64) {
        tracing::trace!("MACH_TRAP :: [{:3?}]", id);
        self.forward();
    }

    fn dispatch_machdep(&mut self, id: u64) {
        match id {
            MACHDEP_SET_CTHREAD_SELF => {
                let value = self.args[0];
                tracing::trace!("MACH_DEP  :: [  2] set_cthread_self(self=0x{value:x})");
                self.cpu.write_sys_reg(SysReg::TPIDRRO_EL0, value);
            }
            MACHDEP_GET_CTHREAD_SELF => {
                tracing::trace!("MACH_DEP  :: [  3] get_cthread_self()");
                self.args[0] = self.cpu.read_sys_reg(SysReg::TPIDRRO_EL0)
            }
            _ => {}
        }
    }
}

impl Syscall<'_> {
    pub fn dispatch(&mut self) {
        if self.num == SYS_MACHDEP {
            self.dispatch_machdep(self.args[3]);
        } else if self.num < 0 {
            self.dispatch_mach(-self.num as u64);
        } else {
            self.dispatch_bsd(self.num as u64);
        }
    }
}

impl Syscall<'_> {
    pub fn finalize(self) {
        let spsr = (self.spsr & !PSTATE_NZCV) | self.nzcv;
        self.cpu.write_sys_reg(SysReg::SPSR_EL1, spsr);
        self.cpu.write_reg(Reg::X0, self.args[0]);
        self.cpu.write_reg(Reg::X1, self.args[1]);
    }
}
