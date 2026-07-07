mod bsd;
mod mach;
mod machdep;

use bsd::BsdSyscall;
use mach::MachTrap;
use machdep::MachDep;

use super::{
    cpu::Cpu,
    regs::{PSTATE_NZCV, Reg, SysReg},
};
use crate::{
    aarch64::{regs::PSTATE_C, virtos},
    utils::ptr::Uintptr,
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
    fn bsd_return(&mut self, err: i32, ret0: u64, ret1: u64) {
        if err != 0 {
            self.args[0] = err as u64;
            self.args[1] = 0;
            self.nzcv |= PSTATE_C;
        } else {
            self.args[0] = ret0;
            self.args[1] = ret1;
            self.nzcv &= !PSTATE_C;
        }
    }
}

impl Syscall<'_> {
    fn dispatch_bsd(&mut self, bsd: BsdSyscall) {
        match bsd {
            BsdSyscall::shared_region_check_np(args) => {
                let ptr = Uintptr::from(args.start_address);
                let err = virtos::shared_region_check_np(ptr);
                self.bsd_return(err, 0, 0);
            }
            BsdSyscall::shared_region_map_and_slide_2_np(args) => {
                todo!("shared_region_map_and_slide_2_np({args:?})");
            }
            BsdSyscall::Unknown(..) => self.bsd_return(libc::ENOSYS, 0, 0),
            _ => self.forward(),
        }
    }

    fn dispatch_mach(&mut self, mach: MachTrap) {
        match mach {
            MachTrap::Unknown(..) => self.args[0] = libc::KERN_INVALID_ARGUMENT as u64,
            _ => self.forward(),
        }
    }

    fn dispatch_machdep(&mut self, machdep: MachDep) {
        match machdep {
            MachDep::SetCthreadSelf(tsd) => self.cpu.write_sys_reg(SysReg::TPIDRRO_EL0, tsd),
            MachDep::GetCthreadSelf => self.args[0] = self.cpu.read_sys_reg(SysReg::TPIDRRO_EL0),
            MachDep::Unknown(..) => {}
        }
    }
}

impl Syscall<'_> {
    pub fn dispatch(&mut self) {
        if MachDep::is_machdep_trap(self.num) {
            let id = self.args[3];
            let machdep = MachDep::decode(id, &self.args);
            tracing::trace!("MACH_DEP  :: [{id:3?}] {machdep:?}");
            self.dispatch_machdep(machdep);
        } else if self.num < 0 {
            let id = -self.num as u64;
            let mach = MachTrap::decode(id, &self.args);
            tracing::trace!("MACH_TRAP :: [{id:3?}] {mach:?}");
            self.dispatch_mach(mach);
        } else {
            let id = self.num as u64;
            let bsd = BsdSyscall::decode(id, &self.args);
            tracing::trace!("SYSCALL   :: [{id:3?}] {bsd:?}");
            self.dispatch_bsd(bsd);
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
