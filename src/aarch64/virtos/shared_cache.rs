use std::{
    collections::BTreeMap,
    fs::File,
    io::{Error as IoError, Result as IoResult},
    num::NonZeroU64,
    os::fd::{FromRawFd, OwnedFd},
};

use parking_lot::{Mutex, RwLock};

use super::{
    HalProvider, faults,
    mmio::{self, MmioHandler, MmioRequest, MmioResponse},
    tlb::TlbProvider,
};
use crate::{
    aarch64::{
        disasm::disasm,
        paging::{PAGE_SIZE, PageTable},
        syscall::bsd::{shared_file_np, shared_mapping_np},
        vm::Vm,
    },
    mem::{Addressable, Memory, Protection},
    utils::{
        ptr::Uintptr,
        size::{align_to_page, is_page_aligned},
    },
};

#[derive(Debug)]
struct Mappings {
    file: usize,
    size: usize,
    offs: usize,
}

impl Mappings {
    #[inline]
    fn new(id: usize, entry: shared_mapping_np) -> Self {
        Self {
            file: id,
            size: entry.sms_size as usize,
            offs: entry.sms_file_offset as usize,
        }
    }
}

#[derive(Debug)]
struct SharedRegion {
    start: Uintptr,
    files: Vec<Mutex<File>>,
    mappings: BTreeMap<Uintptr, Mappings>,
}

#[derive(Debug, Clone, Copy)]
struct SharedRegionData {
    addr: Uintptr,
    size: usize,
    seal: bool,
}

impl SharedRegionData {
    fn remove(&mut self) {
        Vm::unmap(self.addr.as_u64(), self.size);
    }

    fn replace(&mut self, addr: Uintptr, size: usize) {
        self.remove();
        self.addr = addr;
        self.size = size;
    }
}

static SHARED_REGION: RwLock<SharedRegionData> = {
    RwLock::new(SharedRegionData {
        addr: Uintptr::NIL,
        size: 0,
        seal: false,
    })
};

impl MmioHandler for SharedRegion {
    fn handle(&self, pc: Uintptr, req: &mut MmioRequest) -> MmioResponse {
        let (&base, map) = self
            .mappings
            .range(..=req.addr)
            .next_back()
            .unwrap_or_else(|| panic!("unmapped region at {p:p}: PC={pc:p}", p = req.addr));
        assert!(
            req.addr < base + map.size,
            "MMIO address {addr:p} landed in gaps between regions \
             {base:p}-{next:p}\nInstruction:\n  {insn}",
            addr = req.addr,
            next = base + map.size,
            insn = disasm(pc),
        );
        let prot = PageTable::lookup(req.addr).unwrap_or_else(|e| {
            panic!(
                "cannot lookup protection bits at {p:p}: [PC={pc:?}]: {e}",
                p = req.addr
            )
        });
        match faults::fetch_page(
            req.addr,
            base,
            &mut *self.files[map.file].lock(),
            prot,
            map.offs,
        ) {
            Err(e) => panic!("cannot fetch page at {p:p}: [PC={pc:?}]: {e}", p = req.addr),
            Ok(()) => MmioResponse::Retry,
        }
    }
}

#[inline]
fn mkslice<T>(data: *const T, len: u32) -> &'static [T] {
    unsafe { std::slice::from_raw_parts(data, len as usize) }
}

pub fn shared_region_check_np(_hal: &impl HalProvider, addr: *mut u64) -> IoResult<()> {
    if addr.is_null() {
        SHARED_REGION.write().remove();
        Ok(())
    } else if addr.addr() == usize::MAX {
        SHARED_REGION.write().seal = true;
        Ok(())
    } else if let Some(base) = NonZeroU64::new(SHARED_REGION.read().addr.as_u64()) {
        unsafe { *addr = base.get() };
        Ok(())
    } else {
        Err(IoError::from_raw_os_error(libc::EINVAL))
    }
}

