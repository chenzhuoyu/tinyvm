use std::fmt::{Display, Formatter, Result as FmtResult};

use disarm64::{InsnOpcode, decoder::decode, format_insn::format_insn_pc};

use super::paging::PageTable;
use crate::utils::ptr::Uintptr;

#[derive(Debug, Clone, Copy)]
pub struct Disasm(Uintptr);

impl Disasm {
    fn write_insn(f: &mut Formatter<'_>, buf: &str) -> FmtResult {
        if let Some((name, args)) = buf.split_once("\t\t") {
            write!(f, "{name:-8} {args}")
        } else {
            write!(f, "{buf}")
        }
    }
}

impl Display for Disasm {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Ok(addr) = PageTable::translate(self.0) {
            if let Some(ref opcode) = decode(addr.read()) {
                let mut buf = String::with_capacity(32);
                write!(f, "{:p}: {:08x}  ", self.0, opcode.bits())?;
                format_insn_pc(self.0.as_u64(), &mut buf, opcode)?;
                Self::write_insn(f, &buf)?;
                Ok(())
            } else {
                let insn = addr.read::<u32>();
                write!(f, "{:p}: {:08x}  (???)", self.0, insn)
            }
        } else {
            write!(f, "{:p}: ????????  (???)", self.0)
        }
    }
}

#[inline]
pub fn disasm<P: Into<Uintptr>>(pc: P) -> Disasm {
    Disasm(pc.into())
}
