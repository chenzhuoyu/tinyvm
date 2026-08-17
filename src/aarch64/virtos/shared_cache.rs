use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{Error as IoError, Result as IoResult},
    num::NonZeroU64,
    os::fd::{FromRawFd, OwnedFd},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use parking_lot::{Mutex, RwLock};
use smallvec::SmallVec;

use super::{
    HalProvider, faults, mem,
    mmio::{self, MmioHandler, MmioRequest, MmioResponse},
    pac::SigningKey,
    tlb::TlbProvider,
};
use crate::{
    aarch64::{
        disasm::disasm,
        paging::{PAGE_SIZE, PageTable},
        syscall::bsd::{shared_file_np, shared_mapping_np},
        vm::Vm,
    },
    macros::define_bit_field,
    mem::Protection,
    utils::{
        io::MemoryIo,
        path::is_real_file,
        ptr::Uintptr,
        size::{align_to_page, is_page_aligned},
    },
};

const VM_PROT_ZF: i32 = 0x10;
const VM_PROT_SLIDE: i32 = 0x20;

define_bit_field! {
    struct Rebase : u64 {
        runtime_offset : 34,
        high8          :  8,
        unused         : 10,
        next           : 11,    // 8-byte stride
        auth           :  1,    // == 0
    }

    struct EncodedPtr : u64 {
        runtime_offset : 34,
        high8          :  8,
        div_high8      :  8,
        addr_div       :  1,
        key_is_data    :  1,    // implicitly always the 'A' key. 0: IA, 1: DA
        next           : 11,    // 8-byte stride
        auth           :  1,    // == 1
    }
}

impl EncodedPtr {
    #[inline]
    const fn diversity(self) -> u64 {
        (self.div_high8() << 8) | self.high8()
    }
}

#[derive(Debug)]
struct SliderPage {
    addr: Uintptr,
    lock: Mutex<()>,
    flag: AtomicBool,
}

impl SliderPage {
    fn new(addr: Uintptr) -> Arc<Self> {
        Arc::new(Self {
            addr,
            lock: Mutex::new(()),
            flag: AtomicBool::new(false),
        })
    }
}

impl SliderPage {
    fn do_load(&self, pc: Uintptr) {
        if !self.flag.load(Ordering::Acquire) {
            let mut req = MmioRequest::read_unsized(self.addr);
            assert_eq!(mmio::dispatch(pc, &mut req), Some(MmioResponse::Retry));
            self.flag.store(true, Ordering::Release);
        }
    }

    fn load(&self, pc: Uintptr) {
        if !self.flag.load(Ordering::Acquire) {
            let _m = self.lock.lock();
            self.do_load(pc);
        }
    }
}

#[derive(Debug)]
struct CacheSlider {
    size: usize,
    offs: usize,
    data: Uintptr,
    page: SmallVec<[Arc<SliderPage>; 1]>,
    flag: AtomicBool,
}

impl CacheSlider {
    #[inline(always)]
    const fn page(addr: Uintptr) -> Uintptr {
        Uintptr::new(addr.addr() & !(PAGE_SIZE - 1))
    }
}

impl CacheSlider {
    #[inline]
    fn new(data: Uintptr, size: usize, slide: usize, page: SmallVec<[Arc<SliderPage>; 1]>) -> Self {
        Self {
            size,
            data,
            page,
            offs: slide,
            flag: AtomicBool::new(false),
        }
    }
}

