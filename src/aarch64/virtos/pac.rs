use std::arch::asm;

use crate::utils::ptr::Uintptr;

#[derive(Debug, Clone, Copy)]
pub enum SigningKey {
    IA,
    IB,
    DA,
    DB,
}

impl SigningKey {
    #[inline]
    fn do_sign(self, mut ptr: usize, mods: usize) -> usize {
        unsafe {
            match self {
                Self::IA => asm!("pacia {}, {}", inout(reg) ptr, in(reg) mods),
                Self::IB => asm!("pacib {}, {}", inout(reg) ptr, in(reg) mods),
                Self::DA => asm!("pacda {}, {}", inout(reg) ptr, in(reg) mods),
                Self::DB => asm!("pacdb {}, {}", inout(reg) ptr, in(reg) mods),
            }
        }
        ptr
    }
}

impl SigningKey {
    pub fn sign(self, ptr: Uintptr, slot: Uintptr, addr_div: bool, diversity: u16) -> Uintptr {
        Uintptr::new(self.do_sign(ptr.addr(), {
            if addr_div {
                (slot.addr() & ((1 << 48) - 1)) | ((diversity as usize) << 48)
            } else {
                diversity as usize
            }
        }))
    }
}

impl From<u64> for SigningKey {
    #[inline]
    fn from(value: u64) -> Self {
        match value {
            0 => Self::IA,
            1 => Self::IB,
            2 => Self::DA,
            3 => Self::DB,
            _ => panic!("invalid signing key: {value}"),
        }
    }
}
