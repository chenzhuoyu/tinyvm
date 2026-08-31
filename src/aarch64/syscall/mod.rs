pub mod bsd;
mod bsd_impl;
pub mod mach;
mod mach_impl;
pub mod machdep;
pub mod sys;

use std::fmt::{Display, Formatter, Result as FmtResult};

use bsd::BsdSyscall;
use mach::MachTrap;
use machdep::MachDep;

use crate::{
    aarch64::{
        cpu::Cpu,
        regs::{PSTATE_C, PSTATE_N, PSTATE_NZCV, PSTATE_V, PSTATE_Z, Reg, SysReg},
    },
    utils::ptr::VMA,
};

trait BsdResult: Copy {
    fn as_result(self) -> u64;
}

macro_rules! impl_bsd_result {
    ($($({$($bounds:tt)*})? $ty:ty),+ $(,)?) => {
        $(
            impl<$($($bounds)*)?> BsdResult for $ty {
                #[inline(always)]
                fn as_result(self) -> u64 {
                    self as u64
                }
            }
        )+
    };
}

impl_bsd_result! {
    u8,
    u16,
    u32,
    u64,
    usize,
    i8,
    i16,
    i32,
    i64,
    isize,
    {T} *mut T,
    {T} *const T,
}

impl BsdResult for VMA {
    #[inline(always)]
    fn as_result(self) -> u64 {
        self.addr()
    }
}

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
}

impl<'p> Syscall<'p> {
    pub fn read(cpu: &'p Cpu) -> Self {
        let mut ret = Self {
            args: [
                cpu.read_reg(Reg::X0),
                cpu.read_reg(Reg::X1),
                cpu.read_reg(Reg::X2),
                cpu.read_reg(Reg::X3),
                cpu.read_reg(Reg::X4),
                cpu.read_reg(Reg::X5),
                cpu.read_reg(Reg::X6),
                cpu.read_reg(Reg::X7),
                cpu.read_reg(Reg::X8),
            ],
            spsr: cpu.read_sys_reg(SysReg::SPSR_EL1) & !PSTATE_NZCV,
            num: cpu.read_reg(Reg::X16) as i64,
            cpu,
        };
        if ret.num == 0 {
            let extra = cpu.read_reg(Reg::X9);
            ret.num = ret.args.shift_left([extra])[0] as i64;
        }
        ret
    }
}

impl Syscall<'_> {
    #[inline]
    fn forward(&mut self) {
        let mut nzcv = 0u64;
        sys::syscall_inplace(self.num, &mut self.args, &mut nzcv);
        self.spsr |= nzcv;
    }

    #[inline]
    fn bsd_return<T: BsdResult>(&mut self, ret: T, ok: bool) {
        self.args[0] = ret.as_result();
        self.spsr &= !PSTATE_C;
        self.spsr |= (ok as u64).wrapping_sub(1) & PSTATE_C;
    }
}

impl Syscall<'_> {
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
            nzcv = Nzcv(self.spsr)
        );
    }
}

impl Syscall<'_> {
    #[inline]
    pub fn finalize(self) {
        self.cpu.write_reg(Reg::X0, self.args[0]);
        self.cpu.write_sys_reg(SysReg::SPSR_EL1, self.spsr);
    }
}
