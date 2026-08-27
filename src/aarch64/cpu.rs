use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    ptr::NonNull,
};

use crate::{
    aarch64::{
        disasm::disasm,
        ffi::*,
        paging::{MAIR_EL1_INIT, SCTLR_EL1_INIT, TCR_EL1_INIT},
        regs::*,
        syscall::Syscall,
        virtos::{
            IRQ_STUBS, SP_EL1,
            mem::VmMap,
            mmio::{MmioKind, MmioRequest, MmioResponse},
        },
    },
    hv_call,
    utils::{
        ptr::{Uintptr, VMA},
        str::Sz,
    },
};

#[derive(Clone, Copy)]
struct VmException {
    syndrome: Syndrome,
    virt_addr: VMA,
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
            virt_addr: VMA::new(exc.virtual_address),
            phys_addr: Uintptr::from(exc.physical_address),
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
        cpu.write_sys_reg(SysReg::TTBR0_EL1, VmMap::base().as_u64());
        cpu.write_sys_reg(SysReg::SCTLR_EL1, SCTLR_EL1_INIT);

        /* initialize the vCPU and set it to EL0 */
        cpu.write_reg(Reg::PC, pc);
        cpu.write_reg(Reg::CPSR, 0);
        cpu.write_sys_reg(SysReg::SP_EL0, sp);
        cpu.write_sys_reg(SysReg::SP_EL1, SP_EL1.addr());
        cpu.write_sys_reg(SysReg::VBAR_EL1, IRQ_STUBS.addr());
        cpu.write_sys_reg(SysReg::CPACR_EL1, CPACR_FPEN);
        cpu
    }
}

impl Cpu {
    #[inline]
    pub fn read_reg(&self, reg: Reg) -> u64 {
        let mut ret = 0u64;
        hv_call!(hv_vcpu_get_reg(self.vcpu, reg.reg(), &raw mut ret));
        ret
    }

    #[inline]
    pub fn write_reg(&self, reg: Reg, value: u64) {
        hv_call!(hv_vcpu_set_reg(self.vcpu, reg.reg(), value));
    }

    #[inline]
    pub fn read_sys_reg(&self, reg: SysReg) -> u64 {
        let mut ret = 0u64;
        hv_call!(hv_vcpu_get_sys_reg(self.vcpu, reg.sys_reg(), &raw mut ret));
        ret
    }

    #[inline]
    pub fn write_sys_reg(&self, reg: SysReg, value: u64) {
        hv_call!(hv_vcpu_set_sys_reg(self.vcpu, reg.sys_reg(), value));
    }
}

impl Cpu {
    #[inline]
    pub fn set_single_step(&self, is_enabled: bool) {
        let flags = (-(is_enabled as i64) as u64) & MDSCR_SS;
        let mdscr = self.read_sys_reg(SysReg::MDSCR_EL1) & !MDSCR_SS;
        self.write_sys_reg(SysReg::MDSCR_EL1, mdscr | flags);
    }
}

impl Cpu {
    fn handle_exc(&mut self, exc: VmException) {
        match Exception::from(exc.syndrome.EC()) {
            Exception::AA64_HVC => {
                let esr = self.read_sys_reg(SysReg::ESR_EL1);
                let elr = self.read_sys_reg(SysReg::ELR_EL1);
                self.handle_user_exc(Syndrome(esr), VMA::new(elr));
            }
            Exception::INST_ABORT => {
                let pc = VMA::new(self.read_reg(Reg::PC));
                let iss = InstAbortISS(exc.syndrome.ISS() as u32);
                self.handle_inst_abort(pc, iss, exc.virt_addr);
            }
            Exception::DATA_ABORT => {
                let pc = VMA::new(self.read_reg(Reg::PC));
                let iss = DataAbortISS(exc.syndrome.ISS() as u32);
                self.handle_data_abort(pc, iss, exc.virt_addr);
            }
            Exception::SOFTWARE_STEP => {
                tracing::trace!("SINGLE_STEP: {}", disasm(VMA::new(self.read_reg(Reg::PC))));
                self.write_reg(Reg::CPSR, self.read_reg(Reg::CPSR) | PSTATE_SS);
            }
            // TODO: remove this
            Exception::AA64_BKPT => {
                let pc = VMA::new(self.read_reg(Reg::PC));
                let ptr = pc + (0x1e9c37580u64 - 0x180324818u64);
                let msg = VmMap::translate(ptr).unwrap().read::<Sz>();
                eprintln!(
                    "BREAKPOINT: {msg}\nInstruction:\n  {insn}\nException: {exc:#?}\n{self:#?}",
                    insn = disasm(pc)
                );
                std::intrinsics::breakpoint();
            }
            ec => {
                panic!(
                    "unhandled exception {ec:?}:\nInstruction:\n  {insn}\nException: \
                     {exc:#?}\n{self:#?}",
                    insn = disasm(VMA::new(self.read_reg(Reg::PC)))
                );
            }
        }
    }

