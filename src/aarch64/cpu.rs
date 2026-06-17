use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};

use super::{consts::*, ffi::*, vm::VM};
use crate::{
    hv_call,
    macros::{declare_friendly_enum, define_accessors, define_bit_field},
    ptr::Uintptr,
};

declare_friendly_enum! {
    pub enum Reg : hv_reg_t [ u32 ] => HV_REG_ :: {
        X0,
        X1,
        X2,
        X3,
        X4,
        X5,
        X6,
        X7,
        X8,
        X9,
        X10,
        X11,
        X12,
        X13,
        X14,
        X15,
        X16,
        X17,
        X18,
        X19,
        X20,
        X21,
        X22,
        X23,
        X24,
        X25,
        X26,
        X27,
        X28,
        X29,
        X30,
        PC,
        FPCR,
        FPSR,
        CPSR,
    },
    pub enum SysReg : hv_sys_reg_t [ u16 ] => HV_SYS_REG_ :: {
        DBGBVR0_EL1,
        DBGBCR0_EL1,
        DBGWVR0_EL1,
        DBGWCR0_EL1,
        DBGBVR1_EL1,
        DBGBCR1_EL1,
        DBGWVR1_EL1,
        DBGWCR1_EL1,
        MDCCINT_EL1,
        MDSCR_EL1,
        DBGBVR2_EL1,
        DBGBCR2_EL1,
        DBGWVR2_EL1,
        DBGWCR2_EL1,
        DBGBVR3_EL1,
        DBGBCR3_EL1,
        DBGWVR3_EL1,
        DBGWCR3_EL1,
        DBGBVR4_EL1,
        DBGBCR4_EL1,
        DBGWVR4_EL1,
        DBGWCR4_EL1,
        DBGBVR5_EL1,
        DBGBCR5_EL1,
        DBGWVR5_EL1,
        DBGWCR5_EL1,
        DBGBVR6_EL1,
        DBGBCR6_EL1,
        DBGWVR6_EL1,
        DBGWCR6_EL1,
        DBGBVR7_EL1,
        DBGBCR7_EL1,
        DBGWVR7_EL1,
        DBGWCR7_EL1,
        DBGBVR8_EL1,
        DBGBCR8_EL1,
        DBGWVR8_EL1,
        DBGWCR8_EL1,
        DBGBVR9_EL1,
        DBGBCR9_EL1,
        DBGWVR9_EL1,
        DBGWCR9_EL1,
        DBGBVR10_EL1,
        DBGBCR10_EL1,
        DBGWVR10_EL1,
        DBGWCR10_EL1,
        DBGBVR11_EL1,
        DBGBCR11_EL1,
        DBGWVR11_EL1,
        DBGWCR11_EL1,
        DBGBVR12_EL1,
        DBGBCR12_EL1,
        DBGWVR12_EL1,
        DBGWCR12_EL1,
        DBGBVR13_EL1,
        DBGBCR13_EL1,
        DBGWVR13_EL1,
        DBGWCR13_EL1,
        DBGBVR14_EL1,
        DBGBCR14_EL1,
        DBGWVR14_EL1,
        DBGWCR14_EL1,
        DBGBVR15_EL1,
        DBGBCR15_EL1,
        DBGWVR15_EL1,
        DBGWCR15_EL1,
        MIDR_EL1,
        MPIDR_EL1,
        ID_AA64PFR0_EL1,
        ID_AA64PFR1_EL1,
        ID_AA64ZFR0_EL1,
        ID_AA64SMFR0_EL1,
        ID_AA64DFR0_EL1,
        ID_AA64DFR1_EL1,
        ID_AA64ISAR0_EL1,
        ID_AA64ISAR1_EL1,
        ID_AA64MMFR0_EL1,
        ID_AA64MMFR1_EL1,
        ID_AA64MMFR2_EL1,
        SCTLR_EL1,
        ACTLR_EL1,
        CPACR_EL1,
        SMPRI_EL1,
        SMCR_EL1,
        TTBR0_EL1,
        TTBR1_EL1,
        TCR_EL1,
        APIAKEYLO_EL1,
        APIAKEYHI_EL1,
        APIBKEYLO_EL1,
        APIBKEYHI_EL1,
        APDAKEYLO_EL1,
        APDAKEYHI_EL1,
        APDBKEYLO_EL1,
        APDBKEYHI_EL1,
        APGAKEYLO_EL1,
        APGAKEYHI_EL1,
        SPSR_EL1,
        ELR_EL1,
        SP_EL0,
        AFSR0_EL1,
        AFSR1_EL1,
        ESR_EL1,
        FAR_EL1,
        PAR_EL1,
        MAIR_EL1,
        AMAIR_EL1,
        VBAR_EL1,
        CONTEXTIDR_EL1,
        TPIDR_EL1,
        SCXTNUM_EL1,
        CNTKCTL_EL1,
        CSSELR_EL1,
        TPIDR_EL0,
        TPIDRRO_EL0,
        TPIDR2_EL0,
        SCXTNUM_EL0,
        CNTV_CTL_EL0,
        CNTV_CVAL_EL0,
        SP_EL1,
        CNTP_CTL_EL0,
        CNTP_CVAL_EL0,
        CNTP_TVAL_EL0,
        CNTHCTL_EL2,
        CNTHP_CTL_EL2,
        CNTHP_CVAL_EL2,
        CNTHP_TVAL_EL2,
        CNTVOFF_EL2,
        CPTR_EL2,
        ELR_EL2,
        ESR_EL2,
        FAR_EL2,
        HCR_EL2,
        HPFAR_EL2,
        MAIR_EL2,
        MDCR_EL2,
        SCTLR_EL2,
        SPSR_EL2,
        SP_EL2,
        TCR_EL2,
        TPIDR_EL2,
        TTBR0_EL2,
        TTBR1_EL2,
        VBAR_EL2,
        VMPIDR_EL2,
        VPIDR_EL2,
        VTCR_EL2,
        VTTBR_EL2,
    },
    pub enum Exception : u64 [ u64 ] => EC_ :: {
        UNCATEGORIZED,
        WFX_TRAP,
        CP15RT_TRAP,
        CP15RRT_TRAP,
        CP14RT_TRAP,
        CP14DT_TRAP,
        ADVSIMD_FP_ACCESS_TRAP,
        FPID_TRAP,
        PAC_TRAP,
        BXJ_TRAP,
        CP14RRT_TRAP,
        BTI_TRAP,
        ILLEGAL_STATE,
        AA32_SVC,
        AA32_HVC,
        AA32_SMC,
        AA64_SVC,
        AA64_HVC,
        AA64_SMC,
        SYS_REG_TRAP,
        SVE_ACCESS_TRAP,
        ERET_TRAP,
        PAC_FAIL,
        SME_TRAP,
        GPC,
        INST_ABORT,
        INST_ABORT_SAME_EL,
        PC_ALIGN,
        DATA_ABORT,
        DATA_ABORT_SAME_EL,
        SP_ALIGN,
        MOP,
        AA32_FPTRAP,
        AA64_FPTRAP,
        GCS,
        SERROR,
        BREAKPOINT,
        BREAKPOINT_SAME_EL,
        SOFTWARE_STEP,
        SOFTWARE_STEP_SAME_EL,
        WATCHPOINT,
        WATCHPOINT_SAME_EL,
        AA32_BKPT,
        VECTOR_CATCH,
        AA64_BKPT,
    }
}

