use std::fmt::{Display, Formatter, Result as FmtResult};

use disarm64::{Opcode, decoder::decode, format_insn::format_insn_pc};
use disarm64_defn::defn::InsnOpcode;

use super::ptr::Uintptr;

#[derive(Debug, Clone, Copy)]
pub struct Disasm(Uintptr);

impl Disasm {
    fn write_insn(f: &mut Formatter<'_>, insn: &str) -> FmtResult {
        if let Some((name, args)) = insn.split_once("\t\t") {
            write!(f, "{name:-8} {args}")
        } else {
            write!(f, "{insn}")
        }
    }
}

impl Disasm {
    fn format_insn(&self, insn: &mut String, opcode: &Opcode) -> FmtResult {
        format_insn_pc(self.0.as_u64(), insn, opcode)
    }
}

impl Display for Disasm {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(ref opcode) = decode(self.0.read()) {
            let mut insn = String::with_capacity(32);
            write!(f, "{:p}: {:08x}  ", self.0, opcode.bits())?;
            self.format_insn(&mut insn, opcode)?;
            Self::write_insn(f, &insn)?;
            Ok(())
        } else {
            write!(f, "{:p}: {:08x}  (???)", self.0, self.0.read::<u32>())
        }
    }
}

#[inline(always)]
pub fn disasm<P: Into<Uintptr>>(pc: P) -> Disasm {
    Disasm(pc.into())
}