    fn handle_user_exc(&mut self, esr: Syndrome, elr: VMA) {
        match Exception::from(esr.EC()) {
            Exception::AA64_SVC => {
                let mut syscall = Syscall::read(self);
                syscall.dispatch();
                syscall.finalize();
            }
            Exception::SYS_REG_TRAP => {
                let iss = SysRegTrapISS(esr.ISS() as u32);
                self.handle_sysreg_trap(iss);
                self.write_sys_reg(SysReg::ELR_EL1, elr.addr() + 4);
            }
            Exception::INST_ABORT => {
                let far = VMA::new(self.read_sys_reg(SysReg::FAR_EL1));
                let iss = InstAbortISS(esr.ISS() as u32);
                self.handle_user_inst_abort(elr, iss, far);
            }
            Exception::DATA_ABORT => {
                let far = VMA::new(self.read_sys_reg(SysReg::FAR_EL1));
                let iss = DataAbortISS(esr.ISS() as u32);
                self.handle_user_data_abort(elr, iss, far);
            }
            ec => {
                panic!(
                    "unhandled EL0 exception {ec:?}:\nInstruction:\n  {insn}\n{self:#?}",
                    insn = disasm(elr)
                );
            }
        }
    }

    fn handle_inst_abort(&mut self, pc: VMA, iss: InstAbortISS, addr: VMA) {
        if !iss.is_translation_fault() {
            panic!(
                "segmentation fault from instruction fetching\nAddress:\n  \
                 {addr:p}\nInstruction:\n  {insn}\n{self:#?}",
                insn = disasm(pc)
            );
        }
        let mut req = MmioRequest {
            addr,
            data: 0,
            size: 4,
            kind: MmioKind::Execution,
        };
        let Some(resp) = VmMap::handle_mmio(pc, &mut req) else {
            panic!(
                "unhandled page fault from instruction fetching\nAddress:\n  \
                 {addr:p}\nInstruction:\n  {insn}\n{self:#?}",
                insn = disasm(pc)
            );
        };
        assert_eq!(
            resp,
            MmioResponse::Retry,
            "invalid handling of page fault from instruction fetching"
        );
    }

    fn handle_data_abort(&mut self, pc: VMA, iss: DataAbortISS, addr: VMA) {
        if iss.CM() == 1 {
            self.write_reg(Reg::PC, pc.addr() + 4);
            return;
        }
        if iss.S1PTW() == 1 {
            panic!("page table fault on state 1 translation lookup");
        }
        if !iss.is_translation_fault() {
            panic!(
                "segmentation fault\nAddress:\n  {addr:p}\nInstruction:\n  {insn}\n{self:#?}",
                insn = disasm(pc)
            );
        }
        let mut req = MmioRequest {
            addr,
            data: 0,
            size: 0,
            kind: MmioKind::Read,
        };
        if iss.WnR() == 1 {
            req.kind = MmioKind::Write;
        }
        if iss.ISV() == 1 {
            if iss.AR() == 1 {
                if iss.WnR() == 1 {
                    req.kind = MmioKind::ReadAtomic;
                } else {
                    req.kind = MmioKind::WriteAtomic;
                }
            }
            if iss.AR() == 0 && iss.WnR() == 1 {
                req.data = self.read_reg(Reg::from(iss.SRT()));
            }
            req.size = 1 << iss.SAS();
        }
        let Some(resp) = VmMap::handle_mmio(pc, &mut req) else {
            panic!(
                "unhandled page fault: {self:#?}\nAddress:\n  {addr:p}\nInstruction:\n  {insn}",
                insn = disasm(pc)
            );
        };
        if resp == MmioResponse::Retry {
            return;
        }
        assert!(
            iss.ISV() != 0,
            "MMIO cannot make more progress without valid ISV"
        );
        if iss.WnR() == 0 {
            match (iss.SAS(), iss.SSE()) {
                (0b00, 0b0) => req.data &= u8::MAX as u64,
                (0b01, 0b0) => req.data &= u16::MAX as u64,
                (0b10, 0b0) => req.data &= u32::MAX as u64,
                (0b00, 0b1) => req.data = req.data as i8 as u64,
                (0b01, 0b1) => req.data = req.data as i16 as u64,
                (0b10, 0b1) => req.data = req.data as i32 as u64,
                (0b11, ..) => {}
                _ => unreachable!(),
            };
            if iss.SF() == 0 {
                req.data &= u32::MAX as u64;
            }
            self.write_reg(Reg::from(iss.SRT()), req.data);
        }
        self.write_reg(Reg::PC, pc.addr() + 4);
    }

