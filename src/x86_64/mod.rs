pub mod consts;
pub mod ffi;

use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    io::{Error as IoError, Result as IoResult, Write},
    sync::atomic::{AtomicBool, Ordering},
};

use consts::*;
use ffi::*;

use crate::{Memory, Protection, hv_call, io_error};

macro_rules! declare_friendly_enum {
    ($(pub enum $name:ident : $real_ty:ty [ $repr_ty:ty ] => $prefix:ident :: { $($item:ident),* $(,)? }),* $(,)?) => {
        paste::paste! {
            $(
                #[repr($repr_ty)]
                #[allow(non_camel_case_types)]
                #[derive(Debug, Clone, Copy)]
                pub enum $name {
                    $(
                        $item = [< $prefix $item >],
                    )*
                }

                impl $name {
                    #[allow(dead_code)]
                    #[inline(always)]
                    const fn [< $name:snake >](self) -> $real_ty {
                        self as $real_ty
                    }
                }

                impl TryFrom<$repr_ty> for $name {
                    type Error = std::io::Error;

                    fn try_from(reason: $repr_ty) -> std::io::Result<Self> {
                        match reason {
                            $(
                                [< $prefix $item >] => Ok(Self::$item),
                            )*
                            reason => Err(io_error!(
                                Other,
                                "unknown {}: {:#x}",
                                stringify!($name),
                                reason,
                            )),
                        }
                    }
                }
            )*
        }
    };
}

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
    pub enum ExitReason : u16 [ u16 ] => VMX_REASON_ :: {
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

#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct VmExit {
    pub reason: ExitReason,
    pub flags: u16,
}

impl VmExit {
    #[inline(always)]
    pub const fn is_error(&self) -> bool {
        (self.flags & VMX_FLAGS_ERROR) != 0
    }
}

impl Debug for VmExit {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("VmExit")
            .field("reason", &self.reason)
            .field_with("flags", |f| write!(f, "0x{:04x}", self.flags))
            .finish()
    }
}

impl TryFrom<u64> for VmExit {
    type Error = IoError;

    #[inline]
    fn try_from(value: u64) -> IoResult<Self> {
        Ok(Self {
            reason: ExitReason::try_from(value as u16)?,
            flags: (value >> 16) as u16,
        })
    }
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

pub struct Cpu {
    run: AtomicBool,
    cpu: hv_vcpuid_t,
}

impl Cpu {
    fn new() -> IoResult<Self> {
        let run = AtomicBool::new(true);
        let mut cpu = 0u32;
        hv_call!(hv_vcpu_create(&raw mut cpu, HV_VCPU_ACCEL_RDPMC))?;
        Ok(Self { run, cpu })
    }
}

impl Cpu {
    pub fn read_reg(&self, reg: Reg) -> IoResult<u64> {
        let mut ret = 0u64;
        hv_call!(hv_vcpu_read_register(self.cpu, reg.reg(), &raw mut ret))?;
        Ok(ret)
    }

    pub fn write_reg(&self, reg: Reg, value: u64) -> IoResult<()> {
        hv_call!(hv_vcpu_write_register(self.cpu, reg.reg(), value))
    }
}

impl Cpu {
    pub fn read_msr(&self, msr: Msr) -> IoResult<u64> {
        let mut ret = 0u64;
        hv_call!(hv_vcpu_read_msr(self.cpu, msr.msr(), &raw mut ret))?;
        Ok(ret)
    }

    pub fn write_msr(&self, msr: Msr, value: u64) -> IoResult<()> {
        hv_call!(hv_vcpu_write_msr(self.cpu, msr.msr(), value))
    }
}

impl Cpu {
    pub fn read_vmcs(&self, vmcs: Vmcs) -> IoResult<u64> {
        let mut ret = 0u64;
        hv_call!(hv_vmx_vcpu_read_vmcs(self.cpu, vmcs.vmcs(), &raw mut ret))?;
        Ok(ret)
    }

    pub fn write_vmcs(&self, vmcs: Vmcs, value: u64) -> IoResult<()> {
        hv_call!(hv_vmx_vcpu_write_vmcs(self.cpu, vmcs.vmcs(), value))
    }
}

impl Cpu {
    fn advance(&self) -> IoResult<VmExit> {
        hv_call!(hv_vcpu_run(self.cpu))?;
        self.read_vmcs(Vmcs::RO_EXIT_REASON)?.try_into()
    }
}

impl Cpu {
    #[inline(always)]
    pub fn run(&self) -> Executor<'_> {
        Executor(self)
    }
}

