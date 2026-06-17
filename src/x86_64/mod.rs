pub mod consts;
pub mod ffi;

use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    io::{Error as IoError, Write},
    sync::atomic::{AtomicBool, Ordering},
};

use bytes::BufMut;
use consts::*;
use ffi::*;

use crate::{
    Memory, Protection, Unit, hv_call,
    macros::{declare_friendly_enum, define_accessors},
};

declare_friendly_enum! {
    pub enum Reg : hv_x86_reg_t [ u32 ] => HV_X86_ :: {
        RIP,
        RFLAGS,
        RAX,
        RCX,
        RDX,
        RBX,
        RSI,
        RDI,
        RSP,
        RBP,
        R8,
        R9,
        R10,
        R11,
        R12,
        R13,
        R14,
        R15,
        CS,
        SS,
        DS,
        ES,
        FS,
        GS,
        IDT_BASE,
        IDT_LIMIT,
        GDT_BASE,
        GDT_LIMIT,
        LDTR,
        LDT_BASE,
        LDT_LIMIT,
        LDT_AR,
        TR,
        TSS_BASE,
        TSS_LIMIT,
        TSS_AR,
        CR0,
        CR1,
        CR2,
        CR3,
        CR4,
        DR0,
        DR1,
        DR2,
        DR3,
        DR4,
        DR5,
        DR6,
        DR7,
        TPR,
        XCR0,
    },
    pub enum Msr : u32 [ u32 ] => HV_MSR_ :: {
        IA32_TSC,
        IA32_SPEC_CTRL,
        IA32_PRED_CMD,
        IA32_PMC0,
        IA32_PMC7,
        IA32_ARCH_CAPABILITIES,
        IA32_FLUSH_CMD,
        IA32_SYSENTER_CS,
        IA32_SYSENTER_ESP,
        IA32_SYSENTER_EIP,
        IA32_PERFEVNTSEL0,
        IA32_PERFEVNTSEL7,
        LBR_SELECT,
        LASTBRANCH_TOS,
        LASTINT_FROM_IP,
        LASTINT_TO_IP,
        IA32_DEBUGCTL,
        IA32_FIXED_CTR0,
        IA32_FIXED_CTR1,
        IA32_FIXED_CTR2,
        IA32_FIXED_CTR3,
        PERF_METRICS,
        IA32_FIXED_CTR_CTRL,
        IA32_PERF_GLOBAL_STATUS,
        IA32_PERF_GLOBAL_CTRL,
        IA32_PERF_GLOBAL_STATUS_RESET,
        IA32_PERF_GLOBAL_STATUS_SET,
        IA32_PERF_GLOBAL_INUSE,
        IA32_A_PMC0,
        IA32_A_PMC7,
        LASTBRANCH_0_FROM_IP,
        LASTBRANCH_31_FROM_IP,
        LASTBRANCH_0_TO_IP,
        LASTBRANCH_31_TO_IP,
        IA32_XSS,
        LASTBRANCH_INFO_0,
        LASTBRANCH_INFO_31,
        IA32_EFER,
        IA32_STAR,
        IA32_LSTAR,
        IA32_CSTAR,
        IA32_FMASK,
        IA32_FS_BASE,
        IA32_GS_BASE,
        IA32_KERNEL_GS_BASE,
        IA32_TSC_AUX,
    },
    pub enum Vmcs : u32 [ u32 ] => VMCS_ :: {
        VPID,
        CTRL_POSTED_INT_N_VECTOR,
        CTRL_EPTP_INDEX,
        GUEST_ES,
        GUEST_CS,
        GUEST_SS,
        GUEST_DS,
        GUEST_FS,
        GUEST_GS,
        GUEST_LDTR,
        GUEST_TR,
        GUEST_INT_STATUS,
        HOST_ES,
        HOST_CS,
        HOST_SS,
        HOST_DS,
        HOST_FS,
        HOST_GS,
        HOST_TR,
        CTRL_IO_BITMAP_A,
        CTRL_IO_BITMAP_B,
        CTRL_MSR_BITMAPS,
        CTRL_VMEXIT_MSR_STORE_ADDR,
        CTRL_VMEXIT_MSR_LOAD_ADDR,
        CTRL_VMENTRY_MSR_LOAD_ADDR,
        CTRL_EXECUTIVE_VMCS_PTR,
        CTRL_TSC_OFFSET,
        CTRL_VIRTUAL_APIC,
        CTRL_APIC_ACCESS,
        CTRL_POSTED_INT_DESC_ADDR,
        CTRL_VMFUNC_CTRL,
        CTRL_EPTP,
        CTRL_EOI_EXIT_BITMAP_0,
        CTRL_EOI_EXIT_BITMAP_1,
        CTRL_EOI_EXIT_BITMAP_2,
        CTRL_EOI_EXIT_BITMAP_3,
        CTRL_EPTP_LIST_ADDR,
        CTRL_VMREAD_BITMAP_ADDR,
        CTRL_VMWRITE_BITMAP_ADDR,
        CTRL_VIRT_EXC_INFO_ADDR,
        CTRL_XSS_EXITING_BITMAP,
        GUEST_PHYSICAL_ADDRESS,
        GUEST_LINK_POINTER,
        GUEST_IA32_DEBUGCTL,
        GUEST_IA32_PAT,
        GUEST_IA32_EFER,
        GUEST_IA32_PERF_GLOBAL_CTRL,
        GUEST_PDPTE0,
        GUEST_PDPTE1,
        GUEST_PDPTE2,
        GUEST_PDPTE3,
        HOST_IA32_PAT,
        HOST_IA32_EFER,
        HOST_IA32_PERF_GLOBAL_CTRL,
        CTRL_PIN_BASED,
        CTRL_CPU_BASED,
        CTRL_EXC_BITMAP,
        CTRL_PF_ERROR_MASK,
        CTRL_PF_ERROR_MATCH,
        CTRL_CR3_COUNT,
        CTRL_VMEXIT_CONTROLS,
        CTRL_VMEXIT_MSR_STORE_COUNT,
        CTRL_VMEXIT_MSR_LOAD_COUNT,
        CTRL_VMENTRY_CONTROLS,
        CTRL_VMENTRY_MSR_LOAD_COUNT,
        CTRL_VMENTRY_IRQ_INFO,
        CTRL_VMENTRY_EXC_ERROR,
        CTRL_VMENTRY_INSTR_LEN,
        CTRL_TPR_THRESHOLD,
        CTRL_CPU_BASED2,
        CTRL_PLE_GAP,
        CTRL_PLE_WINDOW,
        RO_INSTR_ERROR,
        RO_EXIT_REASON,
        RO_VMEXIT_IRQ_INFO,
        RO_VMEXIT_IRQ_ERROR,
        RO_IDT_VECTOR_INFO,
        RO_IDT_VECTOR_ERROR,
        RO_VMEXIT_INSTR_LEN,
        RO_VMX_INSTR_INFO,
        GUEST_ES_LIMIT,
        GUEST_CS_LIMIT,
        GUEST_SS_LIMIT,
        GUEST_DS_LIMIT,
        GUEST_FS_LIMIT,
        GUEST_GS_LIMIT,
        GUEST_LDTR_LIMIT,
        GUEST_TR_LIMIT,
        GUEST_GDTR_LIMIT,
        GUEST_IDTR_LIMIT,
        GUEST_ES_AR,
        GUEST_CS_AR,
        GUEST_SS_AR,
        GUEST_DS_AR,
        GUEST_FS_AR,
        GUEST_GS_AR,
        GUEST_LDTR_AR,
        GUEST_TR_AR,
        GUEST_IGNORE_IRQ,
        GUEST_ACTIVITY_STATE,
        GUEST_SMBASE,
        GUEST_IA32_SYSENTER_CS,
        GUEST_VMX_TIMER_VALUE,
        HOST_IA32_SYSENTER_CS,
        CTRL_CR0_MASK,
        CTRL_CR4_MASK,
        CTRL_CR0_SHADOW,
        CTRL_CR4_SHADOW,
        CTRL_CR3_VALUE0,
        CTRL_CR3_VALUE1,
        CTRL_CR3_VALUE2,
        CTRL_CR3_VALUE3,
        RO_EXIT_QUALIFIC,
        RO_IO_RCX,
        RO_IO_RSI,
        RO_IO_RDI,
        RO_IO_RIP,
        RO_GUEST_LIN_ADDR,
        GUEST_CR0,
        GUEST_CR3,
        GUEST_CR4,
        GUEST_ES_BASE,
        GUEST_CS_BASE,
        GUEST_SS_BASE,
        GUEST_DS_BASE,
        GUEST_FS_BASE,
        GUEST_GS_BASE,
        GUEST_LDTR_BASE,
        GUEST_TR_BASE,
        GUEST_GDTR_BASE,
        GUEST_IDTR_BASE,
        GUEST_DR7,
        GUEST_RSP,
        GUEST_RIP,
        GUEST_RFLAGS,
        GUEST_DEBUG_EXC,
        GUEST_SYSENTER_ESP,
        GUEST_SYSENTER_EIP,
        HOST_CR0,
        HOST_CR3,
        HOST_CR4,
        HOST_FS_BASE,
        HOST_GS_BASE,
        HOST_TR_BASE,
        HOST_GDTR_BASE,
        HOST_IDTR_BASE,
        HOST_IA32_SYSENTER_ESP,
        HOST_IA32_SYSENTER_EIP,
        HOST_RSP,
        HOST_RIP,
        MAX,
    },
    pub enum ExitReason : u64 [ u64 ] => VMX_REASON_ :: {
        EXC_NMI,
        IRQ,
        TRIPLE_FAULT,
        INIT,
        SIPI,
        IO_SMI,
        OTHER_SMI,
        IRQ_WND,
        VIRTUAL_NMI_WND,
        TASK,
        CPUID,
        GETSEC,
        HLT,
        INVD,
        INVLPG,
        RDPMC,
        RDTSC,
        RSM,
        VMCALL,
        VMCLEAR,
        VMLAUNCH,
        VMPTRLD,
        VMPTRST,
        VMREAD,
        VMRESUME,
        VMWRITE,
        VMOFF,
        VMON,
        MOV_CR,
        MOV_DR,
        IO,
        RDMSR,
        WRMSR,
        VMENTRY_GUEST,
        VMENTRY_MSR,
        MWAIT,
        MTF,
        MONITOR,
        PAUSE,
        VMENTRY_MC,
        TPR_THRESHOLD,
        APIC_ACCESS,
        VIRTUALIZED_EOI,
        GDTR_IDTR,
        LDTR_TR,
        EPT_VIOLATION,
        EPT_MISCONFIG,
        EPT_INVEPT,
        RDTSCP,
        VMX_TIMER_EXPIRED,
        INVVPID,
        WBINVD,
        XSETBV,
        APIC_WRITE,
        RDRAND,
        INVPCID,
        VMFUNC,
        RDSEED,
        XSAVES,
        XRSTORS,
    },
    pub enum Capability : hv_vmx_capability_t [ u32 ] => HV_VMX_CAP_ :: {
        PINBASED,
        PROCBASED,
        PROCBASED2,
        ENTRY,
        EXIT,
        BASIC,
        TRUE_PINBASED,
        TRUE_PROCBASED,
        TRUE_ENTRY,
        TRUE_EXIT,
        MISC,
        CR0_FIXED0,
        CR0_FIXED1,
        CR4_FIXED0,
        CR4_FIXED1,
        VMCS_ENUM,
        EPT_VPID_CAP,
        PREEMPTION_TIMER,
    },
}

