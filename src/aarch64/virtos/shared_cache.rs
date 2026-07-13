use std::{
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::utils::ptr::Uintptr;

/// The global shared cache mappings
static SHARED_CACHE: AtomicPtr<()> = AtomicPtr::null();

pub fn shared_region_check_np(addr: Uintptr) -> i32 {
    if let Some(cache) = NonNull::new(SHARED_CACHE.load(Ordering::Acquire)) {
        if addr.is_nil() {
            todo!("remove shared cache");
        } else if addr.addr() == usize::MAX {
            todo!("seal shared cache");
        } else {
            addr.write(cache);
            libc::KERN_SUCCESS
        }
    } else {
        libc::EINVAL
    }
}