pub fn shared_region_map_and_slide_2_np(
    hal: &impl HalProvider,
    files_count: u32,
    files: *mut shared_file_np,
    mappings_count: u32,
    mappings_u: *mut shared_mapping_np,
) -> IoResult<()> {
    let mut max_virt = 0usize;
    let mut min_virt = usize::MAX;

    /* empty file list or mappings list, nothing to do */
    if files_count == 0 || mappings_count == 0 {
        return Ok(());
    }

    /* lock shared region */
    let files = mkslice(files, files_count);
    let mappings = mkslice(mappings_u, mappings_count);
    let mut shared_data = SHARED_REGION.write();

    /* create a new shared region */
    let mut region = SharedRegion {
        start: Uintptr::NIL,
        files: Vec::with_capacity(files.len()),
        mappings: BTreeMap::new(),
    };

    /* calculate virtual address range */
    for map in mappings {
        min_virt = min_virt.min(map.sms_address.addr());
        max_virt = max_virt.max(map.sms_address.addr() + (map.sms_size as usize));
    }

    /* check virtual address range */
    assert!(
        min_virt < max_virt,
        "mapping shared region with empty virtual address range"
    );

    /* allocate a block of memory with no access to guest to use MMIO as an on-demand page-in
     * mechanism */
    let size = align_to_page(max_virt - min_virt);
    let block = Memory::alloc(size).map(Protection::NONE);
    let mut map_iter = mappings.iter().copied();

    /* process each file */
    for (i, file) in files.iter().enumerate() {
        let fd = file.sf_fd;
        let num_mappings = file.sf_mappings_count;

        /* no intermediate slides */
        assert!(
            i == 0 || file.sf_slide == 0,
            "non-zero slide in the middle of shared region mappings"
        );

        /* nothing to map */
        if num_mappings == 0 {
            continue;
        }

        /* self-mappings only allow one map per file */
        if fd == -1 && num_mappings != 1 {
            return Err(IoError::from_raw_os_error(libc::EINVAL));
        }

        /* process each mappings */
        for _ in 0..num_mappings {
            if let Some(map) = map_iter.next() {
                let size = map.sms_size as usize;
                let addr = block.addr() + (map.sms_address.addr() - min_virt);
                let prot = Protection::from_bits_truncate(map.sms_init_prot as u64);
                eprintln!("mappings(): {addr:p}-{next:p} fd={fd}", next = addr + size);

                /* add to guest page table */
                PageTable::insert(
                    addr,
                    size,
                    prot,
                    Protection::from_bits_truncate(map.sms_max_prot as u64),
                );

                /* we need to load the page immediately if map from self */
                if fd == -1 {
                    if is_page_aligned(size) && addr.is_aligned_to(PAGE_SIZE) {
                        unsafe {
                            let src = map.sms_file_offset as *const u8;
                            std::ptr::copy_nonoverlapping(src, addr.as_ptr(), size);
                            Vm::protect(addr, size, prot);
                        }
                    } else {
                        return Err(IoError::from_raw_os_error(libc::EINVAL));
                    }
                } else {
                    region
                        .mappings
                        .insert(addr, Mappings::new(region.files.len(), map));
                }
            } else {
                panic!("no more mappings");
            }
        }

        /* add the file if needed */
        if fd != -1 {
            let fd = unsafe { OwnedFd::from_raw_fd(libc::dup(fd)) };
            region.files.push(Mutex::new(File::from(fd)));
        }
    }

    /* check the mappings count */
    assert!(
        map_iter.next().is_none(),
        "there are more mappings to map than required by files"
    );

    /* register the shared region */
    region.start = block.into_parts().0;
    shared_data.replace(region.start, size);

    /* flusth TLB and add the shared region to MMIO */
    hal.flush_tlb(region.start.as_u64(), size / PAGE_SIZE);
    mmio::register(region.start, size, region);
    Ok(())
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]

struct MwlRegion {
    mwlr_fd: i32,
    mwlr_protections: libc::vm_prot_t,
    mwlr_file_offset: u64,
    mwlr_address: Uintptr,
    mwlr_size: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct MwlInfoHeader {
    mwli_version: u32,
    mwli_page_size: u16,
    mwli_pointer_format: u16,
    mwli_binds_offset: u32,
    mwli_binds_count: u32,
    mwli_chains_offset: u32,
    mwli_chains_size: u32,
    mwli_slide: u64,
    mwli_image_address: Uintptr,
}

impl MwlInfoHeader {
    const VERSION: u32 = 7;
}

pub fn map_with_linking_np(
    _hal: &impl HalProvider,
    regions: *mut libc::c_void,
    region_count: u32,
    link_info: *mut libc::c_void,
    link_info_size: u32,
) -> IoResult<()> {
    let regions = mkslice(regions as *const MwlRegion, region_count);
    let mwli_hdr = unsafe { &*(link_info as *const MwlInfoHeader) };

    /* version check */
    if mwli_hdr.mwli_version != MwlInfoHeader::VERSION {
        return Err(IoError::from_raw_os_error(libc::EINVAL));
    }

    dbg!(regions);
    dbg!(mwli_hdr);
    dbg!(link_info_size);

    // TODO: it seems that this is really needed, the userspace implementation in dyld seems broken.
    Err(IoError::from_raw_os_error(libc::ENOSYS))
}