/// Format of Access Rights:
///   3-0 : T - Segment type
///   4   : S — Descriptor type (0 = system; 1 = code or data)
///   6-5 : DPL — Descriptor privilege level
///   7   : P — Segment present
///   11-8: Reserved
///   12  : AVL — Available for use by system software
///   13  : L — 64-bit mode active (for CS only)
///   14  : D/B — Default operation size (0 = 16-bit segment; 1 = 32-bit segment)
///   15  : G — Granularity
///   16  : U - Segment unusable (0 = usable; 1 = unusable)
#[derive(Clone, Copy)]
struct SegAR(u64);

impl Display for SegAR {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let t = self.0 & 0x0f;
        let s = (self.0 >> 4) & 1;
        let p = (self.0 >> 7) & 1;
        let l = (self.0 >> 13) & 1;
        let g = (self.0 >> 15) & 1;
        let u = (self.0 >> 16) & 1;
        let db = (self.0 >> 14) & 1;
        let dpl = (self.0 >> 5) & 3;
        let avl = (self.0 >> 12) & 1;
        write!(f, "{u} {g} {db:2} {l} {avl:3} {p} {dpl:3} {s} {t:2}")
    }
}

const CPU_BASED_FLAGS: u64 = CPU_BASED_HLT
    | CPU_BASED_MWAIT
    | CPU_BASED_TSC_OFFSET
    | CPU_BASED_TPR_SHADOW
    | CPU_BASED_SECONDARY_CTLS;

