use std::{
    fs::File,
    io::{Error as IoError, ErrorKind, Read, Result as IoResult, Seek, SeekFrom},
    os::fd::FromRawFd,
};

use parking_lot::Mutex;

use super::{
    HalProvider,
    mmio::{self, MmioHandler, MmioRequest, MmioResponse},
    tlb::TlbProvider,
};
use crate::{
    aarch64::{
        paging::{PAGE_SIZE, PageTable},
        vm::Vm,
    },
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

struct FileMap {
    file: Mutex<File>,
    base: Uintptr,
    prot: Protection,
    offset: usize,
    write_back: bool,
}

impl FileMap {
    fn new(addr: Uintptr, prot: Protection, fd: i32, pos: libc::off_t, flags: i32) -> Self {
        Self {
            prot,
            base: addr,
            file: unsafe { Mutex::new(File::from_raw_fd(libc::dup(fd))) },
            offset: pos as usize,
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
        let offs = (req.addr - self.base) & !(PAGE_SIZE - 1);
        let base = self.base + offs;

        /* sanity check the calculated address */
        debug_assert!(
            base <= req.addr && req.addr < base + PAGE_SIZE,
            "calculated page address {base:p} does not contain the requested address {addr:p}",
            addr = req.addr,
        );

        /* get the file handle */
        let mut buf = base.as_mut::<[u8; PAGE_SIZE]>().as_mut_slice();
        let mut file = self.file.lock();

        /* seek to the specified offset */
        file.seek(SeekFrom::Start((self.offset + offs) as u64))
            .unwrap_or_else(|err| panic!("cannot seek mapped file at PC={pc:p}: {err}"));

        /* populate one page, read as much as possible */
        while !buf.is_empty() {
            match file.read(buf) {
                Ok(0) => break,
                Ok(n) => buf = &mut buf[n..],
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => panic!("cannot read mapped file at PC={pc:p}: {e}"),
            }
        }

        /* enable read & write on this page */
        Vm::protect(base, PAGE_SIZE, self.prot);
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

pub fn mprotect(hal: &impl HalProvider, addr: Uintptr, len: usize, prot: i32) -> IoResult<()> {
    if let Some(prot) = Protection::from_bits(prot as u64) {
        let size = align_to_page(len);
        PageTable::protect(addr, size, prot, false)?;
        hal.flush_tlb(addr.as_u64(), size / PAGE_SIZE);
        Vm::protect(addr, size, prot);
        Ok(())
    } else {
        Err(IoError::from_raw_os_error(libc::EINVAL))
    }
}

pub fn mmap(
    hal: &impl HalProvider,
    addr: Uintptr,
    len: usize,
    prot: i32,
    flags: i32,
    mut fd: i32,
    pos: libc::off_t,
) -> IoResult<Uintptr> {
    let map_fd = fd;
    let map_flags = flags | libc::MAP_ANON;
    let mut map_prot = prot & !libc::PROT_EXEC;

    /* parse protection bits */
    let Some(prot) = Protection::from_bits(prot as u64) else {
        return Err(IoError::from_raw_os_error(libc::EINVAL));
    };

    /* no file mappings on host side, always map them as regular pages and populate later with
     * MMIO handlers */
    if flags & libc::MAP_ANON == 0 {
        if pos % (PAGE_SIZE as i64) != 0 {
            return Err(IoError::from_raw_os_error(libc::EINVAL));
        }
        map_prot = libc::PROT_READ | libc::PROT_WRITE;
        fd = 0;
    }

    /* always map as anonymous non-executable pages at host side (fd may have extra flags for
     * MAP_ANON, so keep them) */
    let addr = unsafe {
        match libc::mmap(addr.as_ptr(), len, map_prot, map_flags, fd, 0) {
            libc::MAP_FAILED => return Err(IoError::last_os_error()),
            mem => Uintptr::from(mem),
        }
    };

    /* align the size to page boundary */
    let size = align_to_page(len);
    let mut vm_prot = prot;

    /* register file mappings as MMIO */
    if flags & libc::MAP_ANON == 0 {
        let handler = FileMap::new(addr, prot, map_fd, pos, flags);
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
