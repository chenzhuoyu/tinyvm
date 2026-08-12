use std::{
    fs::File,
    io::{Error as IoError, Result as IoResult},
    os::fd::{FromRawFd, OwnedFd},
};

use parking_lot::Mutex;

use super::{
    HalProvider, faults, mem,
    mmio::{self, MmioHandler, MmioRequest, MmioResponse},
    tlb::TlbProvider,
};
use crate::{
    aarch64::{
        paging::{PAGE_SIZE, PageTable},
        vm::Vm,
    },
    mem::Protection,
    utils::{path::is_real_file, ptr::Uintptr, size::align_to_page},
};

struct FileMap {
    file: Mutex<File>,
    base: Uintptr,
    offset: usize,
    write_back: bool,
}

impl FileMap {
    fn new(addr: Uintptr, fd: i32, flags: i32, offset: libc::off_t) -> Self {
        Self {
            base: addr,
            file: unsafe { Mutex::new(File::from(OwnedFd::from_raw_fd(libc::dup(fd)))) },
            offset: offset as usize,
            write_back: flags & libc::MAP_SHARED != 0,
        }
    }
}

impl Drop for FileMap {
    fn drop(&mut self) {
        if self.write_back {
            unimplemented!("file map write back");
        }
    }
}

impl MmioHandler for FileMap {
    fn handle(&self, pc: Uintptr, req: &mut MmioRequest) -> MmioResponse {
        let prot = PageTable::lookup(req.addr);
        let mut file = self.file.lock();
        faults::fetch_page(pc, req.addr, self.base, &mut *file, Some(prot), self.offset);
        MmioResponse::Retry
    }
}

pub fn msync(_hal: &impl HalProvider, addr: Uintptr, len: usize, flags: i32) -> IoResult<()> {
    todo!("msync(): addr={addr:p} len={len} flags=0x{flags:x}");
}

pub fn munmap(hal: &impl HalProvider, addr: Uintptr, len: usize) -> IoResult<()> {
    let base = addr.as_u64();
    let size = align_to_page(len);
    let num_pages = size / PAGE_SIZE;

    /* remove from page table and such */
    PageTable::unmap(addr, len)?;
    mmio::unregister(addr, size);
    hal.flush_tlb(base, num_pages);
    Vm::unmap(base, size);

    /* actually remove from host address space */
    if unsafe { libc::munmap(addr.as_ptr(), len) } != 0 {
        Err(IoError::last_os_error())
    } else {
        Ok(())
    }
}

pub fn mprotect(hal: &impl HalProvider, addr: Uintptr, len: usize, raw_prot: i32) -> IoResult<()> {
    let base = addr.as_u64();
    let size = align_to_page(len);
    let num_pages = size / PAGE_SIZE;

    /* parse protection bits */
    let Some(prot) = Protection::from_bits(raw_prot as u64) else {
        return Err(IoError::from_raw_os_error(libc::EINVAL));
    };

    /* modify the page table, then actually modify the host address space */
    PageTable::protect(addr, size, prot, false)?;
    mmio::protect(addr, size, prot)?;
    mem::protect(addr, size, prot)?;
    hal.flush_tlb(base, num_pages);
    Ok(())
}

pub fn mmap(
    hal: &impl HalProvider,
    addr: Uintptr,
    len: usize,
    prot: i32,
    flags: i32,
    mut fd: i32,
    offset: libc::off_t,
) -> IoResult<Uintptr> {
    let map_fd = fd;
    let map_flags = flags | libc::MAP_ANON;
    let mut map_prot = prot & !libc::PROT_EXEC;

    /* parse protection bits */
    let Some(prot) = Protection::from_bits(prot as u64) else {
        return Err(IoError::from_raw_os_error(libc::EINVAL));
    };

    /* only map real files with MMIO */
    if !is_real_file(fd) {
        let bits = prot.bits() as i32;
        let size = align_to_page(len);

        let addr = unsafe {
            match libc::mmap(addr.as_ptr(), len, bits, flags, fd, offset) {
                libc::MAP_FAILED => return Err(IoError::last_os_error()),
                mem => Uintptr::from(mem),
            }
        };

        Vm::map(addr, size, prot);
        PageTable::insert(addr, size, prot, Protection::all());
        hal.flush_tlb(addr.as_u64(), size / PAGE_SIZE);
        return Ok(addr);
    }

    /* no file mappings on host side, always map them as regular pages and populate later with
     * MMIO handlers */
    if flags & libc::MAP_ANON == 0 {
        if offset % (PAGE_SIZE as i64) != 0 {
            return Err(IoError::from_raw_os_error(libc::EINVAL));
        }
        map_prot = libc::PROT_NONE;
        fd = 0;
    }

    /* always map as anonymous non-executable pages at host side (fd may have extra flags for
     * MAP_ANON, so keep them) */
    let addr = unsafe {
        match libc::mmap(addr.as_ptr(), len, map_prot, map_flags, fd, offset) {
            libc::MAP_FAILED => return Err(IoError::last_os_error()),
            mem => Uintptr::from(mem),
        }
    };

    /* align the size to page boundary */
    let size = align_to_page(len);
    let mut vm_prot = prot;

    /* register file mappings as MMIO */
    if flags & libc::MAP_ANON == 0 {
        let handler = FileMap::new(addr, map_fd, flags, offset);
        mmio::register(addr, size, handler);
        vm_prot = Protection::NONE;
    }

    /* insert into page table */
    Vm::map(addr, size, vm_prot);
    PageTable::insert(addr, size, prot, Protection::all());
    hal.flush_tlb(addr.as_u64(), size / PAGE_SIZE);
    Ok(addr)
}

pub fn msync_nocancel(
    _hal: &impl HalProvider,
    addr: Uintptr,
    len: usize,
    flags: i32,
) -> IoResult<()> {
    todo!("msync_nocancel(): addr={addr:p} len={len} flags=0x{flags:x}");
}