const PIN_BASED_FLAGS: u64 = PIN_BASED_INTR | PIN_BASED_NMI | PIN_BASED_VIRTUAL_NMI;
const CPU_BASED_2_FLAGS: u64 = CPU_BASED2_VIRTUAL_APIC | CPU_BASED2_RDTSCP;

pub struct Cpu {
    run: AtomicBool,
    cpu: hv_vcpuid_t,
}

impl Cpu {
    fn new(vm: &Vm, id: u32, rip: u64) -> Self {
        let entry = cap2ctrl(vm.caps(Capability::ENTRY), 0);
        let pin_based = cap2ctrl(vm.caps(Capability::PINBASED), PIN_BASED_FLAGS);
        let proc_based = cap2ctrl(vm.caps(Capability::PROCBASED), CPU_BASED_FLAGS);
        let proc_based_2 = cap2ctrl(vm.caps(Capability::PROCBASED2), CPU_BASED_2_FLAGS);

        /* construct the CPU instance */
        let cpu = Self {
            run: AtomicBool::new(true),
            cpu: id,
        };

        /* setup VM control registers */
        cpu.write_vmcs(Vmcs::CTRL_PIN_BASED, pin_based);
        cpu.write_vmcs(Vmcs::CTRL_CPU_BASED, proc_based);
        cpu.write_vmcs(Vmcs::CTRL_CPU_BASED2, proc_based_2);
        cpu.write_vmcs(Vmcs::CTRL_VMENTRY_CONTROLS, entry);
        cpu.write_vmcs(Vmcs::CTRL_EXC_BITMAP, 0);
        cpu.write_vmcs(Vmcs::CTRL_TPR_THRESHOLD, 0);

        /* enable native MSRs */
        cpu.enable_native_msr(Msr::IA32_STAR, true);
        cpu.enable_native_msr(Msr::IA32_LSTAR, true);
        cpu.enable_native_msr(Msr::IA32_CSTAR, true);
        cpu.enable_native_msr(Msr::IA32_FMASK, true);
        cpu.enable_native_msr(Msr::IA32_FS_BASE, true);
        cpu.enable_native_msr(Msr::IA32_GS_BASE, true);
        cpu.enable_native_msr(Msr::IA32_KERNEL_GS_BASE, true);
        cpu.enable_native_msr(Msr::IA32_TSC_AUX, true);
        cpu.enable_native_msr(Msr::IA32_TSC, true);
        cpu.enable_native_msr(Msr::IA32_SYSENTER_CS, true);
        cpu.enable_native_msr(Msr::IA32_SYSENTER_EIP, true);
        cpu.enable_native_msr(Msr::IA32_SYSENTER_ESP, true);

        /* reset the processor */
        cpu.reset(rip);
        cpu
    }
}

