use std::{
    ffi::{CStr, OsStr},
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    os::unix::ffi::OsStrExt,
};

use crate::Maybe;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Sz(*mut i8);

impl Sz {
    pub const NIL: Self = Self(std::ptr::null_mut());
}

impl Sz {
    #[inline]
    pub const fn is_nil(self) -> bool {
        self.0.is_null()
    }
}

impl Sz {
    #[inline]
    pub fn to_ptr(self) -> *mut i8 {
        self.0
    }

    #[inline]
    pub fn to_str(self) -> Maybe<&'static str> {
        Ok(self.to_c_str().to_str()?)
    }

    #[inline]
    pub fn to_c_str(self) -> &'static CStr {
        unsafe { CStr::from_ptr(self.0) }
    }

    #[inline]
    pub fn to_bytes(self) -> &'static [u8] {
        self.to_c_str().to_bytes()
    }

    #[inline]
    pub fn to_os_str(self) -> &'static OsStr {
        OsStr::from_bytes(self.to_bytes())
    }
}

impl From<*mut i8> for Sz {
    #[inline]
    fn from(str: *mut i8) -> Self {
        Self(str)
    }
}

impl From<*const i8> for Sz {
    #[inline]
    fn from(str: *const i8) -> Self {
        Self(str as *mut i8)
    }
}

impl From<&'static CStr> for Sz {
    #[inline]
    fn from(str: &'static CStr) -> Self {
        Self(str.as_ptr() as *mut i8)
    }
}

impl AsRef<[u8]> for Sz {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.to_bytes()
    }
}

impl AsRef<CStr> for Sz {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self.to_c_str()
    }
}

impl AsRef<OsStr> for Sz {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.to_os_str()
    }
}

impl Debug for Sz {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.is_nil() {
            write!(f, "(null)")
        } else {
            Debug::fmt(self.to_c_str(), f)
        }
    }
}

impl Display for Sz {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.is_nil() {
            write!(f, "(null)")
        } else {
            Display::fmt(&self.to_c_str().display(), f)
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SzList(*mut Sz);

impl SzList {
    #[inline]
    pub const fn is_nil(self) -> bool {
        self.0.is_null()
    }
}

impl SzList {
    #[inline]
    pub fn to_ptr(self) -> *mut Sz {
        self.0
    }

    #[inline]
    pub fn to_slice(self) -> &'static [Sz] {
        unsafe { std::slice::from_raw_parts(self.0, self.count_items()) }
    }

    #[inline]
    pub fn count_items(mut self) -> usize {
        let mut len = {
            if self.is_nil() {
                return 0usize;
            } else {
                0usize
            }
        };
        while unsafe { !(*self.0).is_nil() } {
            self.0 = unsafe { self.0.add(1) };
            len += 1;
        }
        len
    }
}

impl Debug for SzList {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self.to_slice(), f)
    }
}

impl Iterator for SzList {
    type Item = Sz;

    #[inline]
    fn next(&mut self) -> Option<Sz> {
        unsafe {
            if !self.is_nil() && !(*self.0).is_nil() {
                let ret = *self.0;
                self.0 = self.0.add(1);
                Some(ret)
            } else {
                None
            }
        }
    }
}

impl<T: AsMut<[Sz]>> From<&mut T> for SzList {
    #[inline]
    fn from(data: &mut T) -> Self {
        Self(data.as_mut().as_mut_ptr())
    }
}