impl Reg {
    pub const FP: Self = Self::X29;
    pub const LR: Self = Self::X30;
}

define_bit_field! {
    struct Syndrome : u64 {
        iss      : 25,  // Instruction Specific Syndrome
        length   : 1,   // Instruction Length
        class    : 6,   // Exception Class
        iss2     : 5,   // Instruction Specific Syndrome 2
        reserved : 27,  // Reserved
    }
}

#[derive(Clone, Copy)]
struct VmException {
    syndrome: Syndrome,
    virt_addr: u64,
    phys_addr: u64,
}

impl Debug for VmException {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("VmException")
            .field("syndrome", &self.syndrome)
            .field_with("virt_addr", |f| write!(f, "0x{:x}", self.virt_addr))
            .field_with("phys_addr", |f| write!(f, "0x{:x}", self.phys_addr))
            .finish()
    }
}

impl From<hv_vcpu_exit_exception_t> for VmException {
    fn from(exc: hv_vcpu_exit_exception_t) -> Self {
        Self {
            syndrome: Syndrome(exc.syndrome),
            virt_addr: exc.virtual_address,
            phys_addr: exc.physical_address,
        }
    }
}

pub struct Cpu {
    cpu: hv_vcpu_t,
    run: AtomicBool,
    vmx: NonNull<hv_vcpu_exit_t>,
}