define_accessors! {
    msr  : u64 = (msr: Msr)   :: hv_vcpu_read_msr      -> hv_vcpu_write_msr,
    reg  : u64 = (reg: Reg)   :: hv_vcpu_read_register -> hv_vcpu_write_register,
    vmcs : u64 = (vmcs: Vmcs) :: hv_vmx_vcpu_read_vmcs -> hv_vmx_vcpu_write_vmcs,
}

impl Cpu {
    fn flush_tlb(&self) {
        hv_call!(hv_vcpu_invalidate_tlb(self.cpu));
    }

    fn enable_native_msr(&self, msr: Msr, enable: bool) {
        hv_call!(hv_vcpu_enable_native_msr(self.cpu, msr.msr(), enable));
    }
}

impl Cpu {
    fn next(&self) {
        let rip = self.read_reg(Reg::RIP);
        let len = self.read_vmcs(Vmcs::RO_VMEXIT_INSTR_LEN);
        self.write_reg(Reg::RIP, rip + len);
    }

    fn reset(&self, rip: u64) {
        macro_rules! set_segment {
            ($name:ident, $ar:literal) => {
                paste::paste! {
                    self.write_vmcs(Vmcs::[< GUEST_ $name >], 0);
                    self.write_vmcs(Vmcs::[< GUEST_ $name _AR >], $ar);
                    self.write_vmcs(Vmcs::[< GUEST_ $name _BASE >], 0);
                    self.write_vmcs(Vmcs::[< GUEST_ $name _LIMIT >], 0xffff);
                }
            };
        }

        /* general purpose registers */
        self.write_reg(Reg::RIP, rip);
        self.write_reg(Reg::RFLAGS, 2);
        self.write_reg(Reg::RAX, 0);
        self.write_reg(Reg::RCX, 0);
        self.write_reg(Reg::RDX, 0);
        self.write_reg(Reg::RBX, 0);
        self.write_reg(Reg::RSP, 0);
        self.write_reg(Reg::RBP, 0);
        self.write_reg(Reg::RSI, 0);
        self.write_reg(Reg::RDI, 0);
        self.write_reg(Reg::R8, 0);
        self.write_reg(Reg::R9, 0);
        self.write_reg(Reg::R10, 0);
        self.write_reg(Reg::R11, 0);
        self.write_reg(Reg::R12, 0);
        self.write_reg(Reg::R13, 0);
        self.write_reg(Reg::R14, 0);
        self.write_reg(Reg::R15, 0);
        self.write_reg(Reg::XCR0, 1);

        /* GDTR & IDTR */
        self.write_vmcs(Vmcs::GUEST_GDTR_BASE, 0);
        self.write_vmcs(Vmcs::GUEST_GDTR_LIMIT, 0xffff);
        self.write_vmcs(Vmcs::GUEST_IDTR_BASE, 0);
        self.write_vmcs(Vmcs::GUEST_IDTR_LIMIT, 0xffff);

        /* control registers */
        self.write_vmcs(Vmcs::GUEST_CR3, 0);
        self.write_reg(Reg::TPR, 0);
        self.write_vmcs(Vmcs::CTRL_TPR_THRESHOLD, 0);
        self.write_vmcs(Vmcs::GUEST_IA32_EFER, 0);
        self.set_cr4(0);
        self.set_cr0(0x60000010);

        /* segments */
        set_segment!(CS, 0x9b);
        set_segment!(DS, 0x93);
        set_segment!(ES, 0x93);
        set_segment!(FS, 0x93);
        set_segment!(GS, 0x93);
        set_segment!(SS, 0x93);
        set_segment!(TR, 0x8b);
        set_segment!(LDTR, 0x82);

        /* MSRs */
        self.write_msr(Msr::IA32_SYSENTER_CS, 0);
        self.write_msr(Msr::IA32_SYSENTER_ESP, 0);
        self.write_msr(Msr::IA32_SYSENTER_EIP, 0);
        self.write_msr(Msr::IA32_STAR, 0);
        self.write_msr(Msr::IA32_CSTAR, 0);
        self.write_msr(Msr::IA32_KERNEL_GS_BASE, 0);
        self.write_msr(Msr::IA32_FMASK, 0);
        self.write_msr(Msr::IA32_LSTAR, 0);
        self.write_msr(Msr::IA32_GS_BASE, 0);
        self.write_msr(Msr::IA32_FS_BASE, 0);

        /* debug registers */
        self.write_reg(Reg::DR0, 0);
        self.write_reg(Reg::DR1, 0);
        self.write_reg(Reg::DR2, 0);
        self.write_reg(Reg::DR3, 0);
        self.write_reg(Reg::DR4, 0);
        self.write_reg(Reg::DR5, 0);
        self.write_reg(Reg::DR6, 0xffff0ff0);
        self.write_reg(Reg::DR7, 0x00000400);
    }
}

