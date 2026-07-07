use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    ptr::NonNull,
};

use super::{
    ffi::*,
    paging::{MAIR_EL1_INIT, PageTable, SCTLR_EL1_INIT, TCR_EL1_INIT},
    regs::*,
    syscall::Syscall,
    vm::Vm,
};
use crate::{
    hv_call,
    macros::define_accessors,
    utils::{disasm::disasm, ptr::Uintptr},
};

pub(super) const COMMPAGE_END: Uintptr = Uintptr::new(0x1000000000);
pub(super) const COMMPAGE_BEGIN: Uintptr = Uintptr::new(0xfffffc000);

pub(super) const COMMPAGE_RO_END: Uintptr = Uintptr::new(0xfffff8000);
pub(super) const COMMPAGE_RO_BEGIN: Uintptr = Uintptr::new(0xfffff4000);

#[derive(Clone, Copy)]
struct VmException {
    syndrome: Syndrome,
    virt_addr: Uintptr,
    phys_addr: Uintptr,
}

impl Debug for VmException {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("VmException")
            .field_with("syndrome", |f| {
                if f.alternate() {
                    write!(
                        f,
                        "{:?} :: {:#?}",
                        Exception::from(self.syndrome.EC()),
                        self.syndrome
                    )
                } else {
                    Debug::fmt(&self.syndrome, f)
                }
            })
            .field("virt_addr", &self.virt_addr)
            .field("phys_addr", &self.phys_addr)
            .finish()
    }
}

impl From<hv_vcpu_exit_exception_t> for VmException {
    fn from(exc: hv_vcpu_exit_exception_t) -> Self {
        Self {
            syndrome: Syndrome(exc.syndrome),
            virt_addr: exc.virtual_address.into(),
            phys_addr: exc.physical_address.into(),
        }
    }
}

pub struct Cpu {
    vcpu: hv_vcpu_t,
    exit: NonNull<hv_vcpu_exit_t>,
}

impl Cpu {
    pub fn new(pc: u64, sp: u64) -> Self {
        let mut vcpu = 0u64;
        let mut exit = std::ptr::null_mut();

        /* create the vCPU */
        hv_call!(hv_vcpu_create(
            &raw mut vcpu,
            &raw mut exit,
            hv_vcpu_config_create()
        ));

        /* trap all debug access to vCPU from guest */
        hv_call!(hv_vcpu_set_trap_debug_exceptions(vcpu, true));
        hv_call!(hv_vcpu_set_trap_debug_reg_accesses(vcpu, true));

        /* construct the CPU state */
        let cpu = Self {
            vcpu,
            exit: NonNull::new(exit).expect("null VM exit buffer"),
        };

        /* setup paging */
        cpu.write_sys_reg(SysReg::TCR_EL1, TCR_EL1_INIT);
        cpu.write_sys_reg(SysReg::MAIR_EL1, MAIR_EL1_INIT);
        cpu.write_sys_reg(SysReg::TTBR0_EL1, PageTable::base().as_u64());
        cpu.write_sys_reg(SysReg::SCTLR_EL1, SCTLR_EL1_INIT);

        /* initialize the vCPU and set it to EL0 */
        cpu.write_reg(Reg::PC, pc);
        cpu.write_reg(Reg::CPSR, 0);
        cpu.write_sys_reg(SysReg::SP_EL0, sp);
        cpu.write_sys_reg(SysReg::VBAR_EL1, Vm::irq_stubs().as_u64());
        cpu.write_sys_reg(SysReg::CPACR_EL1, CPACR_FPEN);
        // cpu.write_sys_reg(SysReg::MDSCR_EL1, MDSCR_SS);
        cpu
    }
}

define_accessors! {
    reg     : u64 = (reg: Reg)        :: hv_vcpu_get_reg     -> hv_vcpu_set_reg,
    sys_reg : u64 = (sys_reg: SysReg) :: hv_vcpu_get_sys_reg -> hv_vcpu_set_sys_reg,
}