impl CacheSlider {
    fn slide_v5(&self, page: Uintptr, base: Uintptr) {
        #[repr(C)]
        #[derive(Debug, Clone, Copy)]
        struct Header {
            version: u32,
            page_size: u32,
            page_start_count: u32,
            value_add: u64,
        }

        /* read the header */
        let mut mio = MemoryIo::new(self.data, self.size);
        let hdr = mio.read::<Header>().expect("cannot read slide header");
        let pos = (page - base) / PAGE_SIZE;

        /* check header version & page size */
        assert_eq!(hdr.version, 5);
        assert_eq!(hdr.page_size as usize, PAGE_SIZE);

        /* skip to the starting position */
        let offs = mio
            .read_at::<u16>(pos * std::mem::size_of::<u16>())
            .expect("slide info indexing out of bounds");

        /* no sliding needed in this page */
        if offs == u16::MAX {
            return;
        }

        /* calculate the sliding start and end */
        let end = page + PAGE_SIZE;
        let mut slot = page + (offs as usize);

        /* slide each pointers */
        while slot < end {
            let item = slot.read::<EncodedPtr>();
            let addr = item.runtime_offset() + hdr.value_add + (self.offs as u64);
            let next = item.next() * 8;

            /* sign the pointer if needed */
            if item.auth() != 0 {
                slot.write(SigningKey::from(item.key_is_data() << 1).sign(
                    Uintptr::from(addr),
                    slot,
                    item.addr_div() != 0,
                    item.diversity() as u16,
                ));
            } else {
                let high8 = item.high8() << 56;
                slot.write::<u64>(high8 | addr);
            }

            /* move to the next slot */
            if next != 0 {
                slot += next;
            } else {
                return;
            }
        }

        /* should not occure */
        panic!(
            "sliding pointers at {slot:p} beyound the page boundary {page:p}-{end:p} with sliding \
             info at {si:p}-{se:p}",
            si = self.data,
            se = self.data + self.size,
        );
    }

    fn slide(&self, pc: Uintptr, page: Uintptr, base: Uintptr) {
        if !self.flag.load(Ordering::Acquire) {
            self.page.iter().for_each(|p| p.load(pc));
            self.flag.store(true, Ordering::Release);
        }
        match self.data.read::<u32>() {
            1 => unimplemented!("slide info v1"),
            2 => unimplemented!("slide info v2"),
            3 => unimplemented!("slide info v3"),
            4 => unimplemented!("slide info v4"),
            5 => self.slide_v5(page, base),
            v => panic!("unknown slide info version {v}"),
        };
    }
}

