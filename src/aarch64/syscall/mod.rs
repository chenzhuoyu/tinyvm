mod bsd;
mod mach;
mod machdep;

use std::fmt::{Display, Formatter, Result as FmtResult};

use bsd::BsdSyscall;
use mach::MachTrap;
use machdep::MachDep;

use super::{
    cpu::Cpu,
    regs::{PSTATE_NZCV, Reg, SysReg},
};
use crate::{
    aarch64::{
        regs::{PSTATE_C, PSTATE_N, PSTATE_V, PSTATE_Z},
        virtos,
    },
    utils::ptr::Uintptr,
};

#[repr(transparent)]
struct Nzcv(u64);

impl Display for Nzcv {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", if self.0 & PSTATE_N != 0 { "N" } else { "-" })?;
        write!(f, "{}", if self.0 & PSTATE_Z != 0 { "Z" } else { "-" })?;
        write!(f, "{}", if self.0 & PSTATE_C != 0 { "C" } else { "-" })?;
        write!(f, "{}", if self.0 & PSTATE_V != 0 { "V" } else { "-" })?;
        Ok(())
    }
}

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
    #[inline]
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
            BsdSyscall::munmap(args) => {
                dbg!(args);
                todo!()
            }
            BsdSyscall::mprotect(args) => {
                dbg!(args);
                todo!()
            }
            BsdSyscall::mmap(args) => {
                dbg!(args);
                std::intrinsics::breakpoint();
                todo!()
            }
            BsdSyscall::shared_region_check_np(args) => {
                let ptr = Uintptr::from(args.start_address);
                let err = virtos::shared_region_check_np(ptr);
                self.bsd_return(err, 0, 0);
            }
            BsdSyscall::shared_region_map_and_slide_2_np(args) => {
                todo!("shared_region_map_and_slide_2_np({args:?})");
            }
            BsdSyscall::map_with_linking_np(args) => {
                todo!("map_with_linking_np({args:?})");
            }
            BsdSyscall::Unknown(..) => self.bsd_return(libc::ENOSYS, 0, 0),
            _ => self.forward(),
        }
    }

    fn dispatch_mach(&mut self, mach: MachTrap) {
        macro_rules! handle_mach_trap {
            ($($name:ident $(($($field:ident),* $(,)?))?),+ $(,)?) => {
                match mach {
                    $(
                        handle_mach_trap!(@HANDLER $name args @($($($field),*)?)) => {
                            self.args[0] = virtos::$name(self, $($(args.$field),*)?) as u64;
                        }
                    )*
                    MachTrap::Unknown(..) => self.args[0] = libc::KERN_INVALID_ARGUMENT as u64,
                    _ => self.forward(),
                }
            };
            (@HANDLER $name:ident $args:ident @($($_:ident),+)) => { MachTrap::$name($args) };
            (@HANDLER $name:ident $_:ident @()) => { MachTrap::$name };
        }
        handle_mach_trap! {
            _kernelrpc_mach_vm_allocate_trap(target, addr, size, flags),
            _kernelrpc_mach_vm_deallocate_trap(target, address, size),
            _kernelrpc_mach_vm_protect_trap(target, address, size, set_maximum, new_protection),
            _kernelrpc_mach_vm_map_trap(target, address, size, mask, flags, cur_protection),
            task_self_trap,
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
            tracing::trace!("MACH_DEP  [{id:3?}] :: {machdep:?}");
            self.dispatch_machdep(machdep);
        } else if self.num < 0 {
            let id = -self.num as u64;
            let mach = MachTrap::decode(id, &self.args);
            tracing::trace!("MACH_TRAP [{id:3?}] :: {mach:?}");
            self.dispatch_mach(mach);
        } else {
            let id = self.num as u64;
            let bsd = BsdSyscall::decode(id, &self.args);
            tracing::trace!("SYSCALL   [{id:3?}] :: {bsd:?}");
            self.dispatch_bsd(bsd);
        }
        tracing::trace!(
            "{pads:15} => 0x{retv:x} (nzcv={nzcv})",
            pads = ' ',
            retv = self.args[0],
            nzcv = Nzcv(self.nzcv)
        );
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

impl virtos::HalProvider for Syscall<'_> {
    fn flush_tlb_range(&mut self, start: u64, num_pages: usize) {
        let cpsr = self.cpu.read_reg(Reg::CPSR);
        self.cpu.write_reg(Reg::X16, start);
        self.cpu.write_reg(Reg::X17, num_pages as u64);
        self.cpu.write_reg(Reg::CPSR, cpsr | PSTATE_V);
    }
}