impl Cpu {
    fn handle_exc(&mut self, exc: VmException) {
        match Exception::from(exc.syndrome.EC()) {
            Exception::AA64_HVC => {
                let esr = self.read_sys_reg(SysReg::ESR_EL1);
                let elr = self.read_sys_reg(SysReg::ELR_EL1);
                self.handle_user_exc(Syndrome(esr), elr.into());
            }
            Exception::DATA_ABORT => {
                let pc = self.read_reg(Reg::PC);
                let iss = DataAbortISS(exc.syndrome.ISS() as u32);
                self.handle_data_abort(pc.into(), iss, exc.phys_addr);
            }
            Exception::SOFTWARE_STEP => {
                let pc = Uintptr::from(self.read_reg(Reg::PC));
                eprintln!("SINGLE_STEP: {}", disasm(pc));
                self.write_reg(Reg::CPSR, self.read_reg(Reg::CPSR) | PSTATE_SS);
            }
            ec => {
                panic!(
                    "unhandled exception {ec:?}:\nInstruction:\n  {insn:p}\nException: \
                     {exc:#?}\n{self:#?}",
                    insn = Uintptr::from(self.read_reg(Reg::PC))
                );
            }
        }
    }

    fn handle_user_exc(&mut self, esr: Syndrome, elr: Uintptr) {
        match Exception::from(esr.EC()) {
            Exception::AA64_SVC => {
                let mut syscall = Syscall::read(self);
                syscall.dispatch();
                syscall.finalize();
            }
            Exception::SYS_REG_TRAP => {
                let iss = SysRegTrapISS(esr.ISS() as u32);
                self.handle_sysreg_trap(iss);
                self.write_sys_reg(SysReg::ELR_EL1, elr.as_u64() + 4);
            }
            ec => {
                panic!(
                    "unhandled EL0 exception {ec:?}:\nInstruction:\n  {insn}\n{self:#?}",
                    insn = disasm(elr)
                );
            }
        }
    }

    fn handle_data_abort(&mut self, pc: Uintptr, iss: DataAbortISS, addr: Uintptr) {
        if addr < COMMPAGE_END && addr >= COMMPAGE_BEGIN
            || addr < COMMPAGE_RO_END && addr >= COMMPAGE_RO_BEGIN
        {
            if iss.wnr() != 0 {
                unimplemented!("write to commpage: {}", disasm(pc));
            }
            if iss.isv() != 1 {
                unimplemented!("DATA_ABORT with !ISS.ISV: {:#?} {}", iss, disasm(pc));
            }
            if iss.ar() != 0 {
                unimplemented!("acquire/release read of commpage: {}", disasm(pc));
            }
            let mut data = match (iss.sas(), iss.sse()) {
                (0b00, 0b1) => addr.read::<i8>() as u64,
                (0b00, 0b0) => addr.read::<u8>() as u64,
                (0b01, 0b1) => addr.read::<i16>() as u64,
                (0b01, 0b0) => addr.read::<u16>() as u64,
                (0b10, 0b1) => addr.read::<i32>() as u64,
                (0b10, 0b0) => addr.read::<u32>() as u64,
                (0b11, 0b1) => addr.read::<i64>() as u64,
                (0b11, 0b0) => addr.read::<u64>(),
                _ => unreachable!(),
            };
            if iss.sf() == 0 {
                data &= u32::MAX as u64;
            }
            self.write_reg(Reg::from(iss.srt()), data);
            self.write_reg(Reg::PC, pc.as_u64() + 4);
        } else {
            dbg!(&self);
            eprintln!("instr: {}", disasm(self.read_reg(Reg::PC)));
            todo!()
        }
    }

    fn handle_sysreg_trap(&mut self, iss: SysRegTrapISS) {
        macro_rules! wrong_sysreg {
            ($kind:literal) => {
                panic!(
                    "unhandled SysReg {ty} to \
                     \"s{op0}_{op1}_c{crn}_c{crm}_{op2}\"\nInstruction:\n  \
                     {insn}\nISS={iss:#?}\n{self:#?}",
                    ty = $kind,
                    op0 = iss.op0(),
                    op1 = iss.op1(),
                    crn = iss.crn(),
                    crm = iss.crm(),
                    op2 = iss.op2(),
                    insn = disasm(self.read_sys_reg(SysReg::ELR_EL1))
                )
            };
        }
        if iss.dir() == 0 {
            wrong_sysreg!("write");
        }
        macro_rules! read_sys_reg {
            ($($reg:ident => $id:ident),* $(,)?) => {
                match iss.sys_reg() {
                    $(
                        SysReg::$reg => unsafe {
                            let mut val = 0u64;
                            std::arch::asm!(concat!("mrs {}, ", stringify!($id)), out(reg) val);
                            self.write_reg(Reg::from(iss.rt()), val);
                        }
                    )*
                    _ => wrong_sysreg!("read"),
                }
            };
        }
        read_sys_reg! {
            APL_ACNTPCT_EL0 => s3_4_c15_c10_5,
            APL_ACNTVCT_EL0 => s3_4_c15_c10_6,
        }
    }
}

