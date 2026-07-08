use crate::utils::ptr::Uintptr;

pub const COMMPAGE_END: Uintptr = Uintptr::new(0x1000000000);
pub const COMMPAGE_BEGIN: Uintptr = Uintptr::new(0xfffffc000);

pub const COMMPAGE_RO_END: Uintptr = Uintptr::new(0xfffff8000);
pub const COMMPAGE_RO_BEGIN: Uintptr = Uintptr::new(0xfffff4000);

#[inline]
pub fn is_commpage_addr(addr: Uintptr) -> bool {
    addr < COMMPAGE_END && addr >= COMMPAGE_BEGIN
        || addr < COMMPAGE_RO_END && addr >= COMMPAGE_RO_BEGIN
}
