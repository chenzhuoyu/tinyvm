use std::{
    fs::File,
    io::{Error as IoError, Result as IoResult},
    os::fd::{FromRawFd, OwnedFd},
};

use parking_lot::Mutex;

use crate::{
    aarch64::{
        cpu::Cpu,
        disasm::disasm,
        paging::PAGE_SIZE,
        virtos::{
            faults,
            mem::{VmKind, VmMap},
            mmio::{MmioHandler, MmioKind, MmioRequest, MmioResponse},
        },
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
        let prot = VmMap::lookup(req.addr);
        let mut file = self.file.lock();
        faults::fetch_page(pc, req.addr, self.base, prot, &mut *file, self.offset);
        MmioResponse::Retry
    }
}

struct ObjectMap {
    size: usize,
    addr: Uintptr,
}

impl ObjectMap {
    #[inline]
    fn map(addr: Uintptr, size: usize) -> Self {
        Self { size, addr }
    }
}

impl Drop for ObjectMap {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.addr.as_ptr(), self.size) };
    }
}

impl MmioHandler for ObjectMap {
    fn handle(&self, pc: Uintptr, req: &mut MmioRequest) -> MmioResponse {
        assert!(
            req.addr >= self.addr && req.addr + req.size <= self.addr + self.size,
            "MMIO access out of range"
        );
        match req.kind {
            MmioKind::Read => {
                req.data = match req.size {
                    1 => req.addr.read::<u8>() as u64,
                    2 => req.addr.read::<u16>() as u64,
                    4 => req.addr.read::<u32>() as u64,
                    8 => req.addr.read::<u64>(),
                    n => unimplemented!("read {n} bytes shm: {insn}", insn = disasm(pc)),
                };
            }
            MmioKind::ReadAtomic => {
                unimplemented!("atomic read from shared memory: {insn}", insn = disasm(pc));
            }
            MmioKind::Write => {
                unimplemented!("write to shared memory: {insn}", insn = disasm(pc));
            }
            MmioKind::WriteAtomic => {
                unimplemented!("atomic write to shared memory: {insn}", insn = disasm(pc));
            }
            MmioKind::Execution => {
                unimplemented!("execution on shared memory: {insn}", insn = disasm(pc));
            }
        }
        MmioResponse::Advance
    }
}

fn sys_protect(addr: Uintptr, prot: Protection) -> IoResult<()> {
    if unsafe { libc::mprotect(addr.as_ptr(), PAGE_SIZE, prot.bits() as i32) } != 0 {
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
    let addr = unsafe {
        match libc::mmap(addr.as_ptr(), size, prot.bits() as i32, flags, fd, 0) {
            libc::MAP_FAILED => return Err(IoError::last_os_error()),
            mem => Uintptr::from(mem),
        }
    };
    VmMap::map(VmKind::Regular, addr, size, prot, Protection::all(), false);
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
    if offset % (PAGE_SIZE as i64) != 0 {
        return Err(IoError::from_raw_os_error(libc::EINVAL));
    }
    let addr = unsafe {
        Uintptr::from(libc::mmap(
            addr.as_ptr(),
            size,
            libc::PROT_NONE,
            libc::MAP_PRIVATE | libc::MAP_ANON,
            -1,
            0,
        ))
    };
    if addr.as_ptr() == libc::MAP_FAILED {
        return Err(IoError::last_os_error());
    }
    VmMap::map(
        FileMap::new(addr, fd, flags, offset),
        addr,
        size,
        prot,
        Protection::all(),
        false,
    );
    Ok(addr)
}

fn map_from_object(
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
            prot.bits() as i32,
            flags,
            fd,
            offset,
        ))
    };
    if addr.as_ptr() == libc::MAP_FAILED {
        return Err(IoError::last_os_error());
    }
    VmMap::map(
        ObjectMap::map(addr, size),
        addr,
        size,
        prot,
        Protection::all(),
        false,
    );
    Ok(addr)
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
    if let Some(prot) = Protection::from_bits(raw_prot as u64) {
        sys_protect(addr, prot & !Protection::EXEC)?;
        VmMap::protect(cpu, addr, len, prot, false)
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
    let Some(prot) = Protection::from_bits(prot as u64) else {
        return Err(IoError::from_raw_os_error(libc::EINVAL));
    };
    if flags & libc::MAP_ANON != 0 {
        map_regular(addr, size, prot, flags, fd)
    } else if fd < 0 {
        Err(IoError::from_raw_os_error(libc::EINVAL))
    } else if is_real_file(fd) {
        map_from_file(addr, size, flags, prot, fd, offset)
    } else {
        map_from_object(addr, size, flags, prot, fd, offset)
    }
}

pub fn msync_nocancel(_cpu: &Cpu, addr: Uintptr, len: usize, flags: i32) -> IoResult<()> {
    todo!("msync_nocancel(): addr={addr:p} len={len} flags=0x{flags:x}");
}