impl Cpu {
    pub fn run(&mut self) {
        loop {
            let exit = {
                hv_call!(hv_vcpu_run(self.vcpu));
                unsafe { *self.exit.as_ref() }
            };
            match exit.reason {
                HV_EXIT_REASON_CANCELED => break,
                HV_EXIT_REASON_EXCEPTION => self.handle_exc(exit.exception.into()),
                HV_EXIT_REASON_VTIMER_ACTIVATED => todo!("timer: {exit:#?}"),
                reason => panic!("unknown VM exit reason {reason}"),
            }
        }
    }
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        macro_rules! r {
            ($name:ident) => {
                self.read_reg(Reg::$name)
            };
        }
        macro_rules! s {
            ($name:ident) => {
                self.read_sys_reg(SysReg::$name)
            };
        }
        writeln!(f, "Debug dump of CPU {}:", self.vcpu)?;
        writeln!(f, "  Generic Registers:")?;
        writeln!(f, "     PC: {:016x}      SP: {:016x}", r!(PC), s!(SP_EL0))?;
        writeln!(f, "     FP: {:016x}      LR: {:016x}", r!(FP), r!(LR))?;
        writeln!(f, "     X0: {:016x}      X1: {:016x}", r!(X0), r!(X1))?;
        writeln!(f, "     X2: {:016x}      X3: {:016x}", r!(X2), r!(X3))?;
        writeln!(f, "     X4: {:016x}      X5: {:016x}", r!(X4), r!(X5))?;
        writeln!(f, "     X6: {:016x}      X7: {:016x}", r!(X6), r!(X7))?;
        writeln!(f, "     X8: {:016x}      X9: {:016x}", r!(X8), r!(X9))?;
        writeln!(f, "    X10: {:016x}     X11: {:016x}", r!(X10), r!(X11))?;
        writeln!(f, "    X12: {:016x}     X13: {:016x}", r!(X12), r!(X13))?;
        writeln!(f, "    X14: {:016x}     X15: {:016x}", r!(X14), r!(X15))?;
        writeln!(f, "    X16: {:016x}     X17: {:016x}", r!(X16), r!(X17))?;
        writeln!(f, "    X18: {:016x}     X19: {:016x}", r!(X18), r!(X19))?;
        writeln!(f, "    X20: {:016x}     X21: {:016x}", r!(X20), r!(X21))?;
        writeln!(f, "    X22: {:016x}     X23: {:016x}", r!(X22), r!(X23))?;
        writeln!(f, "    X24: {:016x}     X25: {:016x}", r!(X24), r!(X25))?;
        writeln!(f, "    X26: {:016x}     X27: {:016x}", r!(X26), r!(X27))?;
        writeln!(f, "    X28: {:016x}", r!(X28))?;
        writeln!(f)?;
        writeln!(f, "  Control & Status Registers:")?;
        writeln!(f, "          FPCR: {:016x}", r!(FPCR))?;
        writeln!(f, "          FPSR: {:016x}", r!(FPSR))?;
        writeln!(f, "          CPSR: {:016x}", r!(CPSR))?;
        writeln!(f, "      SPSR_EL1: {:016x}", s!(SPSR_EL1))?;
        writeln!(f, "       ELR_EL1: {:016x}", s!(ELR_EL1))?;
        writeln!(f, "       ESR_EL1: {:016x}", s!(ESR_EL1))?;
        writeln!(f, "       FAR_EL1: {:016x}", s!(FAR_EL1))?;
        writeln!(f, "       PAR_EL1: {:016x}", s!(PAR_EL1))?;
        writeln!(f, "     TPIDR_EL0: {:016x}", s!(TPIDR_EL0))?;
        writeln!(f, "   TPIDRRO_EL0: {:016x}", s!(TPIDRRO_EL0))?;
        writeln!(f, "     TPIDR_EL1: {:016x}", s!(TPIDR_EL1))?;
        writeln!(f, "     MDSCR_EL1: {:016x}", s!(MDSCR_EL1))?;
        writeln!(f, "     SCTLR_EL1: {:016x}", s!(SCTLR_EL1))?;
        writeln!(f, "     TTBR0_EL1: {:016x}", s!(TTBR0_EL1))?;
        writeln!(f, "     TTBR1_EL1: {:016x}", s!(TTBR1_EL1))?;
        Ok(())
    }
}