impl Cpu {
    fn dump_vmcs(&self, f: &mut Formatter<'_>) -> FmtResult {
        macro_rules! vmcs {
            ($name:ident) => {
                self.read_vmcs(Vmcs::$name).unwrap_or_default()
            };
        }
        writeln!(f, "  VMCS:")?;
        writeln!(f, "    CR0 mask : {:016x}", vmcs!(CTRL_CR0_MASK))?;
        writeln!(f, "      shadow : {:016x}", vmcs!(CTRL_CR0_SHADOW))?;
        writeln!(f, "    CR4 mask : {:016x}", vmcs!(CTRL_CR4_MASK))?;
        writeln!(f, "      shadow : {:016x}", vmcs!(CTRL_CR4_SHADOW))?;
        writeln!(f, "    PinBased : {:016x}", vmcs!(CTRL_PIN_BASED))?;
        writeln!(f, "         1st : {:016x}", vmcs!(CTRL_CPU_BASED))?;
        writeln!(f, "         2st : {:016x}", vmcs!(CTRL_CPU_BASED2))?;
        writeln!(f, "    VM Entry : {:016x}", vmcs!(CTRL_VMENTRY_CONTROLS))?;
        writeln!(f, "    VM Exit  : {:016x}", vmcs!(CTRL_VMEXIT_CONTROLS))?;
        writeln!(f, "    EFER     : {:016x}", vmcs!(GUEST_IA32_EFER))?;
        writeln!(f)?;
        Ok(())
    }

