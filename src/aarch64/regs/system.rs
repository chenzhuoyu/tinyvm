use crate::{aarch64::ffi::*, macros::declare_friendly_enum};

const fn mksreg(op0: u16, op1: u16, crn: u16, crm: u16, op2: u16) -> u16 {
    assert!(op0 & !0b11 == 0);
    assert!(op1 & !0b111 == 0);
    assert!(op2 & !0b111 == 0);
    assert!(crn & !0b1111 == 0);
    assert!(crm & !0b1111 == 0);
    (op0 << 14) | (op1 << 11) | (crn << 7) | (crm << 3) | op2
}

const HV_SYS_REG_APL_HID0_EL1: u16 = mksreg(3, 0, 15, 0, 0);
const HV_SYS_REG_APL_EHID0_EL1: u16 = mksreg(3, 0, 15, 0, 1);
const HV_SYS_REG_APL_HID1_EL1: u16 = mksreg(3, 0, 15, 1, 0);
const HV_SYS_REG_APL_EHID1_EL1: u16 = mksreg(3, 0, 15, 1, 1);
const HV_SYS_REG_APL_HID2_EL1: u16 = mksreg(3, 0, 15, 2, 0);
const HV_SYS_REG_APL_EHID2_EL1: u16 = mksreg(3, 0, 15, 2, 1);
const HV_SYS_REG_APL_HID3_EL1: u16 = mksreg(3, 0, 15, 3, 0);
const HV_SYS_REG_APL_EHID3_EL1: u16 = mksreg(3, 0, 15, 3, 1);
const HV_SYS_REG_APL_HID4_EL1: u16 = mksreg(3, 0, 15, 4, 0);
const HV_SYS_REG_APL_EHID4_EL1: u16 = mksreg(3, 0, 15, 4, 1);
const HV_SYS_REG_APL_HID5_EL1: u16 = mksreg(3, 0, 15, 5, 0);
const HV_SYS_REG_APL_EHID5_EL1: u16 = mksreg(3, 0, 15, 5, 1);
const HV_SYS_REG_APL_HID6_EL1: u16 = mksreg(3, 0, 15, 6, 0);
const HV_SYS_REG_APL_HID7_EL1: u16 = mksreg(3, 0, 15, 7, 0);
const HV_SYS_REG_APL_EHID7_EL1: u16 = mksreg(3, 0, 15, 7, 1);
const HV_SYS_REG_APL_HID8_EL1: u16 = mksreg(3, 0, 15, 8, 0);
const HV_SYS_REG_APL_HID9_EL1: u16 = mksreg(3, 0, 15, 9, 0);
const HV_SYS_REG_APL_EHID9_EL1: u16 = mksreg(3, 0, 15, 9, 1);
const HV_SYS_REG_APL_HID10_EL1: u16 = mksreg(3, 0, 15, 10, 0);
const HV_SYS_REG_APL_EHID10_EL1: u16 = mksreg(3, 0, 15, 10, 1);
const HV_SYS_REG_APL_HID11_EL1: u16 = mksreg(3, 0, 15, 11, 0);
const HV_SYS_REG_APL_EHID11_EL1: u16 = mksreg(3, 0, 15, 11, 1);
const HV_SYS_REG_APL_HID12_EL1: u16 = mksreg(3, 0, 15, 12, 0);
const HV_SYS_REG_APL_HID13_EL1: u16 = mksreg(3, 0, 15, 14, 0);
const HV_SYS_REG_APL_HID14_EL1: u16 = mksreg(3, 0, 15, 15, 0);
const HV_SYS_REG_APL_HID16_EL1: u16 = mksreg(3, 0, 15, 15, 2);
const HV_SYS_REG_APL_HID17_EL1: u16 = mksreg(3, 0, 15, 15, 5);
const HV_SYS_REG_APL_HID18_EL1: u16 = mksreg(3, 0, 15, 11, 2);
const HV_SYS_REG_APL_EHID18_EL1: u16 = mksreg(3, 0, 15, 11, 3);
const HV_SYS_REG_APL_EHID20_EL1: u16 = mksreg(3, 0, 15, 1, 2);
const HV_SYS_REG_APL_HID21_EL1: u16 = mksreg(3, 0, 15, 1, 3);
const HV_SYS_REG_APL_HID26_EL1: u16 = mksreg(3, 0, 15, 0, 3);
const HV_SYS_REG_APL_HID27_EL1: u16 = mksreg(3, 0, 15, 0, 4);
const HV_SYS_REG_APL_PMCR0_EL1: u16 = mksreg(3, 1, 15, 0, 0);
const HV_SYS_REG_APL_PMCR1_EL1: u16 = mksreg(3, 1, 15, 1, 0);
const HV_SYS_REG_APL_PMCR2_EL1: u16 = mksreg(3, 1, 15, 2, 0);
const HV_SYS_REG_APL_PMCR3_EL1: u16 = mksreg(3, 1, 15, 3, 0);
const HV_SYS_REG_APL_PMCR4_EL1: u16 = mksreg(3, 1, 15, 4, 0);
const HV_SYS_REG_APL_PMESR0_EL1: u16 = mksreg(3, 1, 15, 5, 0);
const HV_SYS_REG_APL_PMESR1_EL1: u16 = mksreg(3, 1, 15, 6, 0);
const HV_SYS_REG_APL_PMCR1_GL1: u16 = mksreg(3, 1, 15, 8, 2);
const HV_SYS_REG_APL_PMSR_EL1: u16 = mksreg(3, 1, 15, 13, 0);
const HV_SYS_REG_APL_PMC0_EL1: u16 = mksreg(3, 2, 15, 0, 0);
const HV_SYS_REG_APL_PMC1_EL1: u16 = mksreg(3, 2, 15, 1, 0);
const HV_SYS_REG_APL_PMC2_EL1: u16 = mksreg(3, 2, 15, 2, 0);
const HV_SYS_REG_APL_PMC3_EL1: u16 = mksreg(3, 2, 15, 3, 0);
const HV_SYS_REG_APL_PMC4_EL1: u16 = mksreg(3, 2, 15, 4, 0);
const HV_SYS_REG_APL_PMC5_EL1: u16 = mksreg(3, 2, 15, 5, 0);
const HV_SYS_REG_APL_PMC6_EL1: u16 = mksreg(3, 2, 15, 6, 0);
const HV_SYS_REG_APL_PMC7_EL1: u16 = mksreg(3, 2, 15, 7, 0);
const HV_SYS_REG_APL_PMC8_EL1: u16 = mksreg(3, 2, 15, 9, 0);
const HV_SYS_REG_APL_PMC9_EL1: u16 = mksreg(3, 2, 15, 10, 0);
const HV_SYS_REG_APL_LSU_ERR_STS_EL1: u16 = mksreg(3, 3, 15, 0, 0);
const HV_SYS_REG_APL_E_LSU_ERR_STS_EL1: u16 = mksreg(3, 3, 15, 2, 0);
const HV_SYS_REG_APL_LSU_ERR_CTL_EL1: u16 = mksreg(3, 3, 15, 1, 0);
const HV_SYS_REG_APL_L2C_ERR_STS_EL1: u16 = mksreg(3, 3, 15, 8, 0);
const HV_SYS_REG_APL_L2C_ERR_ADR_EL1: u16 = mksreg(3, 3, 15, 9, 0);
const HV_SYS_REG_APL_L2C_ERR_INF_EL1: u16 = mksreg(3, 3, 15, 10, 0);
const HV_SYS_REG_APL_FED_ERR_STS_EL1: u16 = mksreg(3, 4, 15, 0, 0);
const HV_SYS_REG_APL_E_FED_ERR_STS_EL1: u16 = mksreg(3, 4, 15, 0, 2);
const HV_SYS_REG_APL_APCTL_EL1: u16 = mksreg(3, 4, 15, 0, 4);
const HV_SYS_REG_APL_SPR_LOCKDOWN_EL1: u16 = mksreg(3, 4, 15, 0, 5);
const HV_SYS_REG_APL_KERNKEYLO_EL1: u16 = mksreg(3, 4, 15, 1, 0);
const HV_SYS_REG_APL_KERNKEYHI_EL1: u16 = mksreg(3, 4, 15, 1, 1);
const HV_SYS_REG_APL_VMSA_LOCK_EL1: u16 = mksreg(3, 4, 15, 1, 2);
const HV_SYS_REG_APL_AMX_STATE_EL12: u16 = mksreg(3, 4, 15, 1, 3);
const HV_SYS_REG_APL_AMX_CONFIG_EL1: u16 = mksreg(3, 4, 15, 1, 4);
const HV_SYS_REG_APL_APRR_EL0: u16 = mksreg(3, 4, 15, 2, 0);
const HV_SYS_REG_APL_APRR_EL1: u16 = mksreg(3, 4, 15, 2, 1);
const HV_SYS_REG_APL_CTRR_LOCK_EL1: u16 = mksreg(3, 4, 15, 2, 2);
const HV_SYS_REG_APL_CTRR_A_LWR_EL1: u16 = mksreg(3, 4, 15, 2, 3);
const HV_SYS_REG_APL_CTRR_A_UPR_EL1: u16 = mksreg(3, 4, 15, 2, 4);
const HV_SYS_REG_APL_CTRR_CTL_EL1: u16 = mksreg(3, 4, 15, 2, 5);
const HV_SYS_REG_APL_VMSA_LOCK_EL12: u16 = mksreg(3, 4, 15, 2, 6);
const HV_SYS_REG_APL_APRR_JIT_MASK_EL2: u16 = mksreg(3, 4, 15, 2, 7);
const HV_SYS_REG_APL_AMX_CONFIG_EL12: u16 = mksreg(3, 4, 15, 4, 6);
const HV_SYS_REG_APL_AMX_CTL_EL2: u16 = mksreg(3, 4, 15, 4, 7);
const HV_SYS_REG_APL_CORE_INDEX: u16 = mksreg(3, 4, 15, 5, 0);
const HV_SYS_REG_APL_SPRR_PPERM_EL20_SILLY_THING: u16 = mksreg(3, 4, 15, 5, 1);
const HV_SYS_REG_APL_SPRR_UPERM_EL02: u16 = mksreg(3, 4, 15, 5, 2);
const HV_SYS_REG_APL_SPRR_UMPRR_EL2: u16 = mksreg(3, 4, 15, 7, 0);
const HV_SYS_REG_APL_SPRR_UPERM_SH1_EL2: u16 = mksreg(3, 4, 15, 7, 1);
const HV_SYS_REG_APL_SPRR_UPERM_SH2_EL2: u16 = mksreg(3, 4, 15, 7, 2);
const HV_SYS_REG_APL_SPRR_UPERM_SH3_EL2: u16 = mksreg(3, 4, 15, 7, 3);
const HV_SYS_REG_APL_SPRR_UMPRR_EL12: u16 = mksreg(3, 4, 15, 8, 0);
const HV_SYS_REG_APL_SPRR_UPERM_SH1_EL12: u16 = mksreg(3, 4, 15, 8, 1);
const HV_SYS_REG_APL_SPRR_UPERM_SH2_EL12: u16 = mksreg(3, 4, 15, 8, 2);
const HV_SYS_REG_APL_SPRR_UPERM_SH3_EL12: u16 = mksreg(3, 4, 15, 8, 3);
const HV_SYS_REG_APL_CTRR_A_LWR_EL12: u16 = mksreg(3, 4, 15, 9, 0);
const HV_SYS_REG_APL_CTRR_A_UPR_EL12: u16 = mksreg(3, 4, 15, 9, 1);
const HV_SYS_REG_APL_CTRR_B_LWR_EL12: u16 = mksreg(3, 4, 15, 9, 2);
const HV_SYS_REG_APL_CTRR_B_UPR_EL12: u16 = mksreg(3, 4, 15, 9, 3);
const HV_SYS_REG_APL_CTRR_CTL_EL12: u16 = mksreg(3, 4, 15, 9, 4);
const HV_SYS_REG_APL_CTRR_LOCK_EL12: u16 = mksreg(3, 4, 15, 9, 5);
const HV_SYS_REG_APL_SIQ_CFG_EL1: u16 = mksreg(3, 4, 15, 10, 4);
const HV_SYS_REG_APL_ACNTPCT_EL0: u16 = mksreg(3, 4, 15, 10, 5);
const HV_SYS_REG_APL_ACNTVCT_EL0: u16 = mksreg(3, 4, 15, 10, 6);
const HV_SYS_REG_APL_CTRR_A_LWR_EL2: u16 = mksreg(3, 4, 15, 11, 0);
const HV_SYS_REG_APL_CTRR_A_UPR_EL2: u16 = mksreg(3, 4, 15, 11, 1);
const HV_SYS_REG_APL_CTRR_CTL_EL2: u16 = mksreg(3, 4, 15, 11, 4);
const HV_SYS_REG_APL_CTRR_LOCK_EL2: u16 = mksreg(3, 4, 15, 11, 5);
const HV_SYS_REG_APL_AHCR_EL2: u16 = mksreg(3, 4, 15, 12, 1);
const HV_SYS_REG_APL_JCTL_EL0: u16 = mksreg(3, 4, 15, 15, 6);
const HV_SYS_REG_APL_IPI_RR_LOCAL_EL1: u16 = mksreg(3, 5, 15, 0, 0);
const HV_SYS_REG_APL_IPI_RR_GLOBAL_EL1: u16 = mksreg(3, 5, 15, 0, 1);
const HV_SYS_REG_APL_DPC_ERR_STS_EL1: u16 = mksreg(3, 5, 15, 0, 5);
const HV_SYS_REG_APL_IPI_SR_EL1: u16 = mksreg(3, 5, 15, 1, 1);
const HV_SYS_REG_APL_VM_TMR_LR_EL2: u16 = mksreg(3, 5, 15, 1, 2);
const HV_SYS_REG_APL_VM_TMR_FIQ_ENA_EL2: u16 = mksreg(3, 5, 15, 1, 3);
const HV_SYS_REG_APL_AWL_SCRATCH_EL1: u16 = mksreg(3, 5, 15, 2, 6);
const HV_SYS_REG_APL_IPI_CR_EL1: u16 = mksreg(3, 5, 15, 3, 1);
const HV_SYS_REG_APL_ACC_CFG_EL1: u16 = mksreg(3, 5, 15, 4, 0);
const HV_SYS_REG_APL_CYC_OVRD_EL1: u16 = mksreg(3, 5, 15, 5, 0);
const HV_SYS_REG_APL_ACC_OVRD_EL1: u16 = mksreg(3, 5, 15, 6, 0);
const HV_SYS_REG_APL_ACC_EBLK_OVRD_EL1: u16 = mksreg(3, 5, 15, 6, 1);
const HV_SYS_REG_APL_MMU_ERR_STS_EL1: u16 = mksreg(3, 6, 15, 0, 0);
const HV_SYS_REG_APL_AFSR1_GL1: u16 = mksreg(3, 6, 15, 0, 1);
const HV_SYS_REG_APL_AFSR1_GL2: u16 = mksreg(3, 6, 15, 0, 2);
const HV_SYS_REG_APL_AFSR1_GL12: u16 = mksreg(3, 6, 15, 0, 3);
const HV_SYS_REG_APL_SPRR_CONFIG_EL1: u16 = mksreg(3, 6, 15, 1, 0);
const HV_SYS_REG_APL_GXF_CONFIG_EL1: u16 = mksreg(3, 6, 15, 1, 2);
const HV_SYS_REG_APL_SPRR_AMRANGE_EL1: u16 = mksreg(3, 6, 15, 1, 3);
const HV_SYS_REG_APL_GXF_CONFIG_EL2: u16 = mksreg(3, 6, 15, 1, 4);
const HV_SYS_REG_APL_SPRR_UPERM_EL0: u16 = mksreg(3, 6, 15, 1, 5);
const HV_SYS_REG_APL_SPRR_PPERM_EL1: u16 = mksreg(3, 6, 15, 1, 6);
const HV_SYS_REG_APL_SPRR_PPERM_EL2: u16 = mksreg(3, 6, 15, 1, 7);
const HV_SYS_REG_APL_E_MMU_ERR_STS_EL1: u16 = mksreg(3, 6, 15, 2, 0);
const HV_SYS_REG_APL_PAC_GAL_EL12: u16 = mksreg(3, 6, 15, 2, 1);
const HV_SYS_REG_APL_PAC_GAH_EL12: u16 = mksreg(3, 6, 15, 2, 2);
const HV_SYS_REG_APL_KERNKEYLO_EL12: u16 = mksreg(3, 6, 15, 2, 3);
const HV_SYS_REG_APL_KERNKEYHI_EL12: u16 = mksreg(3, 6, 15, 2, 4);
const HV_SYS_REG_APL_AFPCR_EL0: u16 = mksreg(3, 6, 15, 2, 5);
const HV_SYS_REG_APL_AIDR2_EL1: u16 = mksreg(3, 6, 15, 2, 7);
const HV_SYS_REG_APL_SPRR_UMPRR_EL1: u16 = mksreg(3, 6, 15, 3, 0);
const HV_SYS_REG_APL_SPRR_PMPRR_EL1: u16 = mksreg(3, 6, 15, 3, 1);
const HV_SYS_REG_APL_SPRR_PMPRR_EL2: u16 = mksreg(3, 6, 15, 3, 2);
const HV_SYS_REG_APL_SPRR_UPERM_SH1_EL1: u16 = mksreg(3, 6, 15, 3, 3);
const HV_SYS_REG_APL_SPRR_UPERM_SH2_EL1: u16 = mksreg(3, 6, 15, 3, 4);
const HV_SYS_REG_APL_SPRR_UPERM_SH3_EL1: u16 = mksreg(3, 6, 15, 3, 5);
const HV_SYS_REG_APL_SPRR_PPERM_SH1_EL1: u16 = mksreg(3, 6, 15, 4, 2);
const HV_SYS_REG_APL_SPRR_PPERM_SH2_EL1: u16 = mksreg(3, 6, 15, 4, 3);
const HV_SYS_REG_APL_SPRR_PPERM_SH3_EL1: u16 = mksreg(3, 6, 15, 4, 4);
const HV_SYS_REG_APL_SPRR_PPERM_SH1_EL2: u16 = mksreg(3, 6, 15, 5, 1);
const HV_SYS_REG_APL_SPRR_PPERM_SH2_EL2: u16 = mksreg(3, 6, 15, 5, 2);
const HV_SYS_REG_APL_SPRR_PPERM_SH3_EL2: u16 = mksreg(3, 6, 15, 5, 3);
const HV_SYS_REG_APL_SPRR_PMPRR_EL12: u16 = mksreg(3, 6, 15, 6, 0);
const HV_SYS_REG_APL_SPRR_PPERM_SH1_EL12: u16 = mksreg(3, 6, 15, 6, 1);
const HV_SYS_REG_APL_SPRR_PPERM_SH2_EL12: u16 = mksreg(3, 6, 15, 6, 2);
const HV_SYS_REG_APL_SPRR_PPERM_SH3_EL12: u16 = mksreg(3, 6, 15, 6, 3);
const HV_SYS_REG_APL_PAC_IAL_EL12: u16 = mksreg(3, 6, 15, 7, 0);
const HV_SYS_REG_APL_PAC_IAH_EL12: u16 = mksreg(3, 6, 15, 7, 1);
const HV_SYS_REG_APL_PAC_IBL_EL12: u16 = mksreg(3, 6, 15, 7, 2);
const HV_SYS_REG_APL_PAC_IBH_EL12: u16 = mksreg(3, 6, 15, 7, 3);
const HV_SYS_REG_APL_PAC_DAL_EL12: u16 = mksreg(3, 6, 15, 7, 4);
const HV_SYS_REG_APL_PAC_DAH_EL12: u16 = mksreg(3, 6, 15, 7, 5);
const HV_SYS_REG_APL_PAC_DBL_EL12: u16 = mksreg(3, 6, 15, 7, 6);
const HV_SYS_REG_APL_PAC_DBH_EL12: u16 = mksreg(3, 6, 15, 7, 7);
const HV_SYS_REG_APL_GXF_STATUS_EL1: u16 = mksreg(3, 6, 15, 8, 0);
const HV_SYS_REG_APL_GXF_ENTRY_EL1: u16 = mksreg(3, 6, 15, 8, 1);
const HV_SYS_REG_APL_GXF_PABENTRY_EL1: u16 = mksreg(3, 6, 15, 8, 2);
const HV_SYS_REG_APL_ASPSR_EL1: u16 = mksreg(3, 6, 15, 8, 3);
const HV_SYS_REG_APL_VBAR_GL12: u16 = mksreg(3, 6, 15, 9, 2);
const HV_SYS_REG_APL_SPSR_GL12: u16 = mksreg(3, 6, 15, 9, 3);
const HV_SYS_REG_APL_ASPSR_GL12: u16 = mksreg(3, 6, 15, 9, 4);
const HV_SYS_REG_APL_ESR_GL12: u16 = mksreg(3, 6, 15, 9, 5);
const HV_SYS_REG_APL_ELR_GL12: u16 = mksreg(3, 6, 15, 9, 6);
const HV_SYS_REG_APL_FAR_GL12: u16 = mksreg(3, 6, 15, 9, 7);
const HV_SYS_REG_APL_SP_GL12: u16 = mksreg(3, 6, 15, 10, 0);
const HV_SYS_REG_APL_TPIDR_GL1: u16 = mksreg(3, 6, 15, 10, 1);
const HV_SYS_REG_APL_VBAR_GL1: u16 = mksreg(3, 6, 15, 10, 2);
const HV_SYS_REG_APL_SPSR_GL1: u16 = mksreg(3, 6, 15, 10, 3);
const HV_SYS_REG_APL_ASPSR_GL1: u16 = mksreg(3, 6, 15, 10, 4);
const HV_SYS_REG_APL_ESR_GL1: u16 = mksreg(3, 6, 15, 10, 5);
const HV_SYS_REG_APL_ELR_GL1: u16 = mksreg(3, 6, 15, 10, 6);
const HV_SYS_REG_APL_FAR_GL1: u16 = mksreg(3, 6, 15, 10, 7);
const HV_SYS_REG_APL_TPIDR_GL2: u16 = mksreg(3, 6, 15, 11, 1);
const HV_SYS_REG_APL_VBAR_GL2: u16 = mksreg(3, 6, 15, 11, 2);
const HV_SYS_REG_APL_SPSR_GL2: u16 = mksreg(3, 6, 15, 11, 3);
const HV_SYS_REG_APL_ASPSR_GL2: u16 = mksreg(3, 6, 15, 11, 4);
const HV_SYS_REG_APL_ESR_GL2: u16 = mksreg(3, 6, 15, 11, 5);
const HV_SYS_REG_APL_ELR_GL2: u16 = mksreg(3, 6, 15, 11, 6);
const HV_SYS_REG_APL_FAR_GL2: u16 = mksreg(3, 6, 15, 11, 7);
const HV_SYS_REG_APL_GXF_ENTRY_EL2: u16 = mksreg(3, 6, 15, 12, 0);
const HV_SYS_REG_APL_GXF_PABENTRY_EL2: u16 = mksreg(3, 6, 15, 12, 1);
const HV_SYS_REG_APL_APCTL_EL2: u16 = mksreg(3, 6, 15, 12, 2);
const HV_SYS_REG_APL_APSTS_EL2_MAYBE: u16 = mksreg(3, 6, 15, 12, 3);
const HV_SYS_REG_APL_APSTS_EL1: u16 = mksreg(3, 6, 15, 12, 4);
const HV_SYS_REG_APL_SPRR_CONFIG_EL2: u16 = mksreg(3, 6, 15, 14, 2);
const HV_SYS_REG_APL_SPRR_AMRANGE_EL2: u16 = mksreg(3, 6, 15, 14, 3);
const HV_SYS_REG_APL_VMKEYLO_EL2: u16 = mksreg(3, 6, 15, 14, 4);
const HV_SYS_REG_APL_VMKEYHI_EL2: u16 = mksreg(3, 6, 15, 14, 5);
const HV_SYS_REG_APL_ACTLR_EL12_PRE: u16 = mksreg(3, 6, 15, 14, 6);
const HV_SYS_REG_APL_APSTS_EL12: u16 = mksreg(3, 6, 15, 14, 7);
const HV_SYS_REG_APL_APCTL_EL12: u16 = mksreg(3, 6, 15, 15, 0);
const HV_SYS_REG_APL_GXF_CONFIG_EL12: u16 = mksreg(3, 6, 15, 15, 1);
const HV_SYS_REG_APL_GXF_ENTRY_EL12: u16 = mksreg(3, 6, 15, 15, 2);
const HV_SYS_REG_APL_GXF_PABENTRY_EL12: u16 = mksreg(3, 6, 15, 15, 3);
const HV_SYS_REG_APL_SPRR_CONFIG_EL12: u16 = mksreg(3, 6, 15, 15, 4);
const HV_SYS_REG_APL_SPRR_AMRANGE_EL12: u16 = mksreg(3, 6, 15, 15, 5);
const HV_SYS_REG_APL_SPRR_PPERM_EL12: u16 = mksreg(3, 6, 15, 15, 7);
const HV_SYS_REG_APL_UPMCR0_EL1: u16 = mksreg(3, 7, 15, 0, 4);
const HV_SYS_REG_APL_UPMESR0_EL1: u16 = mksreg(3, 7, 15, 1, 4);
const HV_SYS_REG_APL_UPMECM0_EL1: u16 = mksreg(3, 7, 15, 3, 4);
const HV_SYS_REG_APL_UPMECM1_EL1: u16 = mksreg(3, 7, 15, 4, 4);
const HV_SYS_REG_APL_UPMPCM_EL1: u16 = mksreg(3, 7, 15, 5, 4);
const HV_SYS_REG_APL_UPMSR_EL1: u16 = mksreg(3, 7, 15, 6, 4);
const HV_SYS_REG_APL_UPMECM2_EL1: u16 = mksreg(3, 7, 15, 8, 5);
const HV_SYS_REG_APL_UPMECM3_EL1: u16 = mksreg(3, 7, 15, 9, 5);
const HV_SYS_REG_APL_UPMESR1_EL1: u16 = mksreg(3, 7, 15, 11, 5);
const HV_SYS_REG_APL_UPMC0_EL1: u16 = mksreg(3, 7, 15, 7, 4);
const HV_SYS_REG_APL_UPMC1_EL1: u16 = mksreg(3, 7, 15, 8, 4);
const HV_SYS_REG_APL_UPMC2_EL1: u16 = mksreg(3, 7, 15, 9, 4);
const HV_SYS_REG_APL_UPMC3_EL1: u16 = mksreg(3, 7, 15, 10, 4);
const HV_SYS_REG_APL_UPMC4_EL1: u16 = mksreg(3, 7, 15, 11, 4);
const HV_SYS_REG_APL_UPMC5_EL1: u16 = mksreg(3, 7, 15, 12, 4);
const HV_SYS_REG_APL_UPMC6_EL1: u16 = mksreg(3, 7, 15, 13, 4);
const HV_SYS_REG_APL_UPMC7_EL1: u16 = mksreg(3, 7, 15, 14, 4);
const HV_SYS_REG_APL_UPMC8_EL1: u16 = mksreg(3, 7, 15, 0, 5);
const HV_SYS_REG_APL_UPMC9_EL1: u16 = mksreg(3, 7, 15, 1, 5);
const HV_SYS_REG_APL_UPMC10_EL1: u16 = mksreg(3, 7, 15, 2, 5);
const HV_SYS_REG_APL_UPMC11_EL1: u16 = mksreg(3, 7, 15, 3, 5);
const HV_SYS_REG_APL_UPMC12_EL1: u16 = mksreg(3, 7, 15, 4, 5);
const HV_SYS_REG_APL_UPMC13_EL1: u16 = mksreg(3, 7, 15, 5, 5);
const HV_SYS_REG_APL_UPMC14_EL1: u16 = mksreg(3, 7, 15, 6, 5);
const HV_SYS_REG_APL_UPMC15_EL1: u16 = mksreg(3, 7, 15, 7, 5);
const HV_SYS_REG_APL_AGTCNTRDIR_EL1: u16 = mksreg(3, 1, 15, 1, 5);
const HV_SYS_REG_APL_AGTCNTRDIR_EL12: u16 = mksreg(3, 4, 15, 14, 6);

