use std::{
    io::{Error as IoError, Result as IoResult},
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use super::HalProvider;
use crate::aarch64::syscall::bsd::{shared_file_mapping_slide_np_ut, shared_file_np};

/// The global shared cache mappings
static SHARED_CACHE: AtomicPtr<()> = AtomicPtr::null();

pub fn shared_region_check_np(_hal: &impl HalProvider, addr: *mut u64) -> IoResult<()> {
    if let Some(cache) = NonNull::new(SHARED_CACHE.load(Ordering::Acquire)) {
        if addr.is_null() {
            todo!("remove shared cache");
        } else if addr.addr() == usize::MAX {
            todo!("seal shared cache");
        } else {
            unsafe { *addr = cache.as_ptr() as u64 }
            Ok(())
        }
    } else {
        Err(IoError::from_raw_os_error(libc::EINVAL))
    }
}

pub fn shared_region_map_and_slide_2_np(
    _hal: &impl HalProvider,
    files_count: u32,
    files: *mut shared_file_np,
    mappings_count: u32,
    mappings_u: *mut shared_file_mapping_slide_np_ut,
) -> IoResult<()> {
    todo!(
        "shared_region_map_and_slide_2_np(files_count={files_count}, files={files:p}, \
         mappings_count={mappings_count}, mappings_u={mappings_u:p})"
    );
}

pub fn map_with_linking_np(
    _hal: &impl HalProvider,
    regions: *mut libc::c_void,
    region_count: u32,
    link_info: *mut libc::c_void,
    link_info_size: u32,
) -> IoResult<()> {
    todo!(
        "map_with_linking_np(regions={regions:p}, region_count={region_count}, \
         link_info={link_info:p}, link_info_size={link_info_size})"
    );
}
