use std::fmt::Debug;

use super::SysReg;
use crate::macros::{declare_friendly_enum, define_bit_field};

const EC_UNCATEGORIZED: u64 = 0x00;
const EC_WFX_TRAP: u64 = 0x01;
const EC_CP15RT_TRAP: u64 = 0x03;
const EC_CP15RRT_TRAP: u64 = 0x04;
const EC_CP14RT_TRAP: u64 = 0x05;
const EC_CP14DT_TRAP: u64 = 0x06;
const EC_ADVSIMD_FP_ACCESS_TRAP: u64 = 0x07;
const EC_FPID_TRAP: u64 = 0x08;
const EC_PAC_TRAP: u64 = 0x09;
const EC_BXJ_TRAP: u64 = 0x0a;
const EC_CP14RRT_TRAP: u64 = 0x0c;
const EC_BTI_TRAP: u64 = 0x0d;
const EC_ILLEGAL_STATE: u64 = 0x0e;
const EC_AA32_SVC: u64 = 0x11;
const EC_AA32_HVC: u64 = 0x12;
const EC_AA32_SMC: u64 = 0x13;
const EC_AA64_SVC: u64 = 0x15;
const EC_AA64_HVC: u64 = 0x16;
const EC_AA64_SMC: u64 = 0x17;
const EC_SYS_REG_TRAP: u64 = 0x18;
const EC_SVE_ACCESS_TRAP: u64 = 0x19;
const EC_ERET_TRAP: u64 = 0x1a;
const EC_PAC_FAIL: u64 = 0x1c;
const EC_SME_TRAP: u64 = 0x1d;
const EC_GPC: u64 = 0x1e;
const EC_INST_ABORT: u64 = 0x20;
const EC_INST_ABORT_SAME_EL: u64 = 0x21;
const EC_PC_ALIGN: u64 = 0x22;
const EC_DATA_ABORT: u64 = 0x24;
const EC_DATA_ABORT_SAME_EL: u64 = 0x25;
const EC_SP_ALIGN: u64 = 0x26;
const EC_MOP: u64 = 0x27;
const EC_AA32_FPTRAP: u64 = 0x28;
const EC_AA64_FPTRAP: u64 = 0x2c;
const EC_GCS: u64 = 0x2d;
const EC_SERROR: u64 = 0x2f;
const EC_BREAKPOINT: u64 = 0x30;
const EC_BREAKPOINT_SAME_EL: u64 = 0x31;
const EC_SOFTWARE_STEP: u64 = 0x32;
const EC_SOFTWARE_STEP_SAME_EL: u64 = 0x33;
const EC_WATCHPOINT: u64 = 0x34;
const EC_WATCHPOINT_SAME_EL: u64 = 0x35;
const EC_AA32_BKPT: u64 = 0x38;
const EC_VECTOR_CATCH: u64 = 0x3a;
const EC_AA64_BKPT: u64 = 0x3c;

declare_friendly_enum! {
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

define_bit_field! {
    pub struct Syndrome : u64 {
        /// Instruction Specific Syndrome
        ISS: 25,

        // Instruction Length
        IL: 1,

        // Exception Class
        EC: 6,

        // Instruction Specific Syndrome 2
        ISS2: 5,

        // Reserved
        RES0: 27,
    }

    pub struct DataAbortISS : u32 {
        DFSC  : 6,
        WnR   : 1,
        S1PTW : 1,
        CM    : 1,
        EA    : 1,
        FnV   : 1,
        SET   : 2,
        RES0  : 1,
        AR    : 1,
        SF    : 1,
        SRT   : 5,
        SSE   : 1,
        SAS   : 2,
        ISV   : 1,
    }

    pub struct SysRegTrapISS : u32 {
        dir : 1,
        CRm : 4,
        Rt  : 5,
        CRn : 4,
        Op1 : 3,
        Op2 : 3,
        Op0 : 2,
    }
}

impl SysRegTrapISS {
    #[inline]
    pub fn sys_reg(self) -> Result<SysReg, u16> {
        SysReg::new(
            self.Op0() as u16,
            self.Op1() as u16,
            self.CRn() as u16,
            self.CRm() as u16,
            self.Op2() as u16,
        )
    }
}
