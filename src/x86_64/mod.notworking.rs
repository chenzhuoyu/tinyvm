pub mod consts;
pub mod ffi;

use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    io::{Error as IoError, Result as IoResult, Write},
    sync::atomic::{AtomicBool, Ordering},
};

use bytes::BufMut;
use consts::*;
use ffi::*;

use crate::{Memory, MemoryViewMut, Protection, hv_call, io_error};

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

const PAGE_P: u64 = 1 << 0;
const PAGE_RW: u64 = 1 << 1;
const PAGE_PS: u64 = 1 << 7;

const CR0_INIT: u64 = 0x80010033; // PG WP NE ET MP PE
const CR4_INIT: u64 = 0x00000000; // PAE
const EFER_INIT: u64 = 0x00000000; // LME
const RFLAGS_INIT: u64 = 0x00000002;

const CS_AR: u64 = 0xc09b; // G=1 DB=1 L=0 P=1 DPL=00 S=1 E=1 DC=0 RW=1 A=1
const DS_AR: u64 = 0xc093; // G=1 DB=1 L=0 P=1 DPL=00 S=1 E=0 DC=0 RW=1 A=1
const TSS_AR: u64 = 0x0089; // G=0 DB=0 L=0 P=1 DPL=00 S=0 Type=9 (TSS)
const UNUSED_AR: u64 = 0x10000; // Unused

const CS_SEL: u64 = 0x08;
const DS_SEL: u64 = 0x10;
const TSS_SEL: u64 = 0x18;

const GDT_NULL: u64 = 0;
const GDT_CODE: u64 = CS_AR << 40;
const GDT_DATA: u64 = DS_AR << 40;
const GDT_TSSL: u64 = mksgdt(TSS_BASE, TSS_SIZE - 1, TSS_AR);
const GDT_TSSH: u64 = (TSS_BASE as u64) >> 32;

const TSS_BASE: usize = 0x0f00;
const TSS_SIZE: usize = 0x0068;
const GDT_BASE: usize = 0x0fd0;
const GDT_SIZE: usize = 0x0028;

const PAGE_SIZE: usize = 0x1000;
const PML4_BASE: usize = 0x1000;
const PDPT_BASE: usize = 0x2000;
const SMEM_SIZE: usize = (PAGE_SIZE * 2 + GDT_SIZE).div_ceil(PAGE_SIZE) * PAGE_SIZE;

const IMAGE_BASE: usize = 0x0010_0000;
const IMAGE_SIZE: usize = 0x0010_0000;
const STACK_BASE: usize = 0x0001_0000;
const STACK_SIZE: usize = 0x000f_0000;
const ENTRY_ADDR: usize = IMAGE_BASE;

#[inline(always)]
const fn mksgdt(base: usize, limit: usize, access: u64) -> u64 {
    ((((base as u64) >> 24) & 0xff) << 56)
        | ((((limit as u64) >> 16) & 0x0f) << 48)
        | (access << 40)
        | (((base as u64) & 0xffffff) << 16)
        | ((limit as u64) & 0xffff)
}

#[inline(always)]
const fn cap2ctrl(cap: u64, ctrl: u64) -> u64 {
    (ctrl | (cap & 0xffffffff)) & (cap >> 32)
}

#[inline(always)]
fn setup_gdt(mut mem: MemoryViewMut<'_>) {
    mem.put_u64_le(GDT_NULL);
    mem.put_u64_le(GDT_CODE);
    mem.put_u64_le(GDT_DATA);
    mem.put_u64_le(GDT_TSSL);
    mem.put_u64_le(GDT_TSSH);
}

#[inline(always)]
fn setup_pml4(mut mem: MemoryViewMut<'_>) {
    mem.put_u64_le((PDPT_BASE as u64) | PAGE_RW | PAGE_P);
    mem.put_bytes(0, PAGE_SIZE - 8);
}

#[inline(always)]
fn setup_pdpt(mut mem: MemoryViewMut<'_>) {
    mem.put_u64_le(PAGE_PS | PAGE_RW | PAGE_P);
    mem.put_bytes(0, PAGE_SIZE - 8);
}

