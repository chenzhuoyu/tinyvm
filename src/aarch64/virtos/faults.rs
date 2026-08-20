use std::io::{ErrorKind, Read, Result as IoResult, Seek, SeekFrom};

use crate::{
    aarch64::{disasm::disasm, paging::PAGE_SIZE, vm::Vm},
    mem::Protection,
    utils::ptr::Uintptr,
};

fn do_fetch_page<F: Read + Seek>(
    addr: Uintptr,
    base: Uintptr,
    prot: Protection,
    file: &mut F,
    offset: usize,
) -> IoResult<()> {
    let dist = (addr - base) & !(PAGE_SIZE - 1);
    let base = base + dist;

    /* sanity check the calculated address */
    debug_assert!(
        base <= addr && addr < base + PAGE_SIZE,
        "calculated page address {base:p} does not contain the requested address {addr:p}",
    );

    /* seek to the specified location */
    let mut page = {
        file.seek(SeekFrom::Start((offset + dist) as u64))?;
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

    // FIXME: it may need page table invalidations here
    /* map the loaded page into guest space */
    // VmMap::protect(cpu, base, PAGE_SIZE, prot, false)?;
    Vm::map(base, PAGE_SIZE, prot);
    Ok(())
}

pub fn fetch_page<F: Read + Seek>(
    pc: Uintptr,
    addr: Uintptr,
    base: Uintptr,
    prot: Protection,
    file: &mut F,
    offset: usize,
) {
    do_fetch_page(addr, base, prot, file, offset).unwrap_or_else(|err| {
        panic!(
            "cannot fetch page at {addr:p}\nInstruction:\n  {insn}\nError:\n  {err}",
            insn = disasm(pc)
        )
    });
}
