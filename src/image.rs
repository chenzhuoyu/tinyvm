use std::{
    borrow::Cow,
    ffi::{CString, OsStr},
    fmt::{Debug, Formatter, Result as FmtResult},
    fs::File,
    io::Read,
    mem::MaybeUninit,
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::Path,
    sync::LazyLock,
};

use anyhow::{anyhow, ensure};
use object::{
    LittleEndian,
    macho::{
        CPU_TYPE_ARM64, CPU_TYPE_X86_64, FAT_CIGAM, FAT_CIGAM_64, FatArch32, FatArch64,
        MH_CIGAM_64, MH_MAGIC, MH_MAGIC_64, MachHeader64,
    },
    read::macho::{FatArch, LoadCommandVariant, MachHeader, Segment},
};

use crate::{
    Maybe,
    mem::{Addressable, Memory, MemoryExt, Protection},
    utils::{
        io::{MappedFile, MemoryIo, ValueExt},
        path::LibPathNormalizeExt,
        ptr::Uintptr,
        str::Sz,
    },
};

#[repr(C)]
#[derive(Debug)]
struct DyldImageInfo {
    addr: Uintptr,
    path: Sz,
    time: i64,
}

static LLDB_IMAGE_NOTIFIER: LazyLock<
    Option<unsafe extern "C" fn(mode: u32, count: usize, info: *const DyldImageInfo)>,
> = LazyLock::new(|| unsafe {
    std::mem::transmute(libc::dlsym(
        libc::dlopen(c"/usr/lib/dyld".as_ptr(), libc::RTLD_LAZY),
        c"lldb_image_notifier".as_ptr(),
    ))
});

pub struct Image {
    pub data: Memory,
    pub entry: Uintptr,
}

pub static DYLD: LazyLock<Image> =
    LazyLock::new(|| Image::load(Image::DYLD_PATH).expect("cannot load dyld"));

impl Image {
    const CPU_TYPE: u32 = CPU_TYPE_ARM64;
    const DYLD_PATH: &str = "/usr/lib/dyld";
}

impl Image {
    fn read<T>(file: &mut File) -> Maybe<T> {
        unsafe {
            let mut ret = MaybeUninit::<T>::uninit();
            file.read_exact(ret.as_bytes_mut().assume_init_mut())?;
            Ok(ret.assume_init())
        }
    }

    fn map_image(path: &Path) -> Maybe<MappedFile> {
        let mut file = File::open(path)?;
        let magic = Self::read::<u32>(&mut file)?;

        /* check the file magic, and remap the file at correct offset */
        let offset = match magic {
            MH_MAGIC => return Err(anyhow!("32-bit binaries are not supported: {path:?}")),
            MH_MAGIC_64 => 0usize,
            FAT_CIGAM => Self::find_binary(&mut file, false)?,
            FAT_CIGAM_64 => Self::find_binary(&mut file, true)?,
            _ => return Err(anyhow!("not a valid Mach-O binary: {path:?}")),
        };

        /* map the image */
        let size = file.metadata()?.len() as usize;
        let size = size - offset;
        MappedFile::map(file, size, offset)
    }

    fn find_binary(file: &mut File, is_fat64: bool) -> Maybe<usize> {
        for _ in 0..Self::read::<u32>(file)? {
            let (cpu, offset) = {
                if is_fat64 {
                    let arch = Self::read::<FatArch64>(file)?;
                    (arch.cputype(), arch.offset() as usize)
                } else {
                    let arch = Self::read::<FatArch32>(file)?;
                    (arch.cputype(), arch.offset() as usize)
                }
            };
            if cpu == Self::CPU_TYPE {
                return Ok(offset);
            }
        }
        Err(anyhow!("cannot find valid architecture in fat binary"))
    }
}

