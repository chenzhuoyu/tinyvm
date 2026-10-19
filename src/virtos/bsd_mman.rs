use std::{
    fs::File,
    io::{Error as IoError, Result as IoResult},
    os::fd::{FromRawFd, OwnedFd},
};

use parking_lot::Mutex;

use crate::{
    cpu::Cpu,
    mem::Protection,
    paging::PAGE_SIZE,
    utils::{ptr::Uintptr, size::align_to_page},
    virtos::{
        mem::VmMap,
        mmio::{MmioContext, MmioHandler},
    },
};

struct FileMap {
    size: usize,
    host: Uintptr,
    guest: Uintptr,
    write_back: bool,
}

impl MmioHandler for FileMap {
    fn handle(&self, mcx: MmioContext<'_>) -> bool {
        todo!()
    }
}

fn sys_map(
    addr: Uintptr,
    size: usize,
    prot: Protection,
    flags: i32,
    fd: i32,
    offset: i64,
) -> IoResult<Uintptr> {
    unsafe {
        match libc::mmap(addr.as_ptr(), size, prot.bits(), flags, fd, offset) {
            libc::MAP_FAILED => Err(IoError::last_os_error()),
            mem => Ok(Uintptr::from(mem)),
        }
    }
}

fn sys_protect(addr: Uintptr, prot: Protection) -> IoResult<()> {
    if unsafe { libc::mprotect(addr.as_ptr(), PAGE_SIZE, prot.bits()) } != 0 {
        Err(IoError::last_os_error())
    } else {
        Ok(())
    }
}

fn map_regular(
    addr: Uintptr,
    size: usize,
    prot: Protection,
    flags: i32,
    fd: i32,
) -> IoResult<Uintptr> {
    let addr = sys_map(addr, size, prot, flags, fd, 0)?;
    VmMap::mmap(addr, size, prot, Protection::all(), false);
    Ok(addr)
}

fn map_from_file(
    addr: Uintptr,
    size: usize,
    flags: i32,
    prot: Protection,
    fd: i32,
    offset: libc::off_t,
) -> IoResult<Uintptr> {
    let addr = unsafe {
        Uintptr::from(libc::mmap(
            addr.as_ptr(),
            size,
            prot.bits(),
            flags,
            fd,
            offset,
        ))
    };
    if addr.as_ptr() == libc::MAP_FAILED {
        return Err(IoError::last_os_error());
    }
    eprintln!("addr={addr:p} size={size} flags=0x{flags:x} prot={prot:?} fd={fd} offset={offset}");
    eprintln!("  data = {:?}", addr.read::<[u8; 32]>());
    todo!();
    // VmMap::iomap(
    //     FileMap::new(addr, fd, flags, offset),
    //     addr,
    //     size,
    //     prot,
    //     Protection::all(),
    //     false,
    // );
    // Ok(addr)
}

pub fn msync(_cpu: &Cpu, addr: Uintptr, len: usize, flags: i32) -> IoResult<()> {
    todo!("msync(): addr={addr:p} len={len} flags=0x{flags:x}");
}

pub fn munmap(cpu: &Cpu, addr: Uintptr, len: usize) -> IoResult<()> {
    VmMap::unmap(addr, len);
    cpu.flush_tlb(addr, len.div_ceil(PAGE_SIZE));

    /* actually remove from host address space */
    if unsafe { libc::munmap(addr.as_ptr(), len) } != 0 {
        Err(IoError::last_os_error())
    } else {
        Ok(())
    }
}

pub fn mprotect(cpu: &Cpu, addr: Uintptr, len: usize, raw_prot: i32) -> IoResult<()> {
    if let Some(prot) = Protection::from_bits(raw_prot) {
        std::thread::sleep(std::time::Duration::from_secs(1));
        sys_protect(addr, prot & !Protection::EXEC)?;
        VmMap::protect(cpu, addr, len, prot, false)?;
        Ok(())
    } else {
        Err(IoError::from_raw_os_error(libc::EINVAL))
    }
}

pub fn mmap(
    _cpu: &Cpu,
    addr: Uintptr,
    len: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: libc::off_t,
) -> IoResult<Uintptr> {
    let size = {
        if len == 0 {
            return Err(IoError::from_raw_os_error(libc::EINVAL));
        } else {
            align_to_page(len)
        }
    };
    let Some(prot) = Protection::from_bits(prot) else {
        return Err(IoError::from_raw_os_error(libc::EINVAL));
    };
    if flags & libc::MAP_ANON != 0 {
        map_regular(addr, size, prot, flags, fd)
    } else if fd >= 0 {
        map_from_file(addr, size, flags, prot, fd, offset)
    } else {
        Err(IoError::from_raw_os_error(libc::EINVAL))
    }
}

pub fn msync_nocancel(_cpu: &Cpu, addr: Uintptr, len: usize, flags: i32) -> IoResult<()> {
    todo!("msync_nocancel(): addr={addr:p} len={len} flags=0x{flags:x}");
}