pub fn vm_main() -> IoResult<()> {
    let mut vm = Vm::new()?;
    let mut cfg = Memory::mmap(SMEM_SIZE, Protection::RW)?;
    let mut code = Memory::mmap(IMAGE_SIZE, Protection::RW)?;

    // TODO: load code
    code[0] = 0xf4; // HLT
    code.protect(Protection::RX)?;

    /* setup GDT & initial page table with 1G page */
    setup_gdt(cfg.view_mut(GDT_BASE));
    setup_pml4(cfg.view_mut(PML4_BASE));
    setup_pdpt(cfg.view_mut(PDPT_BASE));

    /* create virtual CPU */
    let cpu = Cpu::new()?;
    let stack = Memory::mmap(STACK_SIZE, Protection::RW)?;

    /* map memory regions */
    vm.map(0, &cfg)?;
    vm.map(IMAGE_BASE as u64, &code)?;
    vm.map(STACK_BASE as u64, &stack)?;

    /* setup entry point */
    cpu.write_vmcs(Vmcs::GUEST_RIP, ENTRY_ADDR as u64)?;
    cpu.write_vmcs(Vmcs::GUEST_RSP, (STACK_BASE + STACK_SIZE - 16) as u64)?;
    cpu.write_vmcs(Vmcs::GUEST_RFLAGS, RFLAGS_INIT)?;

    /* setup control registers */
    cpu.write_vmcs(Vmcs::GUEST_CR0, CR0_INIT)?;
    cpu.write_vmcs(Vmcs::GUEST_CR3, PML4_BASE as u64)?;
    cpu.write_vmcs(Vmcs::GUEST_CR4, CR4_INIT)?;
    cpu.write_vmcs(Vmcs::GUEST_IA32_EFER, EFER_INIT)?;
    cpu.write_vmcs(Vmcs::GUEST_DR7, 0x400)?;
    cpu.write_vmcs(Vmcs::GUEST_IA32_DEBUGCTL, 0)?;

    /* setup code segment */
    cpu.write_vmcs(Vmcs::GUEST_CS, CS_SEL)?;
    cpu.write_vmcs(Vmcs::GUEST_CS_AR, CS_AR)?;
    cpu.write_vmcs(Vmcs::GUEST_CS_BASE, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_CS_LIMIT, 0xffffffff)?;

    /* setup data segments */
    cpu.write_vmcs(Vmcs::GUEST_SS, DS_SEL)?;
    cpu.write_vmcs(Vmcs::GUEST_DS, DS_SEL)?;
    cpu.write_vmcs(Vmcs::GUEST_ES, DS_SEL)?;
    cpu.write_vmcs(Vmcs::GUEST_SS_AR, DS_AR)?;
    cpu.write_vmcs(Vmcs::GUEST_DS_AR, DS_AR)?;
    cpu.write_vmcs(Vmcs::GUEST_ES_AR, DS_AR)?;
    cpu.write_vmcs(Vmcs::GUEST_SS_BASE, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_DS_BASE, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_ES_BASE, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_SS_LIMIT, 0xffffffff)?;
    cpu.write_vmcs(Vmcs::GUEST_DS_LIMIT, 0xffffffff)?;
    cpu.write_vmcs(Vmcs::GUEST_ES_LIMIT, 0xffffffff)?;

    /* FS & GS */
    cpu.write_vmcs(Vmcs::GUEST_FS, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_GS, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_FS_AR, UNUSED_AR)?;
    cpu.write_vmcs(Vmcs::GUEST_GS_AR, UNUSED_AR)?;
    cpu.write_vmcs(Vmcs::GUEST_FS_BASE, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_GS_BASE, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_FS_LIMIT, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_GS_LIMIT, 0)?;

    /* GDTR & IDTR */
    cpu.write_vmcs(Vmcs::GUEST_GDTR_BASE, GDT_BASE as u64)?;
    cpu.write_vmcs(Vmcs::GUEST_IDTR_BASE, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_GDTR_LIMIT, (GDT_SIZE - 1) as u64)?;
    cpu.write_vmcs(Vmcs::GUEST_IDTR_LIMIT, 0)?;

    /* LDTR */
    cpu.write_vmcs(Vmcs::GUEST_LDTR, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_LDTR_AR, UNUSED_AR)?;
    cpu.write_vmcs(Vmcs::GUEST_LDTR_BASE, 0)?;
    cpu.write_vmcs(Vmcs::GUEST_LDTR_LIMIT, 0)?;

    /* TR (TSS) */
    cpu.write_vmcs(Vmcs::GUEST_TR, TSS_SEL)?;
    cpu.write_vmcs(Vmcs::GUEST_TR_AR, TSS_AR)?;
    cpu.write_vmcs(Vmcs::GUEST_TR_BASE, TSS_BASE as u64)?;
    cpu.write_vmcs(Vmcs::GUEST_TR_LIMIT, (TSS_SIZE - 1) as u64)?;

    /* Pin-base Control */
    cpu.write_vmcs(
        Vmcs::CTRL_PIN_BASED,
        cap2ctrl(vm.caps(Capability::PINBASED)?, 0),
    )?;

    /* CPU-base Control */
    cpu.write_vmcs(
        Vmcs::CTRL_CPU_BASED,
        cap2ctrl(vm.caps(Capability::PROCBASED)?, 0),
    )?;

    /* CPU-base Control 2 */
    cpu.write_vmcs(
        Vmcs::CTRL_CPU_BASED2,
        cap2ctrl(vm.caps(Capability::PROCBASED2)?, 0),
    )?;

    /* VM Entry Contorl */
    cpu.write_vmcs(
        Vmcs::CTRL_VMENTRY_CONTROLS,
        cap2ctrl(vm.caps(Capability::ENTRY)?, VMENTRY_LOAD_EFER),
    )?;

    /* initialize register state */
    // cpu.write_reg(Reg::RSI, 0)?;
    // cpu.write_reg(Reg::XCR0, 0x07)?;

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

    eprintln!("err = {}", IoError::last_os_error());
    eprintln!(
        "VMCS_RO_EXIT_REASON = 0x{:016x}",
        cpu.read_vmcs(Vmcs::RO_EXIT_REASON)?
    );
    eprintln!(
        "VMCS_RO_EXIT_QUALIFIC = 0x{:016x}",
        cpu.read_vmcs(Vmcs::RO_EXIT_QUALIFIC)?
    );
    drop(vm);
    Ok(())
}