#[derive(Debug)]
struct Mappings {
    file: usize,
    entry: shared_mapping_np,
    slider: Option<CacheSlider>,
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
        if !self.addr.is_nil() {
            Vm::unmap(self.addr, self.size);
            Vm::dealloc(self.addr, self.size);
        }
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

impl SharedRegion {
    fn protect(&self, addr: Uintptr) -> IoResult<()> {
        let page = CacheSlider::page(addr);
        let prot = PageTable::lookup(addr);
        mem::protect(page, PAGE_SIZE, prot)?;
        Vm::protect(page, PAGE_SIZE, prot);
        Ok(())
    }
}

impl MmioHandler for SharedRegion {
    fn handle(&self, pc: Uintptr, req: &mut MmioRequest) -> MmioResponse {
        let (&base, map) = self
            .mappings
            .range(..=req.addr)
            .next_back()
            .unwrap_or_else(|| panic!("unmapped region at {p:p}: PC={pc:p}", p = req.addr));
        assert!(
            req.addr < base + map.entry.sms_size,
            "MMIO address {addr:p} landed in gaps between regions \
             {base:p}-{next:p}\nInstruction:\n  {insn}",
            addr = req.addr,
            next = base + map.entry.sms_size,
            insn = disasm(pc),
        );
        if map.entry.sms_max_prot & VM_PROT_ZF != 0 {
            unimplemented!("VM_PROT_ZF for shared cache mappings");
        }
        let prot = {
            if map.slider.is_none() {
                Some(PageTable::lookup(req.addr))
            } else {
                None
            }
        };
        faults::fetch_page(
            pc,
            req.addr,
            base,
            &mut *self.files[map.file].lock(),
            prot,
            map.entry.sms_file_offset as usize,
        );
        if let Some(slider) = map.slider.as_ref() {
            slider.slide(pc, CacheSlider::page(req.addr), base);
            self.protect(req.addr).expect("cannot protect memory");
        }
        MmioResponse::Retry
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

    /* verify file descriptors */
    for fp in files {
        if fp.sf_fd != -1 && !is_real_file(fp.sf_fd) {
            return Err(IoError::from_raw_os_error(libc::EINVAL));
        }
    }

    /* create a new shared region */
    let mut region = SharedRegion {
        start: Uintptr::NIL,
        files: Vec::with_capacity(files.len()),
        mappings: BTreeMap::new(),
    };

    /* calculate virtual address range, while validating the input */
    for map in mappings {
        if map.sms_slide_size > PAGE_SIZE {
            unimplemented!(
                "shared cache mappings slide info does not fit into one page: {size}",
                size = map.sms_slide_size,
            );
        }
        if map.sms_slide_start.is_nil() != (map.sms_max_prot & VM_PROT_SLIDE == 0) {
            return Err(IoError::from_raw_os_error(libc::EINVAL));
        }
        min_virt = min_virt.min(map.sms_address.addr());
        max_virt = max_virt.max(map.sms_address.addr() + (map.sms_size as usize));
    }

    /* check virtual address range */
    assert!(
        min_virt < max_virt,
        "mapping shared region with empty virtual address range"
    );

    /* allocate a block of memory without mapping to guest space to use MMIO as an on-demand
     * page-in mechanism, and calculate the ASLR slide */
    let block = Vm::alloc(align_to_page(max_virt - min_virt));
    let slide = block.addr().wrapping_sub(min_virt);

    /* iterate over mappings */
    let mut pages = HashMap::new();
    let mut miter = mappings.iter().copied();

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
            let item = miter.next().expect("no more mappings");
            let prot = Protection::from_bits_truncate(item.sms_init_prot as u64);
            let addr = item.sms_address + slide;
            let size = item.sms_size as usize;

            /* add to guest page table */
            PageTable::map(
                addr,
                addr,
                size,
                prot,
                Protection::from_bits_truncate(item.sms_max_prot as u64),
            );

            /* we need to load the page immediately if map from self */
            if fd == -1 {
                if is_page_aligned(size) && addr.is_aligned_to(PAGE_SIZE) {
                    unsafe {
                        let src = item.sms_file_offset as *const u8;
                        std::ptr::copy_nonoverlapping(src, addr.as_ptr(), size);
                        Vm::map(addr, size, prot);
                        continue;
                    }
                } else {
                    return Err(IoError::from_raw_os_error(libc::EINVAL));
                }
            }

            /* create the cache slider, if needed */
            let slider = {
                if item.sms_max_prot & VM_PROT_SLIDE != 0 {
                    let ptr = item.sms_slide_start + slide;
                    let end = CacheSlider::page(ptr + item.sms_slide_size - 1);
                    let mut pos = CacheSlider::page(ptr);
                    let mut page = SmallVec::new();

                    /* collect all slider info pages */
                    while pos <= end {
                        let item = pages
                            .entry(pos)
                            .or_insert_with_key(|&addr| SliderPage::new(addr))
                            .clone();
                        page.push(item);
                        pos += PAGE_SIZE;
                    }

                    /* create the slider */
                    Some(CacheSlider::new(
                        item.sms_slide_start + slide,
                        item.sms_slide_size,
                        slide,
                        page,
                    ))
                } else {
                    None
                }
            };

            /* register the mapping into shared region */
            region.mappings.insert(addr, {
                Mappings {
                    file: region.files.len(),
                    entry: item,
                    slider,
                }
            });
        }

        /* add the file if needed */
        if fd != -1 {
            let fd = unsafe { OwnedFd::from_raw_fd(libc::dup(fd)) };
            region.files.push(Mutex::new(File::from(fd)));
        }
    }

    /* check the mappings count */
    assert!(
        miter.next().is_none(),
        "there are more mappings to process than those required by files"
    );

    /* get the address and size of the memory block */
    let size = align_to_page(max_virt - min_virt);
    let num_pages = size / PAGE_SIZE;

    /* register the shared region */
    region.start = block;
    shared_data.replace(block, size);

    /* flusth TLB and add the shared region to MMIO */
    hal.flush_tlb(region.start.as_u64(), num_pages);
    mmio::map(region.start, size, region);
    Ok(())
}

pub fn map_with_linking_np(
    _hal: &impl HalProvider,
    _regions: *mut libc::c_void,
    _region_count: u32,
    _link_info: *mut libc::c_void,
    _link_info_size: u32,
) -> IoResult<()> {
    Err(IoError::from_raw_os_error(libc::ENOSYS))
}
