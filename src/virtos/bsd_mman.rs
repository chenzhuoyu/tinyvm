use std::io::{Error as IoError, Result as IoResult};

use crate::{
    cpu::Cpu,
    mem::Protection,
    paging::PAGE_SIZE,
    utils::{ptr::Uintptr, size::align_to_page},
    virtos::{
        mem::VmMap,
        mmio::{MmioContext, MmioHandler},
        sys::{sys_map, sys_protect, sys_unmap},
    },
    vm::Vm,
};

#[repr(C, align(16384))]
#[derive(Debug, Clone, Copy)]
struct Page([u8; PAGE_SIZE]);

struct FileMap {
    size: usize,
    addr: Uintptr,
    phys: Uintptr,
    prot: Protection,
}

impl FileMap {
    #[inline]
    const fn new(size: usize, addr: Uintptr, phys: Uintptr, prot: Protection) -> Self {
        Self {
            size,
            addr,
            phys,
            prot,
        }
    }
}

impl Drop for FileMap {
    fn drop(&mut self) {
        if self.prot.contains(Protection::WRITE) {
            unimplemented!("file map write back");
        }
        sys_unmap(self.addr, self.size).ok();
        Vm::unmap(self.phys.into(), self.size);
        Vm::dealloc(self.phys, self.size);
    }
}

impl MmioHandler for FileMap {
    fn handle(&self, mcx: MmioContext<'_>) -> bool {
        // FIXME: may need some sort of swap-out mechanism to save memory.
        let offs = mcx.addr - self.addr;
        let addr = self.addr + (offs & !(PAGE_SIZE - 1));
        let phys = self.phys + (offs & !(PAGE_SIZE - 1));
        *phys.as_mut::<Page>() = *addr.as_ref();
        Vm::map(phys, phys.into(), PAGE_SIZE, self.prot);
        true
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
    // TODO: implement file map write back
    if flags & libc::MAP_SHARED != 0 && prot.contains(Protection::WRITE) {
        unimplemented!("file map write back");
    }
    let addr = sys_map(addr, size, prot & !Protection::EXEC, flags, fd, offset)?;
    let phys = Vm::alloc(size);
    let mmio = FileMap::new(size, addr, phys, prot);
    let max_prot = Protection::all();
    VmMap::iomap(mmio, addr, phys.into(), size, prot, max_prot, false);
    Ok(addr)
}

pub fn msync(_cpu: &Cpu, addr: Uintptr, len: usize, flags: i32) -> IoResult<()> {
    todo!("msync(): addr={addr:p} len={len} flags=0x{flags:x}");
}

pub fn munmap(cpu: &Cpu, addr: Uintptr, len: usize) -> IoResult<()> {
    sys_unmap(addr, len)?;
    VmMap::unmap(addr, len);
    cpu.flush_tlb(addr, len.div_ceil(PAGE_SIZE));
    Ok(())
}

pub fn mprotect(cpu: &Cpu, addr: Uintptr, len: usize, raw_prot: i32) -> IoResult<()> {
    if let Some(prot) = Protection::from_bits(raw_prot) {
        sys_protect(addr, len, prot & !Protection::EXEC)?;
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
