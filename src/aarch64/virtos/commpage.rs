use super::mmio::{self, MmioKind, MmioRequest, MmioResponse, MmioSize};
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
        MmioSize::Mem8 => req.addr.read::<u8>() as u64,
        MmioSize::Mem16 => req.addr.read::<u16>() as u64,
        MmioSize::Mem32 => req.addr.read::<u32>() as u64,
        MmioSize::Mem64 => req.addr.read(),
        MmioSize::Unknown => unimplemented!("unsized read of COMMPAGE"),
    };
    MmioResponse::Advance
}

pub(super) fn init() {
    mmio::register(COMMPAGE_RO, PAGE_SIZE, handle_commpage_reads);
    mmio::register(COMMPAGE_RW, PAGE_SIZE, handle_commpage_reads);
    PageTable::insert(COMMPAGE_RO, PAGE_SIZE, Protection::READ, Protection::READ);
    PageTable::insert(COMMPAGE_RW, PAGE_SIZE, Protection::READ, Protection::READ);
}
