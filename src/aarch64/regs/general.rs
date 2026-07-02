use crate::{aarch64::ffi::*, macros::declare_friendly_enum};

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
    }
}

impl Reg {
    pub const FP: Self = Self::X29;
    pub const LR: Self = Self::X30;
}
