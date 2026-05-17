/* This file is modified from the bindgen generated one */

#![allow(non_camel_case_types)]

pub const HV_APIC_STATE_EXT_VER: u64 = 100;
pub const HV_ATPIC_STATE_EXT_VER: u64 = 100;
pub const HV_IOAPIC_STATE_EXT_VER: u64 = 100;

pub type hv_x86_reg_t = u32;
pub const HV_X86_RIP: hv_x86_reg_t = 0;
pub const HV_X86_RFLAGS: hv_x86_reg_t = 1;
pub const HV_X86_RAX: hv_x86_reg_t = 2;
pub const HV_X86_RCX: hv_x86_reg_t = 3;
pub const HV_X86_RDX: hv_x86_reg_t = 4;
pub const HV_X86_RBX: hv_x86_reg_t = 5;
pub const HV_X86_RSI: hv_x86_reg_t = 6;
pub const HV_X86_RDI: hv_x86_reg_t = 7;
pub const HV_X86_RSP: hv_x86_reg_t = 8;
pub const HV_X86_RBP: hv_x86_reg_t = 9;
pub const HV_X86_R8: hv_x86_reg_t = 10;
pub const HV_X86_R9: hv_x86_reg_t = 11;
pub const HV_X86_R10: hv_x86_reg_t = 12;
pub const HV_X86_R11: hv_x86_reg_t = 13;
pub const HV_X86_R12: hv_x86_reg_t = 14;
pub const HV_X86_R13: hv_x86_reg_t = 15;
pub const HV_X86_R14: hv_x86_reg_t = 16;
pub const HV_X86_R15: hv_x86_reg_t = 17;
pub const HV_X86_CS: hv_x86_reg_t = 18;
pub const HV_X86_SS: hv_x86_reg_t = 19;
pub const HV_X86_DS: hv_x86_reg_t = 20;
pub const HV_X86_ES: hv_x86_reg_t = 21;
pub const HV_X86_FS: hv_x86_reg_t = 22;
pub const HV_X86_GS: hv_x86_reg_t = 23;
pub const HV_X86_IDT_BASE: hv_x86_reg_t = 24;
pub const HV_X86_IDT_LIMIT: hv_x86_reg_t = 25;
pub const HV_X86_GDT_BASE: hv_x86_reg_t = 26;
pub const HV_X86_GDT_LIMIT: hv_x86_reg_t = 27;
pub const HV_X86_LDTR: hv_x86_reg_t = 28;
pub const HV_X86_LDT_BASE: hv_x86_reg_t = 29;
pub const HV_X86_LDT_LIMIT: hv_x86_reg_t = 30;
pub const HV_X86_LDT_AR: hv_x86_reg_t = 31;
pub const HV_X86_TR: hv_x86_reg_t = 32;
pub const HV_X86_TSS_BASE: hv_x86_reg_t = 33;
pub const HV_X86_TSS_LIMIT: hv_x86_reg_t = 34;
pub const HV_X86_TSS_AR: hv_x86_reg_t = 35;
pub const HV_X86_CR0: hv_x86_reg_t = 36;
pub const HV_X86_CR1: hv_x86_reg_t = 37;
pub const HV_X86_CR2: hv_x86_reg_t = 38;
pub const HV_X86_CR3: hv_x86_reg_t = 39;
pub const HV_X86_CR4: hv_x86_reg_t = 40;
pub const HV_X86_DR0: hv_x86_reg_t = 41;
pub const HV_X86_DR1: hv_x86_reg_t = 42;
pub const HV_X86_DR2: hv_x86_reg_t = 43;
pub const HV_X86_DR3: hv_x86_reg_t = 44;
pub const HV_X86_DR4: hv_x86_reg_t = 45;
pub const HV_X86_DR5: hv_x86_reg_t = 46;
pub const HV_X86_DR6: hv_x86_reg_t = 47;
pub const HV_X86_DR7: hv_x86_reg_t = 48;
pub const HV_X86_TPR: hv_x86_reg_t = 49;
pub const HV_X86_XCR0: hv_x86_reg_t = 50;
pub const HV_X86_REGISTERS_MAX: hv_x86_reg_t = 51;

