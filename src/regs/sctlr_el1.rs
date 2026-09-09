use crate::macros::define_bit_field;

define_bit_field! {
    /// System Control Register (EL1)
    ///
    /// Provides top level control of the system, including its memory system,
    /// at `EL1` and `EL0`.
    pub struct SCTLR_EL1 : u64 {
        /// MMU enable for `EL1&0` stage 1 address translation.
        M: 1,

        /// Alignment check enable.
        ///
        /// This is the enable bit for Alignment fault checking at `EL1` and
        /// `EL0`.
        A: 1,

        /// Stage 1 Cacheability control, for data accesses.
        C: 1,

        /// `SP` Alignment check enable.
        ///
        /// When set to `1`, if a load or store instruction executed at `EL1`
        /// uses the `SP` as the base address and the `SP` is not aligned to a
        /// 16-byte boundary, then an SP alignment fault exception is generated.
        ///
        /// When the Effective value of `HCR_EL2.{E2H, TGE}` is `{1, 1}`, this
        /// bit has no effect on the PE.
        SA: 1,

        /// `SP` Alignment check enable for `EL0`.
        ///
        /// When set to `1`, if a load or store instruction executed at `EL0`
        /// uses the `SP` as the base address and the `SP` is not aligned to a
        /// 16-byte boundary, then an SP alignment fault exception is generated.
        ///
        /// When the Effective value of `HCR_EL2.{E2H, TGE}` is `{1, 1}`, this
        /// bit has no effect on execution at `EL0`.
        SA0: 1,

        /// System instruction memory barrier enable.
        ///
        /// Enables accesses to the `DMB`, `DSB`, and `ISB` System instructions
        /// in the `coproc==0b1111` encoding space from `EL0`:
        CP15BEN: 1,

        /// Non-aligned access.
        ///
        /// This bit controls generation of Alignment faults at `EL1` and `EL0`
        /// under certain conditions.
        ///
        /// The following instructions generate an Alignment fault if all bytes
        /// being accessed are not within a single 16-byte quantity, aligned to
        /// 16 bytes for access:
        ///
        ///   * `LDAPR`, `LDAPRH`, `LDAPUR`, `LDAPURH`, `LDAPURSH`, `LDAPURSW`,
        ///     `LDAR`, `LDARH`, `LDLAR`, `LDLARH`.
        ///
        ///   * `STLLR`, `STLLRH`, `STLR`, `STLRH`, `STLUR`, and `STLURH`.
        ///
        /// If `FEAT_LRCPC3` is implemented, the following instructions
        /// generate an Alignment fault if all bytes being accessed for a
        /// single register are not within a single 16-byte quantity, aligned
        /// to 16 bytes for access:
        ///
        ///   * `LDIAPP`, `STILP`, the post index versions of `LDAPR` and the
        ///     pre index versions of `STLR`.
        ///
        ///   * If Advanced SIMD and floating-point instructions are
        ///     implemented, `LDAPUR` (SIMD&FP), `LDAP1` (SIMD&FP), `STLUR`
        ///     (SIMD&FP), and `STL1` (SIMD&FP).
        nAA: 1,

        /// `IT` Disable.
        ///
        /// Disables some uses of `IT` instructions at `EL0` using AArch32.
        ITD: 1,

        /// `SETEND` instruction disable.
        ///
        /// Disables `SETEND` instructions at `EL0` using AArch32.
        SED: 1,

        /// User Mask Access.
        ///
        /// Traps `EL0` execution of `MSR` and `MRS` instructions that access
        /// the `PSTATE.{D, A, I, F}` masks to `EL1`, or to `EL2` when it is
        /// implemented and enabled for the current Security state and
        /// `HCR_EL2.TGE` is `1`, from AArch64 state only, reported using `EC`
        /// syndrome value `0x18`.
        UMA: 1,

        /// Enable `EL0` access to the following System instructions:
        ///
        ///   * `CFPRCTX`, `DVPRCTX` and `CPPRCTX` instructions.
        ///   * If `FEAT_SPECRES2` is implemented, `COSPRCTX`.
        ///   * `CFP RCTX`, `DVP RCTX` and `CPP RCTX` instructions.
        ///   * If `FEAT_SPECRES2` is implemented, `COSP RCTX`.
        EnRCTX: 1,

        /// Exception Exit is Context Synchronizing.
        EOS: 1,

        /// Stage 1 instruction access Cacheability control, for accesses at
        /// `EL0` and `EL1`.
        I: 1,

        /// Controls enabling of pointer authentication of instruction
        /// addresses, using the `APDBKey_EL1` key, in the `EL1&0` translation
        /// regime.
        EnDB: 1,

        /// Traps `EL0` execution of `DC ZVA` instructions to `EL1`, or to
        /// `EL2` when it is implemented and enabled for the current Security
        /// state and `HCR_EL2.TGE` is `1`, from AArch64 state only, reported
        /// using `EC` syndrome value `0x18`.
        ///
        /// If `FEAT_MTE` is implemented, this trap also applies to `DC GVA`
        /// and `DC GZVA`.
        DZE: 1,

        /// Traps `EL0` accesses to the `CTR_EL0` to `EL1`, or to `EL2` when it
        /// is implemented and enabled for the current Security state and
        /// `HCR_EL2.TGE` is `1`, from AArch64 state only, reported using `EC`
        /// syndrome value `0x18`.
        UCT: 1,

        /// Traps `EL0` execution of `WFI` instructions to `EL1`, or to `EL2`
        /// when it is implemented and enabled for the current Security state
        /// and `HCR_EL2.TGE` is `1`, from both Execution states, reported
        /// using `EC` syndrome value `0x01`.
        ///
        /// When `FEAT_WFxT` is implemented, this trap also applies to the
        /// `WFIT` instruction.
        nTWI: 1,

        /// Reserved, RES0.
        RES0: 1,

        /// Traps `EL0` execution of `WFE` instructions to `EL1`, or to `EL2`
        /// when it is implemented and enabled for the current Security state
        /// and `HCR_EL2.TGE` is `1`, from both Execution states, reported
        /// using `EC` syndrome value `0x01`.
        ///
        /// When `FEAT_WFxT` is implemented, this trap also applies to the
        /// `WFET` instruction.
        nTWE: 1,

        /// Write permission implies XN (Execute-never).
        ///
        /// For the `EL1&0` translation regime, this bit can restrict execute
        /// permissions on writeable pages.
        WXN: 1,

        /// Trap `EL0` Access to the `SCXTNUM_EL0` register, when `EL0` is
        /// using AArch64.
        TSCXT: 1,

        /// Implicit Error Synchronization event enable.
        IESB: 1,

        /// Exception Entry is Context Synchronizing.
        EIS: 1,

        /// Set Privileged Access Never, on taking an exception to `EL1`.
        SPAN: 1,

        /// Endianness of data accesses at `EL0`.
        E0E: 1,

        /// Endianness of data accesses at `EL1`, and stage 1 translation table
        /// walks in the `EL1&0` translation regime.
        EE: 1,

        /// Traps `EL0` execution of cache maintenance instructions, to `EL1`,
        /// or to `EL2` when it is implemented and enabled in the current
        /// Security state and `HCR_EL2.TGE` is `1`, from AArch64 state only,
        /// reported using `EC` syndrome value `0x18`, as follows:
        ///
        ///   * `DC CVAU`, `DC CIVAC`, `DC CVAC`, and `IC IVAU`.
        ///
        ///   * If `FEAT_MTE` is implemented, `DC CIGVAC`, `DC CIGDVAC`,
        ///     `DC CGVAC`, and `DC CGDVAC`.
        ///
        ///   * If `FEAT_DPB` is implemented, `DC CVAP`.
        ///
        ///   * If `FEAT_DPB` and `FEAT_MTE` are implemented, `DC CGVAP` and
        ///     `DC CGDVAP`.
        ///
        ///   * If `FEAT_DPB2` is implemented, `DC CVADP`.
        ///
        ///   * If `FEAT_DPB2` and FEAT_MTE are implemented, `DC CGVADP` and
        ///     `DC CGDVADP`.
        ///
        ///   * If `FEAT_OCCMO` is implemented, `DC CIVAOC`, `DC CIGDVAOC`,
        ///     `DC CVAOC` and `DC CGDVAOC`.
        UCI: 1,

        /// Controls enabling of pointer authentication of instruction
        /// addresses, using the `APDAKey_EL1` key, in the `EL1&0` translation
        /// regime.
        EnDA: 1,

        /// No Trap Load Multiple and Store Multiple to
        /// `Device-nGRE`/`Device-nGnRE`/`Device-nGnRnE` memory.
        nTLSMD: 1,

        /// Load Multiple and Store Multiple Atomicity and Ordering Enable.
        LSMAOE: 1,

        /// Controls enabling of pointer authentication of instruction
        /// addresses, using the `APIBKey_EL1` key, in the `EL1&0` translation
        /// regime.
        EnIB: 1,

        /// Controls enabling of pointer authentication of instruction
        /// addresses, using the `APIAKey_EL1` key, in the `EL1&0` translation
        /// regime.
        EnIA: 1,

        /// Controls cache maintenance instruction permission for the following
        /// instructions executed at `EL0`.
        ///
        ///   * `IC IVAU` and `DC CIVAC`.
        ///   * If `FEAT_MTE` is implemented, `DC CIGDVAC` and `DC CIGVAC`.
        CMOW: 1,

        /// Memory Copy and Memory Set instructions Enable.
        ///
        /// Enables execution of the Memory Copy and Memory Set instructions at
        /// `EL0`.
        MSCEn: 1,

        /// Enables direct and indirect accesses to `FPMR` from `EL0`.
        ///
        /// When accesses to `FPMR` are disabled by this control:
        ///
        ///   * Direct accesses to `FPMR` from `EL0` are trapped to `EL1`, or
        ///     to `EL2` when `EL2` is implemented and enabled in the current
        ///     Security state and `HCR_EL2.TGE` is `1`. These exceptions are
        ///     reported using `EC` syndrome value `0x18`.
        ///
        ///   * Execution of FP8 data-processing instructions that indirectly
        ///     access `FPMR` is `UNDEFINED` at `EL0`.
        ENFPM: 1,

        /// Configures the Branch Type compatibility of the implicit `BTI`
        /// behavior for the following instructions at `EL0`:
        ///
        ///   * `PACIASP`.
        ///   * `PACIBSP`.
        ///   * If `FEAT_PAuth_LR` is implemented, `PACIASPPC`.
        ///   * If `FEAT_PAuth_LR` is implemented, `PACIBSPPC`.
        BT0: 1,

        /// Configures the Branch Type compatibility of the implicit `BTI`
        /// behavior for the following instructions at `EL1`:
        ///
        ///   * `PACIASP`.
        ///   * `PACIBSP`.
        ///   * If `FEAT_PAuth_LR` is implemented, `PACIASPPC`.
        ///   * If `FEAT_PAuth_LR` is implemented, `PACIBSPPC`.
        BT1: 1,

        /// When synchronous exceptions are not being generated by Tag Check
        /// Faults, this field controls whether on exception entry into `EL1`,
        /// all Tag Check Faults due to instructions executed before exception
        /// entry, that are reported asynchronously, are synchronized into
        /// `TFSRE0_EL1` and `TFSR_EL1` registers.
        ITFSB: 1,

        /// Tag Check Fault in `EL0`.
        ///
        /// When the Effective value of `HCR_EL2.{E2H, TGE}` is not `{1, 1}`,
        /// controls the effect of Tag Check Faults due to Loads and Stores in
        /// `EL0`.
        TCF0: 2,

        /// Tag Check Fault in `EL1`.
        ///
        /// Controls the effect of Tag Check Faults due to Loads and Stores in
        /// `EL1`.
        TCF: 2,

        /// Allocation Tag Access in `EL0`.
        ///
        /// When `SCR_EL3.ATA == 1`, `HCR_EL2.ATA == 1`, and the Effective
        /// value of `HCR_EL2.{E2H, TGE}` is not `{1, 1}`, controls access to
        /// Allocation Tags and Tag Check operations in `EL0`.
        ATA0: 1,

        /// Allocation Tag Access in EL1.
        ///
        /// When `SCR_EL3.ATA == 1` and `HCR_EL2.ATA == 1`, controls access to
        /// Allocation Tags and Tag Check operations in `EL1`.
        ATA: 1,

        /// Default `PSTATE.SSBS` value on Exception Entry.
        DSSBS: 1,

        /// TWE Delay Enable.
        ///
        /// Enables a configurable delayed trap of the `WFE*` instruction
        /// caused by `SCTLR_EL1.nTWE`.
        TWEDEn: 1,

        /// TWE Delay.
        ///
        /// A 4-bit unsigned number that, when `SCTLR_EL1.TWEDEn` is `1`,
        /// encodes the minimum delay in taking a trap of `WFE*` caused by
        /// `SCTLR_EL1.nTWE` as `2^(TWEDEL + 8)` cycles.
        TWEDEL: 4,

        /// Forces a trivial implementation of the Transactional Memory
        /// Extension at `EL0`.
        TMT0: 1,

        /// Forces a trivial implementation of the Transactional Memory
        /// Extension at `EL1`.
        TMT: 1,

        /// Enables the Transactional Memory Extension at `EL0`.
        TME0: 1,

        /// Enables the Transactional Memory Extension at `EL1`.
        TME: 1,

        /// When the Effective value of `HCR_EL2.{E2H, TGE}` is not `{1, 1}`,
        /// traps execution of an `ST64BV` instruction at `EL0` to `EL1`.
        EnASR: 1,

        /// When the Effective value of `HCR_EL2.{E2H, TGE}` is not `{1, 1}`,
        /// traps execution of an `ST64BV0` instruction at `EL0` to `EL1`.
        EnAS0: 1,

        /// When the Effective value of `HCR_EL2.{E2H, TGE}` is not `{1, 1}`,
        /// traps execution of an `LD64B` or `ST64B` instruction at `EL0` to
        /// `EL1`.
        EnALS: 1,

        /// Enhanced Privileged Access Never.
        ///
        /// When `PSTATE.PAN` is `1`, determines whether an `EL1` data access
        /// to a page with stage 1 `EL0` instruction access permission
        /// generates a Permission fault as a result of the Privileged Access
        /// Never mechanism.
        EPAN: 1,

        /// When the Effective value of `HCR_EL2.{E2H, TGE}` is not `{1, 1}`,
        /// Tag Checking Store Only in `EL0`.
        TCSO0: 1,

        /// Tag Checking Store Only.
        TCSO: 1,

        /// Traps instructions executed at `EL0` that access `TPIDR2_EL0` to
        /// `EL1`, or to `EL2` when `EL2` is implemented and enabled for the
        /// current Security state and `HCR_EL2.TGE` is `1`. The exception is
        /// reported using `EC` syndrome value `0x18`.
        EnTP2: 1,

        /// Non-maskable Interrupt enable.
        NMI: 1,

        /// `SP` Interrupt Mask enable.
        ///
        /// When `SCTLR_EL1.NMI` is `1`, controls whether `PSTATE.SP` acts as
        /// an interrupt mask, and controls the value of `PSTATE.ALLINT` on
        /// taking an exception to `EL1`.
        SPINTMASK: 1,

        /// Trap `IMPLEMENTATION DEFINED` functionality.
        ///
        /// When the Effective value of `HCR_EL2.{E2H, TGE}` is not `{1, 1}`,
        /// traps `EL0` accesses to the encodings reserved for
        /// `IMPLEMENTATION DEFINED` functionality to `EL1`.
        TIDCP: 1,
    }
}
