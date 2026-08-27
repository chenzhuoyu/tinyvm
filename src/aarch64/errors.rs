use std::io::{Error as IoError, Result as IoResult};

use mach2::kern_return::{
    KERN_FAILURE, KERN_INVALID_ADDRESS, KERN_INVALID_ARGUMENT, KERN_MEMORY_ERROR,
    KERN_PROTECTION_FAILURE, KERN_SUCCESS, kern_return_t,
};

pub trait AsKernReturn {
    fn as_kern_return(&self) -> kern_return_t;
}

impl AsKernReturn for IoError {
    #[inline]
    fn as_kern_return(&self) -> kern_return_t {
        if let Some(errno) = self.raw_os_error() {
            match errno {
                libc::EACCES => KERN_PROTECTION_FAILURE,
                libc::ENOMEM => KERN_INVALID_ADDRESS,
                libc::EINVAL => KERN_INVALID_ARGUMENT,
                _ => KERN_MEMORY_ERROR,
            }
        } else {
            KERN_FAILURE
        }
    }
}

impl AsKernReturn for IoResult<()> {
    #[inline]
    fn as_kern_return(&self) -> kern_return_t {
        if let Err(err) = self {
            err.as_kern_return()
        } else {
            KERN_SUCCESS
        }
    }
}
