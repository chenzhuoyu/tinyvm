pub mod bsd;
pub mod mach;
pub mod machdep;

use std::fmt::{Display, Formatter, Result as FmtResult};

use bsd::BsdSyscall;
use mach::MachTrap;
use machdep::MachDep;

use super::{
    cpu::Cpu,
    regs::{PSTATE_C, PSTATE_N, PSTATE_NZCV, PSTATE_V, PSTATE_Z, Reg, SysReg},
    virtos::{self, HalProvider},
};
use crate::utils::ptr::Uintptr;

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

impl BsdResult for () {
    #[inline(always)]
    fn as_result(self) -> u64 {
        0u64
    }
}

impl BsdResult for Uintptr {
    #[inline(always)]
    fn as_result(self) -> u64 {
        self.as_u64()
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
            spsr: spsr & !PSTATE_NZCV,
        }
    }
}

impl Syscall<'_> {
    #[inline]
    fn syscall(num: i64, args: &mut [u64; 9], nzcv: &mut u64) {
        unsafe {
            std::arch::asm!(
                "svc #0x80",
                "mrs {}, NZCV",
                out(reg) *nzcv,
                in("x16") num,
                inout("x0") args[0],
                inout("x1") args[1],
                in("x2") args[2],
                in("x3") args[3],
                in("x4") args[4],
                in("x5") args[5],
                in("x6") args[6],
                in("x7") args[7],
                in("x8") args[8],
            );
        }
    }
}

impl Syscall<'_> {
    #[inline]
    fn forward(&mut self) {
        let mut nzcv = 0u64;
        Self::syscall(self.num, &mut self.args, &mut nzcv);
        self.spsr |= nzcv;
    }

    #[inline]
    fn bsd_error(&mut self, err: i32) {
        self.args[0] = err as u64;
        self.args[1] = 0;
        self.spsr |= PSTATE_C;
    }

    #[inline]
    fn bsd_result<T: BsdResult>(&mut self, ret: T) {
        self.args[0] = ret.as_result();
        self.args[1] = 0;
        self.spsr &= !PSTATE_C;
    }
}

impl Syscall<'_> {
    fn dispatch_bsd(&mut self, bsd: BsdSyscall) {
        mod impls {
            pub(super) use super::virtos::{bsd_mman::*, shared_cache::*};
        }
        macro_rules! handle_syscall {
            ($($name:ident $(($($field:ident),* $(,)?))?),+ $(,)?) => {
                match bsd {
                    $(
                        handle_syscall!(@HANDLER $name args @($($($field),*)?)) => {
                            match impls::$name(self.cpu, $($(args.$field),*)?) {
                                Ok(ret) => self.bsd_result(ret),
                                Err(err) => self.bsd_error(err.raw_os_error().unwrap_or(-1)),
                            }
                        }
                    )*
                    BsdSyscall::Unknown(..) => self.bsd_error(libc::ENOSYS),
                    _ => self.forward(),
                }
            };
            (@HANDLER $name:ident $args:ident @($($_:ident),+)) => { BsdSyscall::$name($args) };
            (@HANDLER $name:ident $_:ident @()) => { BsdSyscall::$name };
        }
        handle_syscall! {
            msync(addr, len, flags),
            munmap(addr, len),
            mprotect(addr, len, prot),
            mmap(addr, len, prot, flags, fd, pos),
            shared_region_check_np(start_address),
            msync_nocancel(addr, len, flags),
            shared_region_map_and_slide_2_np(files_count, files, mappings_count, mappings_u),
            map_with_linking_np(regions, region_count, link_info, link_info_size)
        }
    }

    fn dispatch_mach(&mut self, mach: MachTrap) {
        mod impls {
            pub(super) use super::virtos::{mach_msg::*, mach_vm::*, task::*};
        }
        macro_rules! handle_mach_trap {
            ($($name:ident ($($args:ident)?)),+ $(,)?) => {
                match mach {
                    $(
                        handle_mach_trap!(@HANDLER $name [$($args)?]) => {
                            self.args[0] = impls::$name(self.cpu, $($args)?) as u64;
                        }
                    )*
                    MachTrap::Unknown(..) => self.args[0] = libc::KERN_INVALID_ARGUMENT as u64,
                    _ => self.forward(),
                }
            };
            (@HANDLER $name:ident [$args:ident]) => { MachTrap::$name($args) };
            (@HANDLER $name:ident []) => { MachTrap::$name };
        }
        handle_mach_trap! {
            _kernelrpc_mach_vm_allocate_trap(args),
            _kernelrpc_mach_vm_deallocate_trap(args),
            _kernelrpc_mach_vm_protect_trap(args),
            _kernelrpc_mach_vm_map_trap(args),
            task_self_trap(),
            mach_msg_trap(args),
            mach_msg_overwrite_trap(args),
            mach_msg2_trap(args),
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
            nzcv = Nzcv(self.spsr)
        );
    }
}

impl Syscall<'_> {
    pub fn finalize(self) {
        self.cpu.write_reg(Reg::X0, self.args[0]);
        self.cpu.write_reg(Reg::X1, self.args[1]);
        self.cpu.write_sys_reg(SysReg::SPSR_EL1, self.spsr);
    }
}
