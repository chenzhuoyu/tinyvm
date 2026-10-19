use std::fmt::Debug;

use crate::{
    cpu::Cpu,
    regs::{DataAbortISS, Reg},
    utils::ptr::Uintptr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmioKind {
    Read,
    ReadAtomic,
    Write,
    WriteAtomic,
    Execution,
}

#[derive(Debug, Clone, Copy)]
pub struct MmioRequest {
    pub data: u64,
    pub size: usize,
    pub kind: MmioKind,
}

#[derive(Debug, Clone, Copy)]
pub struct MmioContext<'p> {
    iss: Option<DataAbortISS>,
    pub pc: Uintptr,
    pub cpu: &'p Cpu,
    pub addr: Uintptr,
}

impl<'p> MmioContext<'p> {
    #[inline]
    pub fn new(cpu: &'p Cpu, iss: Option<DataAbortISS>, pc: Uintptr, addr: Uintptr) -> Self {
        Self { iss, pc, cpu, addr }
    }
}

impl MmioContext<'_> {
    #[inline]
    pub fn decode(&self) -> MmioRequest {
        const MMIO_KINDS: [MmioKind; 8] = [
            MmioKind::Read,
            MmioKind::Write,
            MmioKind::Read,
            MmioKind::Write,
            MmioKind::Read,
            MmioKind::Write,
            MmioKind::ReadAtomic,
            MmioKind::WriteAtomic,
        ];
        if let Some(iss) = self.iss {
            assert!(
                iss.ISV() == 1,
                "cannot decode MMIO request with invalid ISV bit"
            );
            let data = {
                if iss.WnR() == 1 {
                    self.cpu.read_reg(Reg::from(iss.SRT()))
                } else {
                    0
                }
            };
            MmioRequest {
                data,
                size: 1 << iss.SAS(),
                kind: MMIO_KINDS[iss.isv_ar_wnr() as usize],
            }
        } else {
            MmioRequest {
                data: 0,
                size: 0,
                kind: MmioKind::Execution,
            }
        }
    }

    #[inline]
    pub fn advance(&self, mut value: u64) {
        if let Some(iss) = self.iss
            && iss.WnR() == 0
            && iss.ISV() == 1
        {
            assert!(
                iss.ISV() != 0,
                "MMIO cannot make more progress without valid ISV"
            );
            match iss.sas_sse() {
                0b000 => value &= u8::MAX as u64,
                0b010 => value &= u16::MAX as u64,
                0b100 => value &= u32::MAX as u64,
                0b110 => {}
                0b001 => value = value as i8 as u64,
                0b011 => value = value as i16 as u64,
                0b101 => value = value as i32 as u64,
                0b111 => {}
                _ => unsafe { std::hint::unreachable_unchecked() },
            };
            if iss.SF() == 0 {
                value &= u32::MAX as u64;
            }
            self.cpu.write_reg(Reg::from(iss.SRT()), value);
            self.cpu.write_reg(Reg::PC, self.pc.as_u64() + 4);
        } else {
            panic!("MMIO cannot make more progress without valid ISV and WnR");
        }
    }
}

pub trait MmioHandler {
    fn handle(&self, mcx: MmioContext<'_>) -> bool;
}

impl<F: Fn(MmioContext<'_>) -> bool> MmioHandler for F {
    #[inline]
    fn handle(&self, mcx: MmioContext<'_>) -> bool {
        self(mcx)
    }
}