    fn dump_regs(&self, f: &mut Formatter<'_>) -> FmtResult {
        macro_rules! r {
            ($name:ident) => {
                self.read_reg(Reg::$name).unwrap_or_default()
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
                        self.read_reg(Reg::$name).unwrap_or_default(),
                        self.read_vmcs(Vmcs::[< GUEST_ $name _BASE >]).unwrap_or_default(),
                        self.read_vmcs(Vmcs::[< GUEST_ $name _LIMIT >]).unwrap_or_default(),
                        SegAR(self.read_vmcs(Vmcs::[< GUEST_ $name _AR >]).unwrap_or_default())
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
                        self.read_vmcs(Vmcs::[< GUEST_ $name _BASE >]).unwrap_or_default(),
                        self.read_vmcs(Vmcs::[< GUEST_ $name _LIMIT >]).unwrap_or_default(),
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
        writeln!(f)?;
        Ok(())
    }

    fn dump_vmcs_link(&self, f: &mut Formatter<'_>) -> FmtResult {
        let link = self.read_vmcs(Vmcs::GUEST_LINK_POINTER).unwrap_or_default();
        writeln!(f, "  VMCS Link Pointer:")?;
        writeln!(f, "    {link:016x}")?;
        Ok(())
    }
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Debug dump of CPU {}:", self.cpu)?;
        self.dump_vmcs(f)?;
        self.dump_regs(f)?;
        self.dump_segments(f)?;
        self.dump_vmcs_link(f)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Executor<'p>(&'p Cpu);

impl Iterator for Executor<'_> {
    type Item = IoResult<VmExit>;

    #[inline]
    fn next(&mut self) -> Option<IoResult<VmExit>> {
        if self.0.run.load(Ordering::Relaxed) {
            Some(self.0.advance())
        } else {
            None
        }
    }
}

struct X86_64;
pub struct Vm(X86_64);

impl Vm {
    pub fn new() -> IoResult<Self> {
        hv_call!(hv_vm_create(HV_VM_DEFAULT))?;
        Ok(Self(X86_64))
    }
}

impl Vm {
    pub fn map(&mut self, base: u64, mem: &Memory) -> IoResult<()> {
        hv_call!(hv_vm_map(mem.base, base, mem.size, mem.prot.bits()))?;
        Ok(())
    }
}

impl Vm {
    pub fn caps(&self, caps: Capability) -> IoResult<u64> {
        let mut ret = 0u64;
        hv_call!(hv_vmx_read_capability(caps.capability(), &raw mut ret))?;
        Ok(ret)
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        if let Err(err) = hv_call!(hv_vm_destroy()) {
            tracing::error!("Cannot destroy vm: {err}")
        }
    }
}

pub fn vm_main() -> IoResult<()> {
    let mut vm = Vm::new()?;
    let mut mem = Memory::mmap(65536, Protection::all())?;
    vm.map(0, &mem)?;

    let pinbased = vm.caps(Capability::PINBASED)?;
    let procbased = vm.caps(Capability::PROCBASED)?;
    let procbased2 = vm.caps(Capability::PROCBASED2)?;
    let entry = vm.caps(Capability::ENTRY)?;
    eprintln!(
        "capabilities: pinbased={pinbased:016x} procbased={procbased:016x} \
         procbased2={procbased2:016x} entry={entry:016x}"
    );

    fn cap2ctrl(cap: u64, ctrl: u64) -> u64 {
        (ctrl | (cap & 0xffffffff)) & (cap >> 32)
    }

    let cpu = Cpu::new()?;
    cpu.write_vmcs(Vmcs::CTRL_PIN_BASED, cap2ctrl(pinbased, 0))?;
    cpu.write_vmcs(
        Vmcs::CTRL_CPU_BASED,
        cap2ctrl(
            procbased,
            CPU_BASED_HLT | CPU_BASED_CR8_LOAD | CPU_BASED_CR8_STORE,
        ),
    )?;
    cpu.write_vmcs(Vmcs::CTRL_CPU_BASED2, cap2ctrl(procbased2, 0))?;
    cpu.write_vmcs(Vmcs::CTRL_VMENTRY_CONTROLS, cap2ctrl(entry, 0))?;

    cpu.write_vmcs(Vmcs::CTRL_EXC_BITMAP, 0xffffffff)?;
    cpu.write_vmcs(Vmcs::CTRL_CR0_MASK, 0x60000000)?;
    cpu.write_vmcs(Vmcs::CTRL_CR0_SHADOW, 0)?;
    cpu.write_vmcs(Vmcs::CTRL_CR4_MASK, 0)?;
    cpu.write_vmcs(Vmcs::CTRL_CR4_SHADOW, 0)?;

    cpu.write_vmcs(Vmcs::GUEST_CS, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_CS_BASE, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_CS_LIMIT, 0xffff)?;
    cpu.write_vmcs(Vmcs::GUEST_CS_AR, 0x9b)?;

    cpu.write_vmcs(Vmcs::GUEST_DS, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_DS_LIMIT, 0xffff)?;
    cpu.write_vmcs(Vmcs::GUEST_DS_AR, 0x93)?;
    cpu.write_vmcs(Vmcs::GUEST_DS_BASE, 0)?;

    cpu.write_vmcs(Vmcs::GUEST_ES, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_ES_LIMIT, 0xffff)?;
    cpu.write_vmcs(Vmcs::GUEST_ES_AR, 0x93)?;
    cpu.write_vmcs(Vmcs::GUEST_ES_BASE, 0)?;

    cpu.write_vmcs(Vmcs::GUEST_FS, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_FS_LIMIT, 0xffff)?;
    cpu.write_vmcs(Vmcs::GUEST_FS_AR, 0x93)?;
    cpu.write_vmcs(Vmcs::GUEST_FS_BASE, 0)?;

    cpu.write_vmcs(Vmcs::GUEST_GS, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_GS_LIMIT, 0xffff)?;
    cpu.write_vmcs(Vmcs::GUEST_GS_AR, 0x93)?;
    cpu.write_vmcs(Vmcs::GUEST_GS_BASE, 0)?;

    cpu.write_vmcs(Vmcs::GUEST_SS, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_SS_LIMIT, 0xffff)?;
    cpu.write_vmcs(Vmcs::GUEST_SS_AR, 0x93)?;
    cpu.write_vmcs(Vmcs::GUEST_SS_BASE, 0)?;

    cpu.write_vmcs(Vmcs::GUEST_LDTR, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_LDTR_LIMIT, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_LDTR_AR, 0x10000)?;
    cpu.write_vmcs(Vmcs::GUEST_LDTR_BASE, 0)?;

    cpu.write_vmcs(Vmcs::GUEST_TR, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_TR_LIMIT, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_TR_AR, 0x83)?;
    cpu.write_vmcs(Vmcs::GUEST_TR_BASE, 0)?;

    cpu.write_vmcs(Vmcs::GUEST_GDTR_LIMIT, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_GDTR_BASE, 0)?;

    cpu.write_vmcs(Vmcs::GUEST_IDTR_LIMIT, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_IDTR_BASE, 0)?;

    cpu.write_vmcs(Vmcs::GUEST_CR0, 0x20)?;
    cpu.write_vmcs(Vmcs::GUEST_CR3, 0x0)?;
    cpu.write_vmcs(Vmcs::GUEST_CR4, 0x2000)?;

    let code = [
        0xba, 0xf8, 0x03, // mov dx, $0x3f8
        0x00, 0xd8, // add al, bl
        0x04, b'0', // add al, '0'
        0xee, // out [dx], al
        0xb0, b'\n', // mov al, '\n'
        0xee,  // out [dx], al
        0x90, 0x90, 0x90, 0x90, // nop*4
        0x90, 0x90, 0x90, 0x90, // nop*4
        0x90, 0x90, 0x90, 0x90, // nop*4
        0x90, 0x90, 0x90, // nop*3
        0xf4, // hlt
    ];
    mem.write(0x100, &code);
    cpu.write_reg(Reg::RIP, 0x100)?;
    cpu.write_reg(Reg::RFLAGS, 0x2)?;
    cpu.write_reg(Reg::RSP, 0x0)?;
    cpu.write_reg(Reg::RAX, 0x5)?;
    cpu.write_reg(Reg::RBX, 0x3)?;

    for event in cpu.run() {
        dbg!(&cpu);
        eprintln!("event = {event:?}");
        let Ok(event) = event else {
            break;
        };
        if event.is_error() {
            eprintln!("VM Enter error");
            break;
        }
        std::io::stderr()
            .write_all(b"* Press ENTER to continue ...")
            .unwrap();
        std::io::stderr().flush().unwrap();
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }

    drop(mem);
    drop(vm);
    Ok(())
}