const CR0_PE_MASK: u64 = 1 << 0;
const CR0_MP_MASK: u64 = 1 << 1;
const CR0_EM_MASK: u64 = 1 << 2;
const CR0_TS_MASK: u64 = 1 << 3;
const CR0_ET_MASK: u64 = 1 << 4;
const CR0_NE_MASK: u64 = 1 << 5;
const CR0_WP_MASK: u64 = 1 << 16;
const CR0_AM_MASK: u64 = 1 << 18;
const CR0_NW_MASK: u64 = 1 << 29;
const CR0_CD_MASK: u64 = 1 << 30;
const CR0_PG_MASK: u64 = 1 << 31;

const CR4_PAE_MASK: u64 = 1 << 5;
const CR4_VMXE_MASK: u64 = 1 << 13;

const EFER_SCE: u64 = 1 << 0;
const EFER_LME: u64 = 1 << 8;
const EFER_LMA: u64 = 1 << 10;
const EFER_NXE: u64 = 1 << 11;
const EFER_SVME: u64 = 1 << 12;
const EFER_FFXSR: u64 = 1 << 14;

const AR_TYPE_MASK: u64 = 0x0f;
const AR_TYPE_BUSY_64_TSS: u64 = 11;

impl Cpu {
    fn set_cr0(&self, cr0: u64) {
        let cr0p = self.read_vmcs(Vmcs::GUEST_CR0);
        let efer = self.read_vmcs(Vmcs::GUEST_IA32_EFER);
        let mask = CR0_PG_MASK | CR0_CD_MASK | CR0_NW_MASK | CR0_NE_MASK | CR0_ET_MASK;

        /* modify CR0 in long mode */
        if self.is_lme_ready(cr0, efer) {
            unimplemented!("modify CR0 in long mode");
        }

        /* update CR0 mask & shadow */
        self.write_vmcs(Vmcs::CTRL_CR0_MASK, mask);
        self.write_vmcs(Vmcs::CTRL_CR0_SHADOW, cr0);

        /* switching in and out of long mode */
        if efer & EFER_LME != 0 {
            if (cr0 ^ cr0p) & CR0_PG_MASK != 0 {
                if cr0 & CR0_PG_MASK != 0 {
                    self.enter_long_mode(efer);
                } else {
                    self.exit_long_mode(efer);
                }
            }
        } else {
            let entry = self.read_vmcs(Vmcs::CTRL_VMENTRY_CONTROLS);
            self.write_vmcs(Vmcs::CTRL_VMENTRY_CONTROLS, entry & !VMENTRY_GUEST_IA32E);
        }

        /* Filter new CR0 after we are finished examining it above. */
        let cr0 = cr0 & !(mask & !CR0_PG_MASK);
        self.write_vmcs(Vmcs::GUEST_CR0, cr0 | CR0_NE_MASK | CR0_ET_MASK);
        self.flush_tlb();
    }

