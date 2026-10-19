use crate::{
    disasm::disasm,
    mem::Protection,
    paging::PAGE_SIZE,
    utils::ptr::Uintptr,
    virtos::{
        mem::VmMap,
        mmio::{MmioContext, MmioKind, MmioRequest},
    },
};

const COMMPAGE_RO: Uintptr = Uintptr::new(0xfffff4000);
const COMMPAGE_RW: Uintptr = Uintptr::new(0xfffffc000);
const COMMPAGE_TPRO: Uintptr = Uintptr::new(0xfffffc10c);

fn handle_commpage(mcx: MmioContext<'_>) -> bool {
    let req = mcx.decode();
    let MmioRequest { kind, size, .. } = req;

    /* COMMPAGE can only be read */
    if kind != MmioKind::Read {
        unimplemented!(
            "COMMPAGE {kind:?} to {addr:p}: {insn:?}",
            addr = mcx.addr,
            insn = disasm(mcx.pc),
        );
    }

    /* disable TPRO */
    if mcx.addr == COMMPAGE_TPRO {
        mcx.advance(0);
        return true;
    }

    /* read the bytes from COMMPAGE */
    let data = match size {
        1 => mcx.addr.read::<u8>() as u64,
        2 => mcx.addr.read::<u16>() as u64,
        4 => mcx.addr.read::<u32>() as u64,
        8 => mcx.addr.read(),
        n => unimplemented!("invalid COMMPAGE read size: {n}"),
    };

    /* reply the MMIO request */
    mcx.advance(data);
    true
}

pub(super) fn init() {
    VmMap::iomap(
        handle_commpage,
        COMMPAGE_RO,
        COMMPAGE_RO.into(),
        PAGE_SIZE,
        Protection::READ,
        Protection::READ,
        true,
    );
    VmMap::iomap(
        handle_commpage,
        COMMPAGE_RW,
        COMMPAGE_RW.into(),
        PAGE_SIZE,
        Protection::RW,
        Protection::RW,
        true,
    );
}
