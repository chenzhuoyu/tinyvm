use std::{
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::utils::ptr::Uintptr;

static SHARED_CACHE: AtomicPtr<()> = AtomicPtr::null();

pub fn shared_region_check_np(addr: Uintptr) -> i32 {
    if let Some(cache) = NonNull::new(SHARED_CACHE.load(Ordering::Acquire)) {
        if !addr.is_nil() {
            addr.write(cache);
        }
        libc::KERN_SUCCESS
    } else {
        libc::EINVAL
    }
}
