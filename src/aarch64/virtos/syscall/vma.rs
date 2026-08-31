use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use crate::{
    aarch64::virtos::mem::VmMap,
    utils::{ptr::VMA, str::Sz},
};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct UserSz(VMA);

impl UserSz {
    #[inline]
    pub const fn new(addr: u64) -> Self {
        Self(VMA::new(addr))
    }
}

impl UserSz {
    #[inline]
    pub const fn vma(self) -> VMA {
        self.0
    }
}

impl UserSz {
    #[inline]
    pub fn translate(self) -> Option<Sz> {
        let addr = VmMap::translate(self.0)?;
        Some(Sz::from(addr.as_ptr()))
    }
}

impl Debug for UserSz {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(sz) = self.translate() {
            Debug::fmt(&sz, f)
        } else {
            write!(f, "(bad:{vma:p})", vma = self.0)
        }
    }
}

impl Display for UserSz {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(sz) = self.translate() {
            Display::fmt(&sz, f)
        } else {
            write!(f, "(bad:{vma:p})", vma = self.0)
        }
    }
}
