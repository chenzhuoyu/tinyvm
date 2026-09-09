use crate::macros::define_bit_field;

define_bit_field! {
    /// Translation Control Register (EL1)
    ///
    /// The control register for stage 1 of the `EL1&0` translation regime.
    pub struct TCR_EL1 : u64 {
        /// The size offset of the memory region addressed by `TTBR0_EL1`.
        ///
        /// The region size is `2^(64-T0SZ)` bytes.
        T0SZ: 6,

        /// Reserved, RES0.
        RES0a: 1,

        /// Translation table walk disable for translations using `TTBR0_EL1`.
        ///
        /// This bit controls whether a translation table walk is performed on
        /// a TLB miss, for an address that is translated using `TTBR0_EL1`.
        EPD0: 1,

        /// Inner cacheability attribute for memory associated with translation
        /// table walks using `TTBR0_EL1`.
        IRGN0: 2,

        /// Outer cacheability attribute for memory associated with translation
        /// table walks using `TTBR0_EL1`.
        ORGN0: 2,

        /// Shareability attribute for memory associated with translation table
        /// walks using `TTBR0_EL1`.
        SH0: 2,

        /// Granule size for the `TTBR0_EL1`.
        TG0: 2,

        /// The size offset of the memory region addressed by `TTBR1_EL1`.
        ///
        /// The region size is `2^(64-T1SZ)` bytes.
        T1SZ: 6,

        /// Selects whether `TTBR0_EL1` or `TTBR1_EL1` defines the ASID.
        A1: 1,

        /// Translation table walk disable for translations using `TTBR1_EL1`.
        ///
        /// This bit controls whether a translation table walk is performed on
        /// a TLB miss, for an address that is translated using `TTBR1_EL1`.
        EPD1: 1,

        /// Inner cacheability attribute for memory associated with translation
        /// table walks using `TTBR1_EL1`.
        IRGN1: 2,

        /// Outer cacheability attribute for memory associated with translation
        /// table walks using `TTBR1_EL1`.
        ORGN1: 2,

        /// Shareability attribute for memory associated with translation table
        /// walks using `TTBR1_EL1`.
        SH1: 2,

        /// Granule size for the `TTBR1_EL1`.
        TG1: 2,

        /// Intermediate Physical Address Size.
        IPS: 3,

        /// Reserved, RES0.
        RES0b: 1,

        /// ASID Size.
        AS: 1,

        /// Top Byte ignored.
        ///
        /// Indicates whether the top byte of an address is used for address
        /// match for the `TTBR0_EL1` region, or ignored and used for tagged
        /// addresses.
        TBI0: 1,

        /// Top Byte ignored.
        ///
        /// Indicates whether the top byte of an address is used for address
        /// match for the `TTBR1_EL1` region, or ignored and used for tagged
        /// addresses.
        TBI1: 1,

        /// Hardware Access flag update in stage 1 translations from `EL0` and
        /// `EL1`.
        HA: 1,

        /// Hardware management of dirty state in stage 1 translations from
        /// `EL0` and `EL1`.
        HD: 1,

        /// Hierarchical Permission Disables.
        ///
        /// This affects the hierarchical control bits, APTable, PXNTable, and
        /// UXNTable, except NSTable, in the translation tables pointed to by
        /// `TTBR0_EL1`.
        HPD0: 1,

        /// Hierarchical Permission Disables.
        ///
        /// This affects the hierarchical control bits, APTable, PXNTable, and
        /// UXNTable, except NSTable, in the translation tables pointed to by
        /// `TTBR1_EL1`.
        HPD1: 1,

        /// Hardware Use.
        ///
        /// Indicates `IMPLEMENTATION DEFINED` hardware use of `bit[59]` of the
        /// stage 1 translation table Block or Page entry for translations
        /// using `TTBR0_EL1`.
        HWU059: 1,

        /// Hardware Use.
        ///
        /// Indicates `IMPLEMENTATION DEFINED` hardware use of `bit[60]` of the
        /// stage 1 translation table Block or Page entry for translations
        /// using `TTBR0_EL1`.
        HWU060: 1,

        /// Hardware Use.
        ///
        /// Indicates `IMPLEMENTATION DEFINED` hardware use of `bit[61]` of the
        /// stage 1 translation table Block or Page entry for translations
        /// using `TTBR0_EL1`.
        HWU061: 1,

        /// Hardware Use.
        ///
        /// Indicates `IMPLEMENTATION DEFINED` hardware use of `bit[62]` of the
        /// stage 1 translation table Block or Page entry for translations
        /// using `TTBR0_EL1`.
        HWU062: 1,

        /// Hardware Use.
        ///
        /// Indicates `IMPLEMENTATION DEFINED` hardware use of `bit[59]` of the
        /// stage 1 translation table Block or Page entry for translations
        /// using `TTBR1_EL1`.
        HWU159: 1,

        /// Hardware Use.
        ///
        /// Indicates `IMPLEMENTATION DEFINED` hardware use of `bit[60]` of the
        /// stage 1 translation table Block or Page entry for translations
        /// using `TTBR1_EL1`.
        HWU160: 1,

        /// Hardware Use.
        ///
        /// Indicates `IMPLEMENTATION DEFINED` hardware use of `bit[61]` of the
        /// stage 1 translation table Block or Page entry for translations
        /// using `TTBR1_EL1`.
        HWU161: 1,

        /// Hardware Use.
        ///
        /// Indicates `IMPLEMENTATION DEFINED` hardware use of `bit[62]` of the
        /// stage 1 translation table Block or Page entry for translations
        /// using `TTBR1_EL1`.
        HWU162: 1,

        /// Controls the use of the top byte of instruction addresses for
        /// address matching.
        ///
        /// For the purpose of this field, all cache maintenance and address
        /// translation instructions that perform address translation are
        /// treated as data accesses.
        ///
        /// This affects addresses where the address would be translated by
        /// tables pointed to by `TTBR0_EL1`.
        TBID0: 1,

        /// Controls the use of the top byte of instruction addresses for
        /// address matching.
        ///
        /// For the purpose of this field, all cache maintenance and address
        /// translation instructions that perform address translation are
        /// treated as data accesses.
        ///
        /// This affects addresses where the address would be translated by
        /// tables pointed to by `TTBR1_EL1`.
        TBID1: 1,

        /// Non-Fault translation timing Disable when using `TTBR0_EL1`.
        ///
        /// Controls how a TLB miss is reported in response to a non-fault
        /// unprivileged access for a virtual address that is translated using
        /// `TTBR0_EL1`.
        ///
        /// If SVE is implemented, the affected access types include:
        ///
        ///   * All accesses due to an SVE non-fault contiguous load
        ///     instruction.
        ///
        ///   * Accesses due to an SVE first-fault gather load instruction that
        ///     are not for the First active element. Accesses due to an SVE
        ///     first-fault contiguous load instruction are not affected.
        ///
        ///   * Accesses due to prefetch instructions might be affected, but
        ///     the effect is not architecturally visible.
        NFD0: 1,

        /// Non-Fault translation timing Disable when using `TTBR1_EL1`.
        ///
        /// Controls how a TLB miss is reported in response to a non-fault
        /// unprivileged access for a virtual address that is translated using
        /// `TTBR1_EL1`.
        ///
        /// If SVE is implemented, the affected access types include:
        ///
        ///   * All accesses due to an SVE non-fault contiguous load
        ///     instruction.
        ///
        ///   * Accesses due to an SVE first-fault gather load instruction that
        ///     are not for the First active element. Accesses due to an SVE
        ///     first-fault contiguous load instruction are not affected.
        ///
        ///   * Accesses due to prefetch instructions might be affected, but
        ///     the effect is not architecturally visible.
        NFD1: 1,

        /// Faulting control for unprivileged access to any address translated
        /// by `TTBR0_EL1`.
        E0PD0: 1,

        /// Faulting control for unprivileged access to any address translated
        /// by `TTBR1_EL1`.
        E0PD1: 1,

        /// Controls the generation of Unchecked accesses at `EL1`, and at
        /// `EL0` if the Effective value of `HCR_EL2.{E2H, TGE}` is not
        /// `{1, 1}`, when `address[59:55] = 0b00000`.
        TCMA0: 1,

        /// Controls the generation of Unchecked accesses at `EL1`, and at
        /// `EL0` if the Effective value of `HCR_EL2.{E2H, TGE}` is not
        /// `{1, 1}`, when `address[59:55] = 0b11111`.
        TCMA1: 1,

        /// This field affects:
        ///
        ///   * Whether a 52-bit output address can be described by the
        ///     translation tables of the 4KB or 16KB translation granules.
        ///
        ///   * The minimum value of `TCR_EL1.{T0SZ,T1SZ}`.
        ///
        ///   * How and where shareability for Block and Page descriptors are
        ///     encoded.
        DS: 1,

        /// Extended memory tag checking.
        ///
        /// This field controls address generation and Canonical tagging when
        /// `EL0` and `EL1` are using AArch64 where the data address would be
        /// translated by tables pointed to by `TTBR0_EL1`.
        ///
        /// This control has an effect regardless of whether stage 1 of the
        /// `EL1&0` translation regime is enabled or not.
        MTX0: 1,

        /// Extended memory tag checking.
        ///
        /// This field controls address generation and Canonical tagging when
        /// `EL0` and `EL1` are using AArch64 where the data address would be
        /// translated by tables pointed to by `TTBR1_EL1`.
        ///
        /// This control has an effect regardless of whether stage 1 of the
        /// `EL1&0` translation regime is enabled or not.
        MTX1: 1,

        /// Reserved, RES0.
        RES0c: 2,
    }
}