declare_friendly_enum! {
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
        /// Hardware Implementation-Dependent Register 0
        APL_HID0_EL1,
        /// Hardware Implementation-Dependent Register 0 (E-core)
        APL_EHID0_EL1,
        /// Hardware Implementation-Dependent Register 1
        APL_HID1_EL1,
        /// Hardware Implementation-Dependent Register 1 (E-core)
        APL_EHID1_EL1,
        /// Hardware Implementation-Dependent Register 2
        APL_HID2_EL1,
        /// Hardware Implementation-Dependent Register 2 (E-core)
        APL_EHID2_EL1,
        /// Hardware Implementation-Dependent Register 3
        APL_HID3_EL1,
        /// Hardware Implementation-Dependent Register 3 (E-core)
        APL_EHID3_EL1,
        /// Hardware Implementation-Dependent Register 4
        APL_HID4_EL1,
        /// Hardware Implementation-Dependent Register 4 (E-core)
        APL_EHID4_EL1,
        /// Hardware Implementation-Dependent Register 5
        APL_HID5_EL1,
        /// Hardware Implementation-Dependent Register 5 (E-core)
        APL_EHID5_EL1,
        /// Hardware Implementation-Dependent Register 6
        APL_HID6_EL1,
        /// Hardware Implementation-Dependent Register 7
        APL_HID7_EL1,
        /// Hardware Implementation-Dependent Register 7 (E-core)
        APL_EHID7_EL1,
        /// Hardware Implementation-Dependent Register 8
        APL_HID8_EL1,
        /// Hardware Implementation-Dependent Register 9
        APL_HID9_EL1,
        /// Hardware Implementation-Dependent Register 9 (E-core)
        APL_EHID9_EL1,
        /// Hardware Implementation-Dependent Register 10
        APL_HID10_EL1,
        /// Hardware Implementation-Dependent Register 10 (E-core)
        APL_EHID10_EL1,
        /// Hardware Implementation-Dependent Register 11
        APL_HID11_EL1,
        /// Hardware Implementation-Dependent Register 11 (E-core)
        APL_EHID11_EL1,
        /// Hardware Implementation-Dependent Register 12
        APL_HID12_EL1,
        /// Hardware Implementation-Dependent Register 13
        APL_HID13_EL1,
        /// Hardware Implementation-Dependent Register 14
        APL_HID14_EL1,
        /// Hardware Implementation-Dependent Register 16
        APL_HID16_EL1,
        /// Hardware Implementation-Dependent Register 17
        APL_HID17_EL1,
        /// Hardware Implementation-Dependent Register 18
        APL_HID18_EL1,
        /// Hardware Implementation-Dependent Register 18 (E-core)
        APL_EHID18_EL1,
        /// Hardware Implementation-Dependent Register 20 (E-core)
        APL_EHID20_EL1,
        /// Hardware Implementation-Dependent Register 21
        APL_HID21_EL1,
        /// Hardware Implementation-Dependent Register 26
        APL_HID26_EL1,
        /// Hardware Implementation-Dependent Register 27
        APL_HID27_EL1,
        /// Performance Monitor Control Register 0
        APL_PMCR0_EL1,
        /// Performance Monitor Control Register 1
        APL_PMCR1_EL1,
        /// Performance Monitor Control Register 2
        APL_PMCR2_EL1,
        /// Performance Monitor Control Register 3
        APL_PMCR3_EL1,
        /// Performance Monitor Control Register 4
        APL_PMCR4_EL1,
        /// Performance Monitor Event Selection Register 0
        APL_PMESR0_EL1,
        /// Performance Monitor Event Selection Register 1
        APL_PMESR1_EL1,
        /// Performance Monitor Control Register 1 (GL1)
        APL_PMCR1_GL1,
        /// Performance Monitor Status Register
        APL_PMSR_EL1,
        /// Performance Monitor Counter 0
        APL_PMC0_EL1,
        /// Performance Monitor Counter 1
        APL_PMC1_EL1,
        /// Performance Monitor Counter 2
        APL_PMC2_EL1,
        /// Performance Monitor Counter 3
        APL_PMC3_EL1,
        /// Performance Monitor Counter 4
        APL_PMC4_EL1,
        /// Performance Monitor Counter 5
        APL_PMC5_EL1,
        /// Performance Monitor Counter 6
        APL_PMC6_EL1,
        /// Performance Monitor Counter 7
        APL_PMC7_EL1,
        /// Performance Monitor Counter 8
        APL_PMC8_EL1,
        /// Performance Monitor Counter 9
        APL_PMC9_EL1,
        /// Load-Store Unit Error Status
        APL_LSU_ERR_STS_EL1,
        /// Load-Store Unit Error Status (E-core)
        APL_E_LSU_ERR_STS_EL1,
        /// Load-Store Unit Error Control
        APL_LSU_ERR_CTL_EL1,
        /// L2 Cache Error Status
        APL_L2C_ERR_STS_EL1,
        /// L2 Cache Address
        APL_L2C_ERR_ADR_EL1,
        /// L2 Cache Error Information
        APL_L2C_ERR_INF_EL1,
        /// FED Error Status
        APL_FED_ERR_STS_EL1,
        /// FED Error Status (E-Core)
        APL_E_FED_ERR_STS_EL1,
        /// Pointer Authentication Control
        APL_APCTL_EL1,
        /// SPR Lockdown
        APL_SPR_LOCKDOWN_EL1,
        /// Pointer Authentication Kernel Key Low
        APL_KERNKEYLO_EL1,
        /// Pointer Authentication Kernel Key High
        APL_KERNKEYHI_EL1,
        /// Virtual Memory System Architecture Lock
        APL_VMSA_LOCK_EL1,
        /// AMX State (EL1)
        APL_AMX_STATE_EL12,
        /// AMX Config (EL1)
        APL_AMX_CONFIG_EL1,
        /// APRR EL0
        APL_APRR_EL0,
        /// APRR EL1
        APL_APRR_EL1,
        /// CTRR Lock
        APL_CTRR_LOCK_EL1,
        /// CTRR A Lower Address (EL1)
        APL_CTRR_A_LWR_EL1,
        /// CTRR A Upper Address (EL1)
        APL_CTRR_A_UPR_EL1,
        /// CTRR Control (EL1)
        APL_CTRR_CTL_EL1,
        /// Virtual Memory System Architecture Lock (EL12)
        APL_VMSA_LOCK_EL12,
        /// APRR JIT Mask
        APL_APRR_JIT_MASK_EL2,
        /// AMX Config (EL12)
        APL_AMX_CONFIG_EL12,
        /// AMX Control (EL2)
        APL_AMX_CTL_EL2,
        /// Core index in cluster
        APL_CORE_INDEX,
        /// SPRR Permission Configuration Register (EL
        APL_SPRR_PPERM_EL20_SILLY_THING,
        /// SPRR User Permission Configuration Register (EL02)
        APL_SPRR_UPERM_EL02,
        /// SPRR User MPRR (EL2)
        APL_SPRR_UMPRR_EL2,
        /// SPRR User Permission SH1 (EL2)
        APL_SPRR_UPERM_SH1_EL2,
        /// SPRR User Permission SH2 (EL2)
        APL_SPRR_UPERM_SH2_EL2,
        /// SPRR User Permission SH3 (EL2)
        APL_SPRR_UPERM_SH3_EL2,
        /// SPRR User MPRR (EL12)
        APL_SPRR_UMPRR_EL12,
        /// SPRR User Permission SH1 (EL12)
        APL_SPRR_UPERM_SH1_EL12,
        /// SPRR User Permission SH2 (EL12)
        APL_SPRR_UPERM_SH2_EL12,
        /// SPRR User Permission SH3 (EL12)
        APL_SPRR_UPERM_SH3_EL12,
        /// CTRR A Lower Address (EL12)
        APL_CTRR_A_LWR_EL12,
        /// CTRR A Upper Address (EL12)
        APL_CTRR_A_UPR_EL12,
        /// CTRR B Lower Address (EL12)
        APL_CTRR_B_LWR_EL12,
        /// CTRR B Upper Address (EL12)
        APL_CTRR_B_UPR_EL12,
        /// CTRR Control (EL12)
        APL_CTRR_CTL_EL12,
        /// CTRR Lock (EL12)
        APL_CTRR_LOCK_EL12,
        /// System Interrupt Configuration (EL1)
        APL_SIQ_CFG_EL1,
        /// Physical timer counter register (pre-spec CNTPCTSS_EL0)
        APL_ACNTPCT_EL0,
        /// Virtual timer counter register (pre-spec CNTVCTSS_EL0)
        APL_ACNTVCT_EL0,
        /// CTRR A Lower Address (EL2)
        APL_CTRR_A_LWR_EL2,
        /// CTRR A Upper Address (EL2)
        APL_CTRR_A_UPR_EL2,
        /// CTRR Control (EL2)
        APL_CTRR_CTL_EL2,
        /// CTRR Lock
        APL_CTRR_LOCK_EL2,
        /// AHCR (Apple HCR?)
        APL_AHCR_EL2,
        /// JITBox Control (EL0)
        APL_JCTL_EL0,
        /// IPI Request Register (Local)
        APL_IPI_RR_LOCAL_EL1,
        /// IPI Request Register (Global)
        APL_IPI_RR_GLOBAL_EL1,
        /// DPC Error Status
        APL_DPC_ERR_STS_EL1,
        /// IPI Status Register
        APL_IPI_SR_EL1,
        /// VM Timer Link Register
        APL_VM_TMR_LR_EL2,
        /// VM Timer FIQ Enable
        APL_VM_TMR_FIQ_ENA_EL2,
        /// AWL Scratch Register
        APL_AWL_SCRATCH_EL1,
        /// IPI Control Register
        APL_IPI_CR_EL1,
        /// Apple Core Cluster Configuration
        APL_ACC_CFG_EL1,
        /// Cyclone Override
        APL_CYC_OVRD_EL1,
        /// Apple Core Cluster Override
        APL_ACC_OVRD_EL1,
        /// Apple Core Cluster E-Block Override
        APL_ACC_EBLK_OVRD_EL1,
        /// MMU Error Status
        APL_MMU_ERR_STS_EL1,
        /// Auxiliary Fault Status Register 1 (GL1)
        APL_AFSR1_GL1,
        /// Auxiliary Fault Status Register 1 (GL2)
        APL_AFSR1_GL2,
        /// Auxiliary Fault Status Register 1 (GL12)
        APL_AFSR1_GL12,
        /// SPRR Configuration Register (EL1)
        APL_SPRR_CONFIG_EL1,
        /// GXF Configuration Register (EL1)
        APL_GXF_CONFIG_EL1,
        /// SPRR AM Range (EL1)
        APL_SPRR_AMRANGE_EL1,
        /// GXF Configuration Register (EL2)
        APL_GXF_CONFIG_EL2,
        /// SPRR User Permission Configuration Register (EL0)
        APL_SPRR_UPERM_EL0,
        /// SPRR Kernel Permission Configuration Register (EL1)
        APL_SPRR_PPERM_EL1,
        /// SPRR Kernel Permission Configuration Register (EL2)
        APL_SPRR_PPERM_EL2,
        /// MMU Error Status (E-Core)
        APL_E_MMU_ERR_STS_EL1,
        /// Pointer Authentication Key A for Code Low (EL12)
        APL_PAC_GAL_EL12,
        /// Pointer Authentication Key A for Code High (EL12)
        APL_PAC_GAH_EL12,
        /// Pointer Authentication Kernel Key Low (EL12)
        APL_KERNKEYLO_EL12,
        /// Pointer Authentication Kernel Key High (EL12)
        APL_KERNKEYHI_EL12,
        /// Apple Floating-Point Control Register
        APL_AFPCR_EL0,
        /// Apple ID Register 2
        APL_AIDR2_EL1,
        /// SPRR User MPRR (EL1)
        APL_SPRR_UMPRR_EL1,
        /// SPRR Kernel MPRR (EL1)
        APL_SPRR_PMPRR_EL1,
        /// SPRR Kernel MPRR (EL2)
        APL_SPRR_PMPRR_EL2,
        /// SPRR User Permission SH1 (EL1)
        APL_SPRR_UPERM_SH1_EL1,
        /// SPRR User Permission SH2 (EL1)
        APL_SPRR_UPERM_SH2_EL1,
        /// SPRR User Permission SH3 (EL1)
        APL_SPRR_UPERM_SH3_EL1,
        /// SPRR Kernel Permission SH1 (EL1)
        APL_SPRR_PPERM_SH1_EL1,
        /// SPRR Kernel Permission SH2 (EL1)
        APL_SPRR_PPERM_SH2_EL1,
        /// SPRR Kernel Permission SH3 (EL1)
        APL_SPRR_PPERM_SH3_EL1,
        /// SPRR Kernel Permission SH1 (EL2)
        APL_SPRR_PPERM_SH1_EL2,
        /// SPRR Kernel Permission SH2 (EL2)
        APL_SPRR_PPERM_SH2_EL2,
        /// SPRR Kernel Permission SH3 (EL2)
        APL_SPRR_PPERM_SH3_EL2,
        /// SPRR Kernel MPRR (EL12)
        APL_SPRR_PMPRR_EL12,
        /// SPRR Kernel Permission SH1 (EL12)
        APL_SPRR_PPERM_SH1_EL12,
        /// SPRR Kernel Permission SH2 (EL12)
        APL_SPRR_PPERM_SH2_EL12,
        /// SPRR Kernel Permission SH3 (EL12)
        APL_SPRR_PPERM_SH3_EL12,
        /// Pointer Authentication Key A for Instruction Low (EL12)
        APL_PAC_IAL_EL12,
        /// Pointer Authentication Key A for Instruction High (EL12)
        APL_PAC_IAH_EL12,
        /// Pointer Authentication Key A for Instruction Low (EL12)
        APL_PAC_IBL_EL12,
        /// Pointer Authentication Key A for Instruction High (EL12)
        APL_PAC_IBH_EL12,
        /// Pointer Authentication Key A for Data Low (EL12)
        APL_PAC_DAL_EL12,
        /// Pointer Authentication Key A for Data High (EL12)
        APL_PAC_DAH_EL12,
        /// Pointer Authentication Key A for Data Low (EL12)
        APL_PAC_DBL_EL12,
        /// Pointer Authentication Key A for Data High (EL12)
        APL_PAC_DBH_EL12,
        /// GXF Status Register (CurrentG)
        APL_GXF_STATUS_EL1,
        /// GXF genter Entry Vector Register (EL1)
        APL_GXF_ENTRY_EL1,
        /// GXF Abort Vector Register (EL1)
        APL_GXF_PABENTRY_EL1,
        /// ASPSR (EL1)
        APL_ASPSR_EL1,
        /// Vector Base Address Register (GL12)
        APL_VBAR_GL12,
        /// Saved Program Status Register (GL12)
        APL_SPSR_GL12,
        /// ASPSR (GL12)
        APL_ASPSR_GL12,
        /// Exception Syndrome Register (GL12)
        APL_ESR_GL12,
        /// Exception Link Register (GL12)
        APL_ELR_GL12,
        /// Fault Address Register (GL12)
        APL_FAR_GL12,
        /// Stack Pointer Register (GL12)
        APL_SP_GL12,
        /// Software Thread ID Register (GL1)
        APL_TPIDR_GL1,
        /// Vector Base Address Register (GL1)
        APL_VBAR_GL1,
        /// Saved Program Status Register (GL1)
        APL_SPSR_GL1,
        /// ASPSR (GL1)
        APL_ASPSR_GL1,
        /// Exception Syndrome Register (GL1)
        APL_ESR_GL1,
        /// Exception Link Register (GL1)
        APL_ELR_GL1,
        /// Fault Address Register (GL1)
        APL_FAR_GL1,
        /// Software Thread ID Register (GL2)
        APL_TPIDR_GL2,
        /// Vector Base Address Register (GL2)
        APL_VBAR_GL2,
        /// Saved Program Status Register (GL2)
        APL_SPSR_GL2,
        /// ASPSR (GL2)
        APL_ASPSR_GL2,
        /// Exception Syndrome Register (GL2)
        APL_ESR_GL2,
        /// Exception Link Register (GL2)
        APL_ELR_GL2,
        /// Fault Address Register (GL2)
        APL_FAR_GL2,
        /// GXF genter Entry Vector Register (EL2)
        APL_GXF_ENTRY_EL2,
        /// GXF Abort Vector Register (EL2)
        APL_GXF_PABENTRY_EL2,
        /// Pointer Authentication Control (EL2)
        APL_APCTL_EL2,
        /// Pointer Authentication Status (EL2, maybe)
        APL_APSTS_EL2_MAYBE,
        /// Pointer Authentication Status
        APL_APSTS_EL1,
        /// SPRR Configuration Register (EL2)
        APL_SPRR_CONFIG_EL2,
        /// SPRR AM Range (EL2)
        APL_SPRR_AMRANGE_EL2,
        /// Pointer Authentication VM Machine Key Low
        APL_VMKEYLO_EL2,
        /// Pointer Authentication VM Machine Key High
        APL_VMKEYHI_EL2,
        /// Auxiliary Control Register (EL12, pre-spec)
        APL_ACTLR_EL12_PRE,
        /// Pointer Authentication Status (EL12)
        APL_APSTS_EL12,
        /// Pointer Authentication Control (EL12)
        APL_APCTL_EL12,
        /// GXF Configuration Register (EL12)
        APL_GXF_CONFIG_EL12,
        /// GXF genter Entry Vector Register (EL12)
        APL_GXF_ENTRY_EL12,
        /// GXF Abort Vector Register (EL12)
        APL_GXF_PABENTRY_EL12,
        /// SPRR Configuration Register (EL12)
        APL_SPRR_CONFIG_EL12,
        /// SPRR AM Range (EL12)
        APL_SPRR_AMRANGE_EL12,
        /// SPRR Permission Configuration Register (EL12)
        APL_SPRR_PPERM_EL12,
        /// Uncore Performance Monitor Control Register 0
        APL_UPMCR0_EL1,
        /// Uncore Performance Monitor Event Selection Register 0
        APL_UPMESR0_EL1,
        /// Uncore Performance Monitor Event Core Mask 0
        APL_UPMECM0_EL1,
        /// Uncore Performance Monitor Event Core Mask 1
        APL_UPMECM1_EL1,
        /// Uncore Performance Monitor PMI Core Mask
        APL_UPMPCM_EL1,
        /// Uncore Performance Monitor Status Register
        APL_UPMSR_EL1,
        /// Uncore Performance Monitor Event Core Mask 2
        APL_UPMECM2_EL1,
        /// Uncore Performance Monitor Event Core Mask 3
        APL_UPMECM3_EL1,
        /// Uncore Performance Monitor Event Selection Register 1
        APL_UPMESR1_EL1,
        /// Uncore Performance Monitor Counter 0
        APL_UPMC0_EL1,
        /// Uncore Performance Monitor Counter 1
        APL_UPMC1_EL1,
        /// Uncore Performance Monitor Counter 2
        APL_UPMC2_EL1,
        /// Uncore Performance Monitor Counter 3
        APL_UPMC3_EL1,
        /// Uncore Performance Monitor Counter 4
        APL_UPMC4_EL1,
        /// Uncore Performance Monitor Counter 5
        APL_UPMC5_EL1,
        /// Uncore Performance Monitor Counter 6
        APL_UPMC6_EL1,
        /// Uncore Performance Monitor Counter 7
        APL_UPMC7_EL1,
        /// Uncore Performance Monitor Counter 8
        APL_UPMC8_EL1,
        /// Uncore Performance Monitor Counter 9
        APL_UPMC9_EL1,
        /// Uncore Performance Monitor Counter 10
        APL_UPMC10_EL1,
        /// Uncore Performance Monitor Counter 11
        APL_UPMC11_EL1,
        /// Uncore Performance Monitor Counter 12
        APL_UPMC12_EL1,
        /// Uncore Performance Monitor Counter 13
        APL_UPMC13_EL1,
        /// Uncore Performance Monitor Counter 14
        APL_UPMC14_EL1,
        /// Uncore Performance Monitor Counter 15
        APL_UPMC15_EL1,
        /// AGT Counter Redirect Register (EL1)
        APL_AGTCNTRDIR_EL1,
        /// AGT Counter Redirect Register (EL12)
        APL_AGTCNTRDIR_EL12,
    }
}

impl SysReg {
    #[inline]
    pub fn new(op0: u16, op1: u16, crn: u16, crm: u16, op2: u16) -> Result<Self, u16> {
        match mksreg(op0, op1, crn, crm, op2) {
            value if Self::is_sys_reg(value) => Ok(Self::from(value)),
            value => Err(value),
        }
    }
}