impl Cpu {
    pub(super) fn new(pc: Uintptr, sp: Uintptr) -> Self {
        let mut id = 0u64;
        let mut exit = std::ptr::null_mut();
        let cfg = unsafe { hv_vcpu_config_create() };

        /* allocate the vCPU */
        hv_call!(hv_vcpu_create(&raw mut id, &raw mut exit, cfg));
        hv_call!(hv_vcpu_set_trap_debug_exceptions(id, true));
        hv_call!(hv_vcpu_set_trap_debug_reg_accesses(id, true));

        /* construct the CPU state */
        let cpu = Self {
            cpu: id,
            run: AtomicBool::new(true),
            vmx: NonNull::new(exit).expect("null VM exit buffer"),
        };

        /* initialize the vCPU with PC & SP, and set it to EL0 */
        cpu.write_reg(Reg::PC, pc.as_u64());
        cpu.write_reg(Reg::CPSR, 0);
        cpu.write_sys_reg(SysReg::SP_EL0, sp.as_u64());
        cpu
    }
}

define_accessors! {
    reg     : u64 = (reg: Reg)        :: hv_vcpu_get_reg     -> hv_vcpu_set_reg,
    sys_reg : u64 = (sys_reg: SysReg) :: hv_vcpu_get_sys_reg -> hv_vcpu_set_sys_reg,
}

impl Cpu {
    fn handle_exception(&self, exc: VmException) {
        match Exception::from(exc.syndrome.class()) {
            Exception::AA64_HVC => {
                let mut x0 = self.read_reg(Reg::X0);
                let mut x1 = self.read_reg(Reg::X1);
                let x2 = self.read_reg(Reg::X2);
                let x3 = self.read_reg(Reg::X3);
                let x4 = self.read_reg(Reg::X4);
                let x5 = self.read_reg(Reg::X5);
                let num = self.read_reg(Reg::X16);
                eprintln!(
                    "HVC: {exc:#?}, num={num}, x0=0x{x0:x}, x1=0x{x1:x}, x2=0x{x2:x}, \
                     x3=0x{x3:x}, x4=0x{x4:x}, x5=0x{x5:x}"
                );
                if num == 4 {
                    x1 = VM.translate(x1.into()).expect("invalid address").as_u64();
                }
                unsafe { x0 = libc::syscall(num as i32, x0, x1, x2, x3, x4, x5) as u64 };
                self.write_reg(Reg::X0, x0);
            }
            Exception::DATA_ABORT => {
                eprintln!("DATA_ABORT: {exc:#?}");
                todo!()
            }
            ec => {
                let pc = self.read_reg(Reg::PC);
                panic!("unhandled exception {ec:?} at 0x{pc:x}: {exc:#?}");
            }
        }
    }
}

impl Cpu {
    pub fn run(&self) {
        while self.run.load(Ordering::Acquire) {
            let vmx = {
                hv_call!(hv_vcpu_run(self.cpu));
                unsafe { *self.vmx.as_ref() }
            };
            match vmx.reason {
                HV_EXIT_REASON_CANCELED => break,
                HV_EXIT_REASON_EXCEPTION => self.handle_exception(vmx.exception.into()),
                HV_EXIT_REASON_VTIMER_ACTIVATED => todo!("timer"),
                reason => panic!("unknown VM exit reason {reason}"),
            }
        }
    }
}
