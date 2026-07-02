use std::{
    fmt::{Debug, Formatter, Pointer, Result as FmtResult},
    ops::{Add, AddAssign, Sub, SubAssign},
};

use fn_ptr::{FnPtr, UntypedFnPtr};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uintptr(usize);

impl Uintptr {
    pub const NIL: Self = Self(0);
}

impl Uintptr {
    #[inline]
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }
}

impl Uintptr {
    #[inline]
    pub fn from_fn(ptr: impl FnPtr) -> Self {
        Self(ptr.addr())
    }
}

impl Uintptr {
    #[inline]
    pub const fn addr(self) -> usize {
        self.0
    }

    #[inline]
    pub const fn is_nil(self) -> bool {
        self.0 == 0
    }
}

impl Uintptr {
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }

    #[inline]
    pub const fn as_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    #[inline]
    pub const fn as_ref<'p, T>(self) -> &'p T {
        unsafe { &*self.as_ptr() }
    }

    #[inline]
    pub const fn as_mut<'p, T>(self) -> &'p mut T {
        unsafe { &mut *self.as_ptr() }
    }
}

impl Uintptr {
    #[inline]
    pub fn as_fn<F: FnPtr>(self) -> F {
        unsafe { F::from_ptr(self.0 as UntypedFnPtr) }
    }
}

impl Uintptr {
    #[inline]
    pub fn read<T>(self) -> T {
        unsafe { (self.0 as *const T).read() }
    }

    #[inline]
    pub fn write<T>(self, value: T) {
        unsafe { (self.0 as *mut T).write(value) }
    }
}

macro_rules! impl_operator {
    ($($op:ident : $ty:ty => $rty:ty $([ $suffix:ident ])?),* $(,)?) => {
        paste::paste! {
            $(
                impl $op<$ty> for Uintptr {
                    type Output = Self;

                    #[inline]
                    fn [< $op:lower >](self, rhs: $ty) -> Self {
                        Self(self.0.[< wrapping_ $op:lower $($suffix)? >](rhs as $rty))
                    }
                }

                impl [< $op Assign >]<$ty> for Uintptr {
                    #[inline]
                    fn [< $op:lower _assign >](&mut self, rhs: $ty) {
                        *self = $op::[< $op:lower >](*self, rhs);
                    }
                }
            )*
        }
    };
}

impl Debug for Uintptr {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Pointer::fmt(self, f)
    }
}

impl Pointer for Uintptr {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "0x{:x}", self.0)
    }
}

impl From<u64> for Uintptr {
    #[inline]
    fn from(addr: u64) -> Self {
        Self::from(addr as usize)
    }
}

impl From<usize> for Uintptr {
    #[inline]
    fn from(addr: usize) -> Self {
        Self::new(addr)
    }
}

impl<T> From<*mut T> for Uintptr {
    #[inline]
    fn from(ptr: *mut T) -> Self {
        Self::from(ptr.addr())
    }
}

impl<T> From<*const T> for Uintptr {
    #[inline]
    fn from(ptr: *const T) -> Self {
        Self::from(ptr.addr())
    }
}

impl_operator! {
    Add : u32   => usize,
    Add : u64   => usize,
    Add : usize => usize,
    Add : i32   => isize [ _signed ],
    Add : i64   => isize [ _signed ],
    Add : isize => isize [ _signed ],
    Sub : u32   => usize,
    Sub : u64   => usize,
    Sub : usize => usize,
    Sub : i32   => isize [ _signed ],
    Sub : i64   => isize [ _signed ],
    Sub : isize => isize [ _signed ],
}

impl Sub for Uintptr {
    type Output = usize;

    #[inline]
    fn sub(self, rhs: Self) -> usize {
        self.0
            .checked_sub(rhs.0)
            .expect("pointer subtraction underflow")
    }
}
