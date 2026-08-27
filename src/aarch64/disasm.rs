use std::fmt::{Display, Formatter, Result as FmtResult};

use disarm64::{InsnOpcode, decoder::decode, format_insn::format_insn_pc};

use crate::{aarch64::virtos::mem::VmMap, utils::ptr::VMA};

#[derive(Debug, Clone, Copy)]
pub struct Disasm(VMA);

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
        if let Some(phys) = VmMap::translate(self.0) {
            if let Some(ref opcode) = decode(phys.read()) {
                let mut buf = String::with_capacity(32);
                write!(f, "{pc:p}: {bits:08x}  ", pc = self.0, bits = opcode.bits())?;
                format_insn_pc(self.0.addr(), &mut buf, opcode)?;
                Self::write_insn(f, &buf)?;
                Ok(())
            } else {
                let insn = phys.read::<u32>();
                write!(f, "{pc:p}: {insn:08x}  (???)", pc = self.0)
            }
        } else {
            write!(f, "{pc:p}: ????????  (???)", pc = self.0)
        }
    }
}

#[inline(always)]
pub fn disasm(pc: VMA) -> Disasm {
    Disasm(pc)
}
