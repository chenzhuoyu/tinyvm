use super::mmio::{self, MmioKind, MmioRequest, MmioSize};
use crate::{aarch64::paging::PageTable, mem::Protection, utils::ptr::Uintptr};

const COMMPAGE_END: Uintptr = Uintptr::new(0x1000000000);
const COMMPAGE_BEGIN: Uintptr = Uintptr::new(0xfffffc000);

const COMMPAGE_RO_END: Uintptr = Uintptr::new(0xfffff8000);
const COMMPAGE_RO_BEGIN: Uintptr = Uintptr::new(0xfffff4000);

fn handle_commpage_reads(pc: Uintptr, req: &mut MmioRequest) {
    if req.kind != MmioKind::Read {
        unimplemented!(
            "COMMPAGE {kind:?} to {addr:p} at {pc:?}",
            kind = req.kind,
            addr = req.addr
        );
    }
    req.data = match req.size {
        MmioSize::Mem8 => req.addr.read::<u8>() as u64,
        MmioSize::Mem16 => req.addr.read::<u16>() as u64,
        MmioSize::Mem32 => req.addr.read::<u32>() as u64,
        MmioSize::Mem64 => req.addr.read(),
    };
}

pub(super) fn init() {
    mmio::register(
        COMMPAGE_BEGIN,
        COMMPAGE_END - COMMPAGE_BEGIN,
        handle_commpage_reads,
    );
    mmio::register(
        COMMPAGE_RO_BEGIN,
        COMMPAGE_RO_END - COMMPAGE_RO_BEGIN,
        handle_commpage_reads,
    );
    PageTable::insert(
        COMMPAGE_BEGIN,
        COMMPAGE_BEGIN.as_u64(),
        COMMPAGE_END - COMMPAGE_BEGIN,
        Protection::READ,
    );
    PageTable::insert(
        COMMPAGE_RO_BEGIN,
        COMMPAGE_RO_BEGIN.as_u64(),
        COMMPAGE_RO_END - COMMPAGE_RO_BEGIN,
        Protection::READ,
    );
}
