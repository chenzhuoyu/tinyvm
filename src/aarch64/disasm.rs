use std::fmt::{Display, Formatter, Result as FmtResult};

use disarm64::{decoder::decode, format_insn::format_insn_pc};

use super::paging::PageTable;
use crate::utils::ptr::Uintptr;

#[derive(Debug, Clone)]
pub struct Disasm {
    virt: Uintptr,
    insn: Option<u32>,
}

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
        if let Some(insn) = self.insn {
            if let Some(ref opcode) = decode(insn) {
                let mut buf = String::with_capacity(32);
                write!(f, "{:p}: {:08x}  ", self.virt, insn)?;
                format_insn_pc(self.virt.as_u64(), &mut buf, opcode)?;
                Self::write_insn(f, &buf)?;
                Ok(())
            } else {
                write!(f, "{:p}: {:08x}  (???)", self.virt, insn)
            }
        } else {
            write!(f, "{:p}: ????????  (???)", self.virt)
        }
    }
}

#[inline]
fn read_u32(phys: Uintptr) -> u32 {
    phys.read::<u32>()
}

#[inline]
pub fn disasm<P: Into<Uintptr>>(pc: P) -> Disasm {
    let virt = pc.into();
    let insn = PageTable::translate(virt.as_u64()).map(read_u32).ok();
    Disasm { virt, insn }
}