impl Image {
    pub fn load<P: AsRef<Path>>(path: P) -> Maybe<Self> {
        let path = path.as_ref().normalize()?;
        let file = Self::map_image(&path)?;

        /* read the mach header */
        let mio = &mut MemoryIo(&file);
        let hdr = mio.read::<MachHeader64<LittleEndian>>()?;

        /* validate the Mach-O magic again */
        if hdr.magic() != MH_CIGAM_64 {
            return Err(anyhow!("must be 64-bit little-endian executables"));
        }

        /* virtual address range & fixup types */
        let mut max_addr = 0u64;
        let mut min_addr = u64::MAX;
        let mut segments = Vec::with_capacity(8);
        let mut entry_point = None;

        /* verify & collect load commands */
        for cmd in hdr.load_commands(LittleEndian, file.data(), 0)? {
            match cmd?.variant()? {
                LoadCommandVariant::Thread(.., data) => {
                    if entry_point.is_none() {
                        entry_point = Some(match hdr.cputype.value() {
                            CPU_TYPE_X86_64 => MemoryIo(data).read_at(136)?,
                            CPU_TYPE_ARM64 => MemoryIo(data).read_at(264)?,
                            _ => unreachable!(),
                        })
                    }
                }
                LoadCommandVariant::LoadDylinker(cmd) => unsafe {
                    let ptr = (&raw const *cmd as *const u8).add(cmd.name.offset.usize());
                    let len = file.as_ptr().add(file.len()).offset_from(ptr) as usize;
                    let buf = std::slice::from_raw_parts(ptr, len);
                    let ldr = std::ffi::CStr::from_bytes_until_nul(buf)?;
                    ensure!(ldr == c"/usr/lib/dyld", "unknown loader: {ldr:?}");
                },
                LoadCommandVariant::Segment64(seg, ..) => {
                    if seg.name() != b"__PAGEZERO" {
                        min_addr = min_addr.min(seg.vmaddr.value());
                        max_addr = max_addr.max(seg.vmaddr.value() + seg.vmsize.value());
                        segments.push(seg);
                    }
                }
                LoadCommandVariant::EntryPoint(cmd) => {
                    if entry_point.is_none() {
                        entry_point = Some(cmd.entryoff.usize());
                    } else {
                        tracing::warn!("Found multiple entry points");
                    }
                }
                _ => {}
            }
        }

        /* __TEXT segment should start at fileoff 0 and have the lowest vmaddr */
        for &seg in &segments {
            if seg.name() == b"__TEXT" {
                if seg.fileoff.value() != 0 || seg.vmaddr.value() != min_addr {
                    return Err(anyhow!("malformed Mach-O file: misplaced __TEXT segment"));
                } else {
                    break;
                }
            }
        }

        /* map the image, and calculate ASLR slide */
        let image = Memory::alloc((max_addr - min_addr) as usize, Protection::RW)?;
        let slide = image.addr().addr() - (min_addr as usize);

        /* load the segments */
        for &seg in &segments {
            let vma_addr = Uintptr::new(seg.vmaddr.usize() + slide);
            let vma_next = vma_addr + seg.vmsize.usize();

            /* copy content into place */
            unsafe {
                libc::memcpy(
                    vma_addr.as_ptr(),
                    file.as_ptr().add(seg.fileoff.usize()) as *const libc::c_void,
                    seg.filesize.usize(),
                );
            }

            /* log the segment */
            tracing::debug!(
                "Segment {:?} is loaded at 0x{:x}-0x{:x} ({:p}-{:p})",
                OsStr::from_bytes(seg.name()),
                seg.vmaddr.value(),
                seg.vmaddr.value() + seg.vmsize.value(),
                vma_addr,
                vma_next,
            );
        }

        /* notify the debugger, if present */
        if let Some(lldb_image_notifier) = *LLDB_IMAGE_NOTIFIER {
            let name = CString::new(path.as_os_str().as_bytes())
                .map_or(Cow::Borrowed(c"(???)"), Cow::Owned);
            let info = DyldImageInfo {
                addr: image.addr(),
                path: Sz::from(name.as_ptr()),
                time: path.metadata().map_or(0, |m| m.mtime()),
            };
            unsafe {
                lldb_image_notifier(0, 1, &raw const info);
            }
        }

        /* the following logic is for dyld only */
        if path.as_path() != Self::DYLD_PATH {
            return Ok(Image {
                data: image,
                entry: Uintptr::NIL,
            });
        }

        /* set the segments with correct protection */
        for &seg in &segments {
            if let Some(prot) = Protection::from_bits(seg.initprot.value() as u64) {
                let size = seg.vmsize.usize();
                let addr = seg.vmaddr.usize() + slide;
                let offs = addr - image.addr().addr();
                image.view(offs..offs + size).protect(prot);
            } else {
                return Err(anyhow!("invalid initprot: 0x{:x}", seg.initprot.value()));
            }
        }

        /* construct the image */
        Ok(Self {
            entry: entry_point.map_or(Uintptr::NIL, |entry| image.addr() + entry),
            data: image,
        })
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "Image({:p}-{:p},entry={:p})",
            self.data.addr(),
            self.data.addr() + self.data.size(),
            self.entry
        )
    }
}