    fn handle_sysreg_trap(&mut self, iss: SysRegTrapISS) {
        macro_rules! wrong_sysreg {
            ($kind:literal) => {
                panic!(
                    "unhandled SysReg {ty} to \
                     \"S{op0}_{op1}_C{crn}_C{crm}_{op2}\"\nInstruction:\n  \
                     {insn}\nISS={iss:#?}\n{self:#?}",
                    ty = $kind,
                    op0 = iss.Op0(),
                    op1 = iss.Op1(),
                    crn = iss.CRn(),
                    crm = iss.CRm(),
                    op2 = iss.Op2(),
                    insn = disasm(VMA::new(self.read_sys_reg(SysReg::ELR_EL1)))
                )
            };
            ($kind:literal, $reg:expr) => {
                panic!(
                    "unhandled SysReg {ty} to \"{reg:?}\"\nInstruction:\n  \
                     {insn}\nISS={iss:#?}\n{self:#?}",
                    ty = $kind,
                    reg = $reg,
                    insn = disasm(VMA::new(self.read_sys_reg(SysReg::ELR_EL1)))
                )
            };
        }
        if iss.dir() == 0 {
            if let Ok(reg) = iss.sys_reg() {
                wrong_sysreg!("write", reg);
            } else {
                wrong_sysreg!("write");
            }
        }
        macro_rules! read_sys_reg {
            ($($reg:ident => $id:ident),* $(,)?) => {
                match iss.sys_reg() {
                    $(
                        Ok(SysReg::$reg) => unsafe {
                            let mut val = 0u64;
                            std::arch::asm!(concat!("mrs {}, ", stringify!($id)), out(reg) val);
                            self.write_reg(Reg::from(iss.Rt()), val);
                        }
                    )*
                    Ok(reg) => wrong_sysreg!("read", reg),
                    _ => wrong_sysreg!("read"),
                }
            };
        }
        read_sys_reg! {
            APL_ACNTPCT_EL0 => s3_4_c15_c10_5,
            APL_ACNTVCT_EL0 => s3_4_c15_c10_6,
        }
    }

    fn handle_user_inst_abort(&mut self, pc: VMA, iss: InstAbortISS, addr: VMA) {
        assert!(
            iss.is_translation_fault(),
            "segmentation fault at EL0 from instruction fetching\nAddress:\n  \
             {addr:p}\nInstruction:\n  {insn}\n{self:#?}",
            insn = disasm(pc)
        );
        if !VmMap::handle_page_fault(addr, MmioKind::Execution) {
            panic!(
                "bus fault at EL0 from instruction fetching\nAddress:\n  \
                 {addr:p}\nInstruction:\n  {insn}\n{self:#?}",
                insn = disasm(pc)
            );
        } else {
            self.flush_tlb(addr, 1);
        }
    }

    fn handle_user_data_abort(&mut self, pc: VMA, iss: DataAbortISS, addr: VMA) {
        assert!(
            iss.is_translation_fault(),
            "segmentation fault at EL0\nAddress:\n  {addr:p}\nInstruction:\n  {insn}\n{self:#?}",
            insn = disasm(pc)
        );
        let kind = {
            if iss.WnR() == 1 {
                MmioKind::Write
            } else {
                MmioKind::Read
            }
        };
        if !VmMap::handle_page_fault(addr, kind) {
            panic!(
                "bus fault at EL0\nAddress:\n  {addr:p}\nInstruction:\n  {insn}\n{self:#?}",
                insn = disasm(pc)
            );
        } else {
            self.flush_tlb(addr, 1);
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
        writeln!(f, "    X28: {:016x}      PC: {:016x}", r!(X28), r!(PC))?;
        writeln!(f)?;
        writeln!(f, "  Control & Status Registers:")?;
        writeln!(f, "          FPCR: {:016x}", r!(FPCR))?;
        writeln!(f, "          FPSR: {:016x}", r!(FPSR))?;
        writeln!(f, "          CPSR: {:016x}", r!(CPSR))?;
        writeln!(f, "      SPSR_EL1: {:016x}", s!(SPSR_EL1))?;
        writeln!(f, "        SP_EL0: {:016x}", s!(SP_EL0))?;
        writeln!(f, "        SP_EL1: {:016x}", s!(SP_EL1))?;
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
