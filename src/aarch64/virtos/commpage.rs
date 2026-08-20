use crate::{
    aarch64::{
        paging::PAGE_SIZE,
        virtos::{
            mem::VmMap,
            mmio::{MmioKind, MmioRequest, MmioResponse},
        },
    },
    mem::Protection,
    utils::ptr::Uintptr,
};

const COMMPAGE_RO: Uintptr = Uintptr::new(0xfffff4000);
const COMMPAGE_RW: Uintptr = Uintptr::new(0xfffffc000);
const COMMPAGE_TPRO: Uintptr = Uintptr::new(0xfffffc10c);

fn handle_commpage(pc: Uintptr, req: &mut MmioRequest) -> MmioResponse {
    if req.kind != MmioKind::Read {
        unimplemented!(
            "COMMPAGE {kind:?} to {addr:p} at {pc:?}",
            kind = req.kind,
            addr = req.addr
        );
    }
    if req.addr == COMMPAGE_TPRO {
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
    VmMap::insert(
        handle_commpage,
        COMMPAGE_RO,
        PAGE_SIZE,
        Protection::READ,
        Protection::READ,
        true,
    );
    VmMap::insert(
        handle_commpage,
        COMMPAGE_RW,
        PAGE_SIZE,
        Protection::RW,
        Protection::RW,
        true,
    );
}
