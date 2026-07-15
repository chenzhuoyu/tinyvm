use std::{
    io::{Error as IoError, Result as IoResult},
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use super::HalProvider;
use crate::aarch64::syscall::bsd::{shared_file_np, shared_mapping_np};

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
    mappings_u: *mut shared_mapping_np,
) -> IoResult<()> {
    let files = unsafe { std::slice::from_raw_parts(files, files_count as usize) };
    let mappings = unsafe { std::slice::from_raw_parts(mappings_u, mappings_count as usize) };

    /* empty file list or mappings list, nothing to do */
    if files.is_empty() || mappings.is_empty() {
        return Ok(());
    }

    /* get the mappings iterator */
    let miter = mappings.iter();
    let mut iter = miter.copied();

    /* process each file */
    for file in files {
        eprintln!("file={file:#?}");
        for _ in 0..file.sf_mappings_count {
            let map = iter.next().expect("no more mappings");
            for line in format!("map={map:#?}").lines() {
                eprintln!("    {line}");
            }
        }
    }

    assert!(iter.next().is_none(), "more mappings than needed");
    // todo!("shared_region_map_and_slide_2_np()");
    Ok(())
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
