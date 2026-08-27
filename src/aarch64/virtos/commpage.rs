use crate::{
    aarch64::{
        disasm::disasm,
        paging::PAGE_SIZE,
        virtos::{
            mem::VmMap,
            mmio::{MmioKind, MmioRequest, MmioResponse},
        },
    },
    mem::Protection,
    utils::ptr::{Uintptr, VMA},
};

const COMMPAGE_RO: VMA = VMA::new(0xfffff4000);
const COMMPAGE_RW: VMA = VMA::new(0xfffffc000);
const COMMPAGE_TPRO: VMA = VMA::new(0xfffffc10c);

fn handle_commpage(pc: VMA, req: &mut MmioRequest) -> MmioResponse {
    if req.kind != MmioKind::Read {
        unimplemented!(
            "{kind:?} to COMMPAGE address {addr:p} at {insn}",
            kind = req.kind,
            addr = req.addr,
            insn = disasm(pc),
        );
    }
    if req.addr == COMMPAGE_TPRO {
        req.data = 0;
        return MmioResponse::Advance;
    }
    req.data = match req.size {
        1 => Uintptr::from(req.addr.addr()).read::<u8>() as u64,
        2 => Uintptr::from(req.addr.addr()).read::<u16>() as u64,
        4 => Uintptr::from(req.addr.addr()).read::<u32>() as u64,
        8 => Uintptr::from(req.addr.addr()).read(),
        n => unimplemented!("invalid COMMPAGE read size: {n}"),
    };
    MmioResponse::Advance
}

pub(super) fn init() {
    VmMap::insert(
        handle_commpage,
        Uintptr::from(COMMPAGE_RO.addr()),
        COMMPAGE_RO,
        PAGE_SIZE,
        Protection::READ,
        Protection::READ,
        true,
    );
    VmMap::insert(
        handle_commpage,
        Uintptr::from(COMMPAGE_RW.addr()),
        COMMPAGE_RW,
        PAGE_SIZE,
        Protection::RW,
        Protection::RW,
        true,
    );
}
