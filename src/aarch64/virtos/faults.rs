use std::io::{ErrorKind, Read, Result as IoResult, Seek, SeekFrom};

use super::mem;
use crate::{
    aarch64::{disasm::disasm, paging::PAGE_SIZE, vm::Vm},
    mem::Protection,
    utils::ptr::Uintptr,
};

fn do_fetch_page<F: Read + Seek>(
    addr: Uintptr,
    base: Uintptr,
    file: &mut F,
    prot: Option<Protection>,
    offset: usize,
) -> IoResult<()> {
    let offs = (addr - base) & !(PAGE_SIZE - 1);
    let base = base + offs;

    /* sanity check the calculated address */
    debug_assert!(
        base <= addr && addr < base + PAGE_SIZE,
        "calculated page address {base:p} does not contain the requested address {addr:p}",
    );

    /* map pages into VM */
    Vm::map(base, PAGE_SIZE, Protection::WRITE);
    mem::protect(base, PAGE_SIZE, Protection::WRITE)?;

    /* seek to the specified location */
    let mut page = {
        file.seek(SeekFrom::Start((offset + offs) as u64))?;
        base.as_mut::<[u8; PAGE_SIZE]>().as_mut_slice()
    };

    /* populate one page, read as much as possible */
    while !page.is_empty() {
        match file.read(page) {
            Ok(0) => break,
            Ok(n) => page = &mut page[n..],
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }

    /* get the protection bits */
    let Some(prot) = prot else {
        return Ok(());
    };

    /* finalize the protection on this page */
    mem::protect(base, PAGE_SIZE, prot)?;
    Vm::protect(base, PAGE_SIZE, prot);
    Ok(())
}

pub fn fetch_page<F: Read + Seek>(
    pc: Uintptr,
    addr: Uintptr,
    base: Uintptr,
    file: &mut F,
    prot: Option<Protection>,
    offset: usize,
) {
    do_fetch_page(addr, base, file, prot, offset).unwrap_or_else(|err| {
        panic!(
            "cannot fetch page at {addr:p}\nInstruction:\n  {insn}\nError:\n  {err}",
            insn = disasm(pc)
        )
    });
}
