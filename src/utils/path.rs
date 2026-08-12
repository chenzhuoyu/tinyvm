use std::{
    os::fd::AsRawFd,
    path::{Path, PathBuf},
};

use crate::Maybe;

pub trait LibPathNormalizeExt {
    fn normalize(&self) -> Maybe<PathBuf>;
}

impl<P: AsRef<Path>> LibPathNormalizeExt for P {
    fn normalize(&self) -> Maybe<PathBuf> {
        Ok(soft_canonicalize::soft_canonicalize(self)?)
    }
}

pub fn is_real_file(fd: impl AsRawFd) -> bool {
    unsafe {
        let mut buf = std::mem::zeroed::<libc::stat>();
        let ret = libc::fstat(fd.as_raw_fd(), &raw mut buf);
        ret == 0 && (buf.st_mode & libc::S_IFMT) == libc::S_IFREG
    }
}