    fn set_cr4(&self, cr4: u64) {
        self.write_vmcs(Vmcs::GUEST_CR4, cr4 | CR4_VMXE_MASK);
        self.write_vmcs(Vmcs::CTRL_CR4_MASK, CR4_VMXE_MASK);
        self.write_vmcs(Vmcs::CTRL_CR4_SHADOW, cr4);
        self.flush_tlb();
    }

    fn is_lme_ready(&self, cr0: u64, efer: u64) -> bool {
        cr0 & CR0_PG_MASK != 0
            && efer & EFER_LME == 0
            && self.read_vmcs(Vmcs::GUEST_CR4) & CR4_PAE_MASK != 0
    }

    fn exit_long_mode(&self, efer: u64) {
        let entry = self.read_vmcs(Vmcs::CTRL_VMENTRY_CONTROLS);
        self.write_vmcs(Vmcs::CTRL_VMENTRY_CONTROLS, entry & !VMENTRY_GUEST_IA32E);
        self.write_vmcs(Vmcs::GUEST_IA32_EFER, efer & !EFER_LMA);
    }

    fn enter_long_mode(&self, efer: u64) {
        let tr_ar = self.read_vmcs(Vmcs::GUEST_TR_AR);
        let entry = self.read_vmcs(Vmcs::CTRL_VMENTRY_CONTROLS);

        /* enable LMA & IA32E */
        self.write_vmcs(Vmcs::GUEST_IA32_EFER, efer | EFER_LMA);
        self.write_vmcs(Vmcs::CTRL_VMENTRY_CONTROLS, entry | VMENTRY_GUEST_IA32E);

        /* adjust access rights for TSS */
        if efer & EFER_LME != 0 && (tr_ar & AR_TYPE_MASK) != AR_TYPE_BUSY_64_TSS {
            self.write_vmcs(
                Vmcs::GUEST_TR_AR,
                (tr_ar & !AR_TYPE_MASK) | AR_TYPE_BUSY_64_TSS,
            );
        }
    }
}

impl Cpu {
    fn handle_rdmsr(&self) {
        match Msr::from(self.read_reg(Reg::RCX) as u32) {
            Msr::IA32_EFER => {
                let efer = self.read_vmcs(Vmcs::GUEST_IA32_EFER);
                eprintln!("RDMSR EFER => {efer:016x}");
                self.write_reg(Reg::RDX, efer >> 32);
                self.write_reg(Reg::RAX, efer & 0xffff_ffff);
                self.next();
            }
            msr => {
                dbg!(&self);
                unimplemented!("RDMSR: {msr:?}");
            }
        }
    }

    fn handle_wrmsr(&self) {
        match Msr::from(self.read_reg(Reg::RCX) as u32) {
            Msr::IA32_EFER => {
                let efer = (self.read_reg(Reg::RDX) << 32) | self.read_reg(Reg::RAX);
                eprintln!("WRMSR EFER <= {efer:016x}");
                self.write_vmcs(Vmcs::GUEST_IA32_EFER, efer);
                self.next();
            }
            msr => {
                dbg!(&self);
                unimplemented!("WRMSR: {msr:?}");
            }
        }
    }

    fn handle_mov_cr(&self) {
        let arg = self.read_vmcs(Vmcs::RO_EXIT_QUALIFIC);
        let val = self.read_reg(Reg::from((((arg >> 8) & 15) + 2) as u32));

        /* dispatch on CR index */
        match arg & 15 {
            0 => self.set_cr0(val),
            4 => self.set_cr4(val),
            8 => unimplemented!("CR8"),
            n => panic!("invalid register: CR{n}"),
        }

        /* advance to the next instruction */
        self.next();
    }