pub type hv_return_t = u32;
pub const HV_SUCCESS: hv_return_t = 0;
pub const HV_ERROR: hv_return_t = 0xfae94001;
pub const HV_BUSY: hv_return_t = 0xfae94002;
pub const HV_BAD_ARGUMENT: hv_return_t = 0xfae94003;
pub const HV_NO_RESOURCES: hv_return_t = 0xfae94005;
pub const HV_NO_DEVICE: hv_return_t = 0xfae94006;
pub const HV_DENIED: hv_return_t = 0xfae94007;
pub const HV_FAULT: hv_return_t = 0xfae94008;
pub const HV_UNSUPPORTED: hv_return_t = 0xfae9400f;

pub type hv_boot_state = u32;
pub const HV_BS_INIT: hv_boot_state = 0;
pub const HV_BS_SIPI: hv_boot_state = 1;
pub const HV_BS_RUNNING: hv_boot_state = 2;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hv_apic_state {
    pub apic_gpa: u64,
    pub apic_controls: u64,
    pub tsc_deadline: u64,
    pub apic_id: u32,
    pub ver: u32,
    pub tpr: u32,
    pub apr: u32,
    pub ldr: u32,
    pub dfr: u32,
    pub svr: u32,
    pub isr: [u32; 8usize],
    pub tmr: [u32; 8usize],
    pub irr: [u32; 8usize],
    pub esr: u32,
    pub lvt: [u32; 7usize],
    pub icr: [u32; 2usize],
    pub icr_timer: u32,
    pub dcr_timer: u32,
    pub ccr_timer: u32,
    pub esr_pending: u32,
    pub boot_state: hv_boot_state,
    pub aeoi: [u32; 8usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hv_apic_state_ext_t {
    pub version: u32,
    pub state: hv_apic_state,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hv_atpic_state {
    pub ready: bool,
    pub icw_num: u8,
    pub rd_cmd_reg: u8,
    pub aeoi: bool,
    pub poll: bool,
    pub rotate: bool,
    pub sfn: bool,
    pub irq_base: u8,
    pub request: u8,
    pub service: u8,
    pub mask: u8,
    pub smm: bool,
    pub last_request: u8,
    pub lowprio: u8,
    pub intr_raised: bool,
    pub elc: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hv_atpic_state_ext_t {
    pub version: u32,
    pub state: hv_atpic_state,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hv_ioapic_state {
    pub rtbl: [u64; 32usize],
    pub irr: u32,
    pub ioa_id: u32,
    pub ioregsel: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hv_ioapic_state_ext_t {
    pub version: u32,
    pub state: hv_ioapic_state,
}

pub type mach_port_name_t = u32;
pub type mach_port_t = u32;
pub type mach_msg_bits_t = u32;
pub type mach_msg_size_t = u32;
pub type mach_msg_id_t = i32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mach_msg_header_t {
    pub msgh_bits: mach_msg_bits_t,
    pub msgh_size: mach_msg_size_t,
    pub msgh_remote_port: mach_port_t,
    pub msgh_local_port: mach_port_t,
    pub msgh_voucher_port: mach_port_name_t,
    pub msgh_id: mach_msg_id_t,
}

pub type mach_msg_trailer_type_t = u32;
pub type mach_msg_trailer_size_t = u32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mach_msg_trailer_t {
    pub msgh_trailer_type: mach_msg_trailer_type_t,
    pub msgh_trailer_size: mach_msg_trailer_size_t,
}

pub type hv_capability_t = u64;
pub const HV_CAP_VCPUMAX: hv_capability_t = 0;
pub const HV_CAP_ADDRSPACEMAX: hv_capability_t = 1;

pub type hv_vm_space_t = u32;
pub const HV_VM_SPACE_DEFAULT: hv_vm_space_t = 0;

pub type hv_vm_options_t = u64;
pub const HV_VM_DEFAULT: hv_vm_options_t = 0;
pub const HV_VM_SPECIFY_MITIGATIONS: hv_vm_options_t = 1;
pub const HV_VM_MITIGATION_A_ENABLE: hv_vm_options_t = 2;
pub const HV_VM_MITIGATION_B_ENABLE: hv_vm_options_t = 4;
pub const HV_VM_MITIGATION_C_ENABLE: hv_vm_options_t = 8;
pub const HV_VM_MITIGATION_D_ENABLE: hv_vm_options_t = 16;
pub const HV_VM_MITIGATION_E_ENABLE: hv_vm_options_t = 64;
pub const HV_VM_ACCEL_APIC: hv_vm_options_t = 1024;

pub type hv_vcpu_options_t = u64;
pub const HV_VCPU_DEFAULT: hv_vcpu_options_t = 0;
pub const HV_VCPU_ACCEL_RDPMC: hv_vcpu_options_t = 1;
pub const HV_VCPU_TSC_RELATIVE: hv_vcpu_options_t = 2;

pub type hv_memory_flags_t = u64;
pub const HV_MEMORY_READ: hv_memory_flags_t = 1;
pub const HV_MEMORY_WRITE: hv_memory_flags_t = 2;
pub const HV_MEMORY_EXEC: hv_memory_flags_t = 4;
pub const HV_MEMORY_UEXEC: hv_memory_flags_t = 8;
pub const HV_MEMORY_MAXPROT: hv_memory_flags_t = 16;
pub const HV_MEMORY_MAXPROT_READ: hv_memory_flags_t = 32;
pub const HV_MEMORY_MAXPROT_WRITE: hv_memory_flags_t = 64;
pub const HV_MEMORY_MAXPROT_EXEC: hv_memory_flags_t = 128;
pub const HV_MEMORY_MAXPROT_UEXEC: hv_memory_flags_t = 256;

pub type hv_msr_flags_t = u32;
pub const HV_MSR_NONE: hv_msr_flags_t = 0;
pub const HV_MSR_READ: hv_msr_flags_t = 1;
pub const HV_MSR_WRITE: hv_msr_flags_t = 2;

pub type hv_ion_flags_t = u32;
pub const HV_ION_NONE: hv_ion_flags_t = 0;
pub const HV_ION_ANY_VALUE: hv_ion_flags_t = 2;
pub const HV_ION_ANY_SIZE: hv_ion_flags_t = 4;
pub const HV_ION_EXIT_FULL: hv_ion_flags_t = 8;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hv_ion_message_t {
    pub header: mach_msg_header_t,
    pub addr: u64,
    pub size: u64,
    pub value: u64,
    pub trailer: mach_msg_trailer_t,
}

pub type hv_vcpuid_t = u32;
pub type hv_uvaddr_t = *const libc::c_void;
pub type hv_gpaddr_t = u64;

pub type hv_vm_exitinfo_t = u32;
pub const HV_VM_EXITINFO_VMX: hv_vm_exitinfo_t = 1;
pub const HV_VM_EXITINFO_INIT_AP: hv_vm_exitinfo_t = 2;
pub const HV_VM_EXITINFO_STARTUP_AP: hv_vm_exitinfo_t = 3;
pub const HV_VM_EXITINFO_IOAPIC_EOI: hv_vm_exitinfo_t = 4;
pub const HV_VM_EXITINFO_INJECT_EXCP: hv_vm_exitinfo_t = 5;
pub const HV_VM_EXITINFO_SMI: hv_vm_exitinfo_t = 6;
pub const HV_VM_EXITINFO_APIC_ACCESS_READ: hv_vm_exitinfo_t = 7;

pub type hv_apic_ctrl_t = u64;
pub const HV_APIC_CTRL_DEFAULT: hv_apic_ctrl_t = 0;
pub const HV_APIC_CTRL_EOI_ICR_TPR: hv_apic_ctrl_t = 1;
pub const HV_APIC_CTRL_GUEST_IDLE: hv_apic_ctrl_t = 2;
pub const HV_APIC_CTRL_NO_TIMER: hv_apic_ctrl_t = 4;
pub const HV_APIC_CTRL_IOAPIC_EOI: hv_apic_ctrl_t = 8;

pub type hv_apic_lvt_flavor_t = u32;
pub const HV_APIC_LVT_FLAVOR_TIMER: hv_apic_lvt_flavor_t = 1;

pub type hv_apic_intr_trigger_t = u32;
pub const HV_APIC_EDGE_TRIGGER: hv_apic_intr_trigger_t = 0;
pub const HV_APIC_EDGE_TRIGGER_AEOI: hv_apic_intr_trigger_t = 1;
pub const HV_APIC_LEVEL_TRIGGER: hv_apic_intr_trigger_t = 2;

pub type hv_allocate_flags_t = u64;
pub const HV_ALLOCATE_DEFAULT: hv_allocate_flags_t = 0;

unsafe extern "C" {
    pub fn hv_vm_allocate(
        uvap: *mut *mut libc::c_void,
        size: usize,
        flags: hv_allocate_flags_t,
    ) -> hv_return_t;
    pub fn hv_vm_deallocate(uva: *mut libc::c_void, size: usize) -> hv_return_t;
    pub fn hv_capability(capability: hv_capability_t, value: *mut u64) -> hv_return_t;
    pub fn hv_vm_create(flags: hv_vm_options_t) -> hv_return_t;
    pub fn hv_vm_destroy() -> hv_return_t;
    pub fn hv_vm_space_create(asid: *mut hv_vm_space_t) -> hv_return_t;
    pub fn hv_vm_space_destroy(asid: hv_vm_space_t) -> hv_return_t;
    pub fn hv_vm_map(
        uva: hv_uvaddr_t,
        gpa: hv_gpaddr_t,
        size: usize,
        flags: hv_memory_flags_t,
    ) -> hv_return_t;
    pub fn hv_vm_unmap(gpa: hv_gpaddr_t, size: usize) -> hv_return_t;
    pub fn hv_vm_protect(gpa: hv_gpaddr_t, size: usize, flags: hv_memory_flags_t) -> hv_return_t;
    pub fn hv_vm_map_space(
        asid: hv_vm_space_t,
        uva: hv_uvaddr_t,
        gpa: hv_gpaddr_t,
        size: usize,
        flags: hv_memory_flags_t,
    ) -> hv_return_t;
    pub fn hv_vm_unmap_space(asid: hv_vm_space_t, gpa: hv_gpaddr_t, size: usize) -> hv_return_t;
    pub fn hv_vm_protect_space(
        asid: hv_vm_space_t,
        gpa: hv_gpaddr_t,
        size: usize,
        flags: hv_memory_flags_t,
    ) -> hv_return_t;
    pub fn hv_vm_sync_tsc(tsc: u64) -> hv_return_t;
    pub fn hv_vm_add_pio_notifier(
        addr: u16,
        size: usize,
        value: u32,
        mach_port: mach_port_t,
        flags: hv_ion_flags_t,
    ) -> hv_return_t;
    pub fn hv_vm_remove_pio_notifier(
        addr: u16,
        size: usize,
        value: u32,
        mach_port: mach_port_t,
        flags: hv_ion_flags_t,
    ) -> hv_return_t;
    pub fn hv_vcpu_create(vcpu: *mut hv_vcpuid_t, flags: hv_vcpu_options_t) -> hv_return_t;
    pub fn hv_vcpu_destroy(vcpu: hv_vcpuid_t) -> hv_return_t;
    pub fn hv_vcpu_set_space(vcpu: hv_vcpuid_t, asid: hv_vm_space_t) -> hv_return_t;
    pub fn hv_vcpu_read_register(
        vcpu: hv_vcpuid_t,
        reg: hv_x86_reg_t,
        value: *mut u64,
    ) -> hv_return_t;
    pub fn hv_vcpu_write_register(vcpu: hv_vcpuid_t, reg: hv_x86_reg_t, value: u64) -> hv_return_t;
    pub fn hv_vcpu_read_fpstate(
        vcpu: hv_vcpuid_t,
        buffer: *mut libc::c_void,
        size: usize,
    ) -> hv_return_t;
    pub fn hv_vcpu_write_fpstate(
        vcpu: hv_vcpuid_t,
        buffer: *mut libc::c_void,
        size: usize,
    ) -> hv_return_t;
    pub fn hv_vcpu_enable_native_msr(vcpu: hv_vcpuid_t, msr: u32, enable: bool) -> hv_return_t;
    pub fn hv_vcpu_enable_managed_msr(vcpu: hv_vcpuid_t, msr: u32, enable: bool) -> hv_return_t;
    pub fn hv_vcpu_set_msr_access(
        vcpu: hv_vcpuid_t,
        msr: u32,
        flags: hv_msr_flags_t,
    ) -> hv_return_t;
    pub fn hv_vcpu_read_msr(vcpu: hv_vcpuid_t, msr: u32, value: *mut u64) -> hv_return_t;
    pub fn hv_vcpu_write_msr(vcpu: hv_vcpuid_t, msr: u32, value: u64) -> hv_return_t;
    pub fn hv_vcpu_flush(vcpu: hv_vcpuid_t) -> hv_return_t;
    pub fn hv_vcpu_invalidate_tlb(vcpu: hv_vcpuid_t) -> hv_return_t;
}

pub const HV_DEADLINE_FOREVER: u64 = u64::MAX;

unsafe extern "C" {
    pub fn hv_vcpu_run(vcpu: hv_vcpuid_t) -> hv_return_t;
    pub fn hv_vcpu_run_until(vcpu: hv_vcpuid_t, deadline: u64) -> hv_return_t;
}

unsafe extern "C" {
    pub fn hv_vcpu_interrupt(vcpus: *mut hv_vcpuid_t, vcpu_count: u32) -> hv_return_t;
    pub fn hv_vcpu_get_exec_time(vcpu: hv_vcpuid_t, time: *mut u64) -> hv_return_t;
    pub fn hv_vcpu_get_idle_time(vcpu: hv_vcpuid_t, time: *mut u64) -> hv_return_t;
    pub fn hv_tsc_clock() -> u64;
    pub fn hv_vcpu_set_tsc_relative(vcpu: hv_vcpuid_t, offset: i64) -> hv_return_t;
    pub fn hv_vcpu_vmx_status(vcpu: hv_vcpuid_t, status: *mut u32) -> hv_return_t;
    pub fn hv_vm_lapic_set_intr(
        vcpu: hv_vcpuid_t,
        vector: u8,
        trig: hv_apic_intr_trigger_t,
    ) -> hv_return_t;
    pub fn hv_vm_lapic_msi(addr: u64, data: u64) -> hv_return_t;
    pub fn hv_vm_ioapic_assert_irq(intin: i32) -> hv_return_t;
    pub fn hv_vm_ioapic_deassert_irq(intin: i32) -> hv_return_t;
    pub fn hv_vm_ioapic_pulse_irq(intin: i32) -> hv_return_t;
    pub fn hv_vm_ioapic_read(gpa: hv_gpaddr_t, datap: *mut u32) -> hv_return_t;
    pub fn hv_vm_ioapic_write(gpa: hv_gpaddr_t, data: u32) -> hv_return_t;
    pub fn hv_vm_ioapic_get_state(state: *mut hv_ioapic_state_ext_t) -> hv_return_t;
    pub fn hv_vm_ioapic_put_state(state: *const hv_ioapic_state_ext_t) -> hv_return_t;
    pub fn hv_vm_send_ioapic_intr(data: u64) -> hv_return_t;
    pub fn hv_vm_atpic_assert_irq(irq: i32) -> hv_return_t;
    pub fn hv_vm_atpic_deassert_irq(irq: i32) -> hv_return_t;
    pub fn hv_vm_atpic_port_read(port: i32, valuep: *mut u8) -> hv_return_t;
    pub fn hv_vm_atpic_port_write(port: i32, value: u8) -> hv_return_t;
    pub fn hv_vm_atpic_get_state(state: *mut hv_atpic_state_ext_t, is_primary: bool)
    -> hv_return_t;
    pub fn hv_vm_atpic_put_state(
        state: *const hv_atpic_state_ext_t,
        is_primary: bool,
    ) -> hv_return_t;
    pub fn hv_vm_set_apic_bus_freq(freq: u64) -> hv_return_t;
    pub fn hv_vcpu_inject_extint(vcpu: hv_vcpuid_t) -> hv_return_t;
    pub fn hv_vcpu_apic_read(vcpu: hv_vcpuid_t, offset: u32, data: *mut u32) -> hv_return_t;
    pub fn hv_vcpu_apic_write(
        vcpu: hv_vcpuid_t,
        offset: u32,
        data: u32,
        no_side_effect: *mut bool,
    ) -> hv_return_t;
    pub fn hv_vcpu_apic_get_state(
        vcpu: hv_vcpuid_t,
        state: *mut hv_apic_state_ext_t,
    ) -> hv_return_t;
    pub fn hv_vcpu_apic_put_state(
        vcpu: hv_vcpuid_t,
        state: *const hv_apic_state_ext_t,
    ) -> hv_return_t;
    pub fn hv_vcpu_exit_info(vcpu: hv_vcpuid_t, code: *mut hv_vm_exitinfo_t) -> hv_return_t;
    pub fn hv_vcpu_exit_init_ap(vcpu: hv_vcpuid_t, is_actv: *mut bool, count: u32) -> hv_return_t;
    pub fn hv_vcpu_exit_startup_ap(
        vcpu: hv_vcpuid_t,
        is_actv: *mut bool,
        count: u32,
        ap_rip: *mut u64,
    ) -> hv_return_t;
    pub fn hv_vcpu_exit_ioapic_eoi(vcpu: hv_vcpuid_t, vec: *mut u8) -> hv_return_t;
    pub fn hv_vcpu_exit_apic_access_read(vcpu: hv_vcpuid_t, value: *mut u32) -> hv_return_t;
    pub fn hv_vcpu_exit_inject_excp(
        vcpu: hv_vcpuid_t,
        vec: *mut u8,
        valid: *mut bool,
        code: *mut u32,
        restart: *mut bool,
    ) -> hv_return_t;
    pub fn hv_vcpu_apic_lsc_enter_r32(
        vcpu: hv_vcpuid_t,
        is_load: bool,
        rip: u64,
        ilen: u32,
        cs: u16,
        reg: hv_x86_reg_t,
        uva: *mut u64,
        count: u32,
    ) -> hv_return_t;
    pub fn hv_vcpu_apic_lsc_enter_imm32(
        vcpu: hv_vcpuid_t,
        rip: u64,
        ilen: u32,
        cs: u16,
        imm32: u32,
        uva: *mut u64,
        count: u32,
    ) -> hv_return_t;
    pub fn hv_vcpu_apic_lsc_invalidate(vcpu: hv_vcpuid_t) -> hv_return_t;
    pub fn hv_vcpu_apic_ctrl(vcpu: hv_vcpuid_t, ctrls: hv_apic_ctrl_t) -> hv_return_t;
    pub fn hv_vcpu_apic_trigger_lvt(vcpu: hv_vcpuid_t, flavor: hv_apic_lvt_flavor_t)
    -> hv_return_t;
}

pub const HV_MSR_IA32_TSC: u32 = 16;
pub const HV_MSR_IA32_SPEC_CTRL: u32 = 72;
pub const HV_MSR_IA32_PRED_CMD: u32 = 73;
pub const HV_MSR_IA32_PMC0: u32 = 193;
pub const HV_MSR_IA32_PMC7: u32 = 200;
pub const HV_MSR_IA32_ARCH_CAPABILITIES: u32 = 266;
pub const HV_MSR_IA32_FLUSH_CMD: u32 = 267;
pub const HV_MSR_IA32_SYSENTER_CS: u32 = 372;
pub const HV_MSR_IA32_SYSENTER_ESP: u32 = 373;
pub const HV_MSR_IA32_SYSENTER_EIP: u32 = 374;
pub const HV_MSR_IA32_PERFEVNTSEL0: u32 = 390;
pub const HV_MSR_IA32_PERFEVNTSEL7: u32 = 397;
pub const HV_MSR_LBR_SELECT: u32 = 456;
pub const HV_MSR_LASTBRANCH_TOS: u32 = 457;
pub const HV_MSR_LASTINT_FROM_IP: u32 = 477;
pub const HV_MSR_LASTINT_TO_IP: u32 = 478;
pub const HV_MSR_IA32_DEBUGCTL: u32 = 473;
pub const HV_MSR_IA32_FIXED_CTR0: u32 = 777;
pub const HV_MSR_IA32_FIXED_CTR1: u32 = 778;
pub const HV_MSR_IA32_FIXED_CTR2: u32 = 779;
pub const HV_MSR_IA32_FIXED_CTR3: u32 = 780;
pub const HV_MSR_PERF_METRICS: u32 = 809;
pub const HV_MSR_IA32_FIXED_CTR_CTRL: u32 = 909;
pub const HV_MSR_IA32_PERF_GLOBAL_STATUS: u32 = 910;
pub const HV_MSR_IA32_PERF_GLOBAL_CTRL: u32 = 911;
pub const HV_MSR_IA32_PERF_GLOBAL_STATUS_RESET: u32 = 912;
pub const HV_MSR_IA32_PERF_GLOBAL_STATUS_SET: u32 = 913;
pub const HV_MSR_IA32_PERF_GLOBAL_INUSE: u32 = 914;
pub const HV_MSR_IA32_A_PMC0: u32 = 1217;
pub const HV_MSR_IA32_A_PMC7: u32 = 1224;
pub const HV_MSR_LASTBRANCH_0_FROM_IP: u32 = 1664;
pub const HV_MSR_LASTBRANCH_31_FROM_IP: u32 = 1695;
pub const HV_MSR_LASTBRANCH_0_TO_IP: u32 = 1728;
pub const HV_MSR_LASTBRANCH_31_TO_IP: u32 = 1759;
pub const HV_MSR_IA32_XSS: u32 = 3488;
pub const HV_MSR_LASTBRANCH_INFO_0: u32 = 3520;
pub const HV_MSR_LASTBRANCH_INFO_31: u32 = 3551;
pub const HV_MSR_IA32_EFER: u32 = 3221225600;
pub const HV_MSR_IA32_STAR: u32 = 3221225601;
pub const HV_MSR_IA32_LSTAR: u32 = 3221225602;
pub const HV_MSR_IA32_CSTAR: u32 = 3221225603;
pub const HV_MSR_IA32_FMASK: u32 = 3221225604;
pub const HV_MSR_IA32_FS_BASE: u32 = 3221225728;
pub const HV_MSR_IA32_GS_BASE: u32 = 3221225729;
pub const HV_MSR_IA32_KERNEL_GS_BASE: u32 = 3221225730;
pub const HV_MSR_IA32_TSC_AUX: u32 = 3221225731;

unsafe extern "C" {
    pub fn hv_vmx_vcpu_read_vmcs(vcpu: hv_vcpuid_t, field: u32, value: *mut u64) -> hv_return_t;
    pub fn hv_vmx_vcpu_write_vmcs(vcpu: hv_vcpuid_t, field: u32, value: u64) -> hv_return_t;
    pub fn hv_vmx_vcpu_get_cap_write_vmcs(
        vcpu: hv_vcpuid_t,
        field: u32,
        allowed_0: *mut u64,
        allowed_1: *mut u64,
    ) -> hv_return_t;
    pub fn hv_vmx_vcpu_read_shadow_vmcs(
        vcpu: hv_vcpuid_t,
        field: u32,
        value: *mut u64,
    ) -> hv_return_t;
    pub fn hv_vmx_vcpu_write_shadow_vmcs(vcpu: hv_vcpuid_t, field: u32, value: u64) -> hv_return_t;
}

pub type hv_shadow_flags_t = u64;
pub const HV_SHADOW_VMCS_NONE: hv_shadow_flags_t = 0;
pub const HV_SHADOW_VMCS_READ: hv_shadow_flags_t = 1;
pub const HV_SHADOW_VMCS_WRITE: hv_shadow_flags_t = 2;

unsafe extern "C" {
    pub fn hv_vmx_vcpu_set_shadow_access(
        vcpu: hv_vcpuid_t,
        field: u32,
        flags: hv_shadow_flags_t,
    ) -> hv_return_t;
}

pub type hv_vmx_capability_t = u32;
pub const HV_VMX_CAP_PINBASED: hv_vmx_capability_t = 0;
pub const HV_VMX_CAP_PROCBASED: hv_vmx_capability_t = 1;
pub const HV_VMX_CAP_PROCBASED2: hv_vmx_capability_t = 2;
pub const HV_VMX_CAP_ENTRY: hv_vmx_capability_t = 3;
pub const HV_VMX_CAP_EXIT: hv_vmx_capability_t = 4;
pub const HV_VMX_CAP_BASIC: hv_vmx_capability_t = 5;
pub const HV_VMX_CAP_TRUE_PINBASED: hv_vmx_capability_t = 6;
pub const HV_VMX_CAP_TRUE_PROCBASED: hv_vmx_capability_t = 7;
pub const HV_VMX_CAP_TRUE_ENTRY: hv_vmx_capability_t = 8;
pub const HV_VMX_CAP_TRUE_EXIT: hv_vmx_capability_t = 9;
pub const HV_VMX_CAP_MISC: hv_vmx_capability_t = 10;
pub const HV_VMX_CAP_CR0_FIXED0: hv_vmx_capability_t = 11;
pub const HV_VMX_CAP_CR0_FIXED1: hv_vmx_capability_t = 12;
pub const HV_VMX_CAP_CR4_FIXED0: hv_vmx_capability_t = 13;
pub const HV_VMX_CAP_CR4_FIXED1: hv_vmx_capability_t = 14;
pub const HV_VMX_CAP_VMCS_ENUM: hv_vmx_capability_t = 15;
pub const HV_VMX_CAP_EPT_VPID_CAP: hv_vmx_capability_t = 16;
pub const HV_VMX_CAP_PREEMPTION_TIMER: hv_vmx_capability_t = 32;

unsafe extern "C" {
    pub fn hv_vmx_read_capability(field: hv_vmx_capability_t, value: *mut u64) -> hv_return_t;
}

pub type hv_vmx_msr_info_t = u64;
pub const HV_VMX_INFO_MSR_IA32_ARCH_CAPABILITIES: hv_vmx_msr_info_t = 0;
pub const HV_VMX_INFO_MSR_IA32_PERF_CAPABILITIES: hv_vmx_msr_info_t = 1;
pub const HV_VMX_VALID_MSR_IA32_PERFEVNTSEL: hv_vmx_msr_info_t = 2;
pub const HV_VMX_VALID_MSR_IA32_FIXED_CTR_CTRL: hv_vmx_msr_info_t = 3;
pub const HV_VMX_VALID_MSR_IA32_PERF_GLOBAL_CTRL: hv_vmx_msr_info_t = 4;
pub const HV_VMX_VALID_MSR_IA32_PERF_GLOBAL_STATUS: hv_vmx_msr_info_t = 5;
pub const HV_VMX_VALID_MSR_IA32_DEBUGCTL: hv_vmx_msr_info_t = 6;
pub const HV_VMX_VALID_MSR_IA32_SPEC_CTRL: hv_vmx_msr_info_t = 7;
pub const HV_VMX_NEED_MSR_IA32_SPEC_CTRL: hv_vmx_msr_info_t = 8;

unsafe extern "C" {
    pub fn hv_vmx_get_msr_info(field: hv_vmx_msr_info_t, value: *mut u64) -> hv_return_t;
    pub fn hv_vmx_vcpu_set_apic_address(vcpu: hv_vcpuid_t, gpa: hv_gpaddr_t) -> hv_return_t;
    pub fn hv_vmx_vcpu_set_apic_address_space(
        vcpu: hv_vcpuid_t,
        asid: hv_vm_space_t,
        gpa: hv_gpaddr_t,
    ) -> hv_return_t;
}
