use super::mmio::{self, MmioKind, MmioRequest, MmioResponse};
use crate::{
    aarch64::paging::{PAGE_SIZE, PageTable},
    mem::Protection,
    utils::ptr::Uintptr,
};

const COMMPAGE_RO: Uintptr = Uintptr::new(0xfffff4000);
const COMMPAGE_RW: Uintptr = Uintptr::new(0xfffffc000);

const SLOT_TPRO: Uintptr = Uintptr::new(0xfffffc10c);

fn handle_commpage_reads(pc: Uintptr, req: &mut MmioRequest) -> MmioResponse {
    if req.kind != MmioKind::Read {
        unimplemented!(
            "COMMPAGE {kind:?} to {addr:p} at {pc:?}",
            kind = req.kind,
            addr = req.addr
        );
    }
    if req.addr == SLOT_TPRO {
        req.data = 0;
        return MmioResponse::Advance;
    }
    req.data = match req.size {
        1 => req.addr.read::<u8>() as u64,
        2 => req.addr.read::<u16>() as u64,
        4 => req.addr.read::<u32>() as u64,
        8 => req.addr.read(),
        n => unimplemented!("invalid COMMPAGE read size: {n}"),
    };
    MmioResponse::Advance
}

pub(super) fn init() {
    mmio::register(COMMPAGE_RO, PAGE_SIZE, handle_commpage_reads);
    mmio::register(COMMPAGE_RW, PAGE_SIZE, handle_commpage_reads);
    PageTable::map(COMMPAGE_RO, PAGE_SIZE, Protection::READ, Protection::READ);
    PageTable::map(COMMPAGE_RW, PAGE_SIZE, Protection::READ, Protection::READ);
}