    fn unhandled_vm_exit(&self, reason: ExitReason) {
        dbg!(&self);
        if reason == ExitReason::EXC_NMI {
            let irq_info = self.read_vmcs(Vmcs::RO_VMEXIT_IRQ_INFO);
            eprintln!("irq_info = {irq_info:016x}");
            let vector = irq_info & 0xff;
            let ty = (irq_info >> 8) & 0x7;
            let valid = (irq_info >> 31) & 0x1;
            eprintln!("  vector={vector} ty={ty} valid={valid}");
        }
        std::io::stderr()
            .write_all(b"* Press ENTER to continue ...")
            .unwrap();
        std::io::stderr().flush().unwrap();
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }
}

impl Cpu {
    pub fn run(&self) {
        while self.run.load(Ordering::Acquire) {
            let reason = {
                hv_call!(hv_vcpu_run(self.cpu));
                self.read_vmcs(Vmcs::RO_EXIT_REASON)
            };
            if reason & VMX_FLAGS_ERROR != 0 {
                panic!("VM Enter error: {reason:08x}");
            }
            match ExitReason::from(reason & VMX_REASON_MASK) {
                ExitReason::MOV_CR => self.handle_mov_cr(),
                ExitReason::RDMSR => self.handle_rdmsr(),
                ExitReason::WRMSR => self.handle_wrmsr(),
                ExitReason::EPT_VIOLATION => {}
                reason => self.unhandled_vm_exit(reason),
            }
        }
    }
}

impl Cpu {
    fn dump_vmcs(&self, f: &mut Formatter<'_>) -> FmtResult {
        macro_rules! vmcs {
            ($name:ident) => {
                self.read_vmcs(Vmcs::$name)
            };
        }
        writeln!(f, "  VMCS:")?;
        writeln!(f, "    CR0 mask    : {:016x}", vmcs!(CTRL_CR0_MASK))?;
        writeln!(f, "    CR0 shadow  : {:016x}", vmcs!(CTRL_CR0_SHADOW))?;
        writeln!(f, "    CR4 mask    : {:016x}", vmcs!(CTRL_CR4_MASK))?;
        writeln!(f, "    CR4 shadow  : {:016x}", vmcs!(CTRL_CR4_SHADOW))?;
        writeln!(f, "    Pin Based   : {:016x}", vmcs!(CTRL_PIN_BASED))?;
        writeln!(f, "    CPU Based   : {:016x}", vmcs!(CTRL_CPU_BASED))?;
        writeln!(f, "    CPU Based 2 : {:016x}", vmcs!(CTRL_CPU_BASED2))?;
        writeln!(f, "    VM Entry    : {:016x}", vmcs!(CTRL_VMENTRY_CONTROLS))?;
        writeln!(f, "    VM Exit     : {:016x}", vmcs!(CTRL_VMEXIT_CONTROLS))?;
        writeln!(f, "    IA32 EFER   : {:016x}", vmcs!(GUEST_IA32_EFER))?;
        writeln!(f)?;
        Ok(())
    }

    fn dump_regs(&self, f: &mut Formatter<'_>) -> FmtResult {
        macro_rules! r {
            ($name:ident) => {
                self.read_reg(Reg::$name)
            };
        }
        writeln!(f, "  Generic Registers:")?;
        writeln!(f, "    RIP: {:016x}  RFLAGS: {:016x}", r!(RIP), r!(RFLAGS))?;
        writeln!(f, "    RAX: {:016x}     RCX: {:016x}", r!(RAX), r!(RCX))?;
        writeln!(f, "    RDX: {:016x}     RBX: {:016x}", r!(RDX), r!(RBX))?;
        writeln!(f, "    RSI: {:016x}     RDI: {:016x}", r!(RSI), r!(RDI))?;
        writeln!(f, "    RSP: {:016x}     RBP: {:016x}", r!(RSP), r!(RBP))?;
        writeln!(f, "     R8: {:016x}      R9: {:016x}", r!(R8), r!(R9))?;
        writeln!(f, "    R10: {:016x}     R11: {:016x}", r!(R10), r!(R11))?;
        writeln!(f, "    R12: {:016x}     R13: {:016x}", r!(R12), r!(R13))?;
        writeln!(f, "    R14: {:016x}     R15: {:016x}", r!(R14), r!(R15))?;
        writeln!(f)?;
        writeln!(f, "  Control Registers:")?;
        writeln!(f, "    CR0: {:016x}     CR2: {:016x}", r!(CR0), r!(CR2))?;
        writeln!(f, "    CR3: {:016x}     CR4: {:016x}", r!(CR3), r!(CR4))?;
        writeln!(f)?;
        Ok(())
    }

    fn dump_segments(&self, f: &mut Formatter<'_>) -> FmtResult {
        macro_rules! segment {
            ($name:ident) => {
                paste::paste! {
                    writeln!(
                        f,
                        "    {:-4} {:04x} {:016x} {:016x} {}",
                        stringify!($name),
                        self.read_reg(Reg::$name),
                        self.read_vmcs(Vmcs::[< GUEST_ $name _BASE >]),
                        self.read_vmcs(Vmcs::[< GUEST_ $name _LIMIT >]),
                        SegAR(self.read_vmcs(Vmcs::[< GUEST_ $name _AR >]))
                    )
                }
            };
        }
        macro_rules! seg_basic {
            ($name:ident) => {
                paste::paste! {
                    writeln!(
                        f,
                        "    {:-4}      {:016x} {:016x}",
                        stringify!($name),
                        self.read_vmcs(Vmcs::[< GUEST_ $name _BASE >]),
                        self.read_vmcs(Vmcs::[< GUEST_ $name _LIMIT >]),
                    )
                }
            };
        }
        const SEG: &str = "Seg.";
        const SEL: &str = "Sel.";
        const BASE: &str = "Base";
        const LIMIT: &str = "Limit";
        const FLAGS: &str = "U G DB L AVL P DPL S ST";
        writeln!(f, "  Segments:")?;
        writeln!(f, "    {SEG:-4} {SEL:-4} {BASE:-16} {LIMIT:-16} {FLAGS}")?;
        segment!(CS)?;
        segment!(SS)?;
        segment!(DS)?;
        segment!(ES)?;
        segment!(FS)?;
        segment!(GS)?;
        segment!(TR)?;
        segment!(LDTR)?;
        seg_basic!(GDTR)?;
        seg_basic!(IDTR)?;
        Ok(())
    }
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Debug dump of CPU {}:", self.cpu)?;
        self.dump_vmcs(f)?;
        self.dump_regs(f)?;
        self.dump_segments(f)?;
        Ok(())
    }
}

#[derive(Debug)]
struct X86_64;

#[derive(Debug)]
pub struct Vm(X86_64);

impl Vm {
    #[inline]
    pub fn new() -> Self {
        hv_call!(hv_vm_create(HV_VM_DEFAULT));
        Self(X86_64)
    }
}

impl Vm {
    #[inline]
    pub fn caps(&self, caps: Capability) -> u64 {
        let mut ret = 0u64;
        hv_call!(hv_vmx_read_capability(caps.capability(), &raw mut ret));
        ret
    }
}

impl Vm {
    #[inline]
    pub fn map(&self, base: u64, mem: &Memory, prot: Protection) {
        hv_call!(hv_vm_map(mem.base, base, mem.size, prot.bits()));
    }

    #[inline]
    pub fn protect(&self, base: u64, size: usize, prot: Protection) {
        hv_call!(hv_vm_protect(base, size, prot.bits()))
    }
}

impl Vm {
    #[inline]
    pub fn create_vcpu(&self, rip: u64) -> Cpu {
        let mut id = 0u32;
        hv_call!(hv_vcpu_create(&raw mut id, HV_VCPU_DEFAULT));
        Cpu::new(self, id, rip)
    }
}

impl Drop for Vm {
    #[inline]
    fn drop(&mut self) {
        unsafe { hv_vm_destroy() };
    }
}

impl Default for Vm {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

const ENTRY_POINT: usize = 0x1000;
const BOOTMEM_SIZE: usize = 65536;

#[inline(always)]
const fn cap2ctrl(cap: u64, ctrl: u64) -> u64 {
    (ctrl | (cap & 0xffffffff)) & (cap >> 32)
}

pub fn vm_main() -> Unit {
    let vmm = Vm::new();
    let cpu = vmm.create_vcpu(ENTRY_POINT as u64);

    let code = std::fs::read("/Users/chenzhuoyu/Sources/tests/test_hvf/init.bin")?;
    let mut mem = Memory::mmap(BOOTMEM_SIZE)?;

    mem.view_mut(ENTRY_POINT).put_slice(&code);
    vmm.map(0, &mem, Protection::all());
    cpu.write_reg(Reg::RIP, ENTRY_POINT as u64);
    cpu.run();

    eprintln!("err = {}", IoError::last_os_error());
    eprintln!(
        "VMCS_RO_EXIT_REASON = 0x{:016x}",
        cpu.read_vmcs(Vmcs::RO_EXIT_REASON)
    );
    eprintln!(
        "VMCS_RO_EXIT_QUALIFIC = 0x{:016x}",
        cpu.read_vmcs(Vmcs::RO_EXIT_QUALIFIC)
    );
    drop(vmm);
    Ok(())
}
