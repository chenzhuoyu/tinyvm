use std::{
    fmt::{Debug, Formatter, Pointer, Result as FmtResult},
    ops::{Add, AddAssign, Sub, SubAssign},
};

use fn_ptr::{FnPtr, UntypedFnPtr};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VMA(u64);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uintptr(usize);

impl Uintptr {
    #[inline(always)]
    pub fn from_fn(ptr: impl FnPtr) -> Self {
        Self(ptr.addr())
    }
}

impl Uintptr {
    #[inline(always)]
    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }

    #[inline(always)]
    pub const fn as_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    #[inline(always)]
    pub const fn as_ref<'p, T>(self) -> &'p T {
        unsafe { &*self.as_ptr() }
    }

    #[inline(always)]
    pub const fn as_mut<'p, T>(self) -> &'p mut T {
        unsafe { &mut *self.as_ptr() }
    }
}

impl Uintptr {
    #[inline(always)]
    pub fn as_fn<F: FnPtr>(self) -> F {
        unsafe { F::from_ptr(self.0 as UntypedFnPtr) }
    }
}

impl Uintptr {
    #[inline(always)]
    pub fn read<T>(self) -> T {
        unsafe { (self.0 as *const T).read() }
    }

    #[inline(always)]
    pub fn write<T>(self, value: T) {
        unsafe { (self.0 as *mut T).write(value) }
    }
}

impl From<u64> for Uintptr {
    #[inline(always)]
    fn from(addr: u64) -> Self {
        Self::from(addr as usize)
    }
}

impl From<usize> for Uintptr {
    #[inline(always)]
    fn from(addr: usize) -> Self {
        Self::new(addr)
    }
}

impl<T> From<*mut T> for Uintptr {
    #[inline(always)]
    fn from(ptr: *mut T) -> Self {
        Self::from(ptr.addr())
    }
}

impl<T> From<*const T> for Uintptr {
    #[inline(always)]
    fn from(ptr: *const T) -> Self {
        Self::from(ptr.addr())
    }
}

macro_rules! impl_addr_operator {
    (impl $addr:ident { $($op:ident : $ty:ty => $rty:ty [$($action:tt)*]),* $(,)? }) => {
        paste::paste! {
            $(
                impl $op<$ty> for $addr {
                    type Output = Self;

                    #[inline(always)]
                    fn [< $op:lower >](self, rhs: $ty) -> Self {
                        Self(self.0 $($action)* (rhs as $rty))
                    }
                }

                impl [< $op Assign >]<$ty> for $addr {
                    #[inline(always)]
                    fn [< $op:lower _assign >](&mut self, rhs: $ty) {
                        *self = $op::[< $op:lower >](*self, rhs);
                    }
                }
            )*
        }
    };
}

macro_rules! derive_pointer_types {
    ($($name:ident : ($uty:ty, $sty:ty)),+ $(,)?) => {
        $(
            impl $name {
                pub const NIL: Self = Self(0);
            }

            impl $name {
                #[inline(always)]
                pub const fn new(addr: $uty) -> Self {
                    Self(addr)
                }
            }

            impl $name {
                #[inline(always)]
                pub const fn addr(self) -> $uty {
                    self.0
                }
            }

            impl $name {
                #[inline(always)]
                pub const fn is_nil(self) -> bool {
                    self.0 == 0
                }

                #[inline(always)]
                pub const fn align_down(self, size: usize) -> Self {
                    Self(self.0 - self.0 % (size as $uty))
                }

                #[inline(always)]
                pub const fn is_aligned_to(self, size: usize) -> bool {
                    self.0.is_multiple_of(size as $uty)
                }
            }

            impl Debug for $name {
                fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                    Pointer::fmt(self, f)
                }
            }

            impl Pointer for $name {
                fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                    write!(f, "0x{:x}", self.0)
                }
            }

            impl_addr_operator! {
                impl $name {
                    Add : u32   => $uty [ .wrapping_add ] ,
                    Add : u64   => $uty [ .wrapping_add ] ,
                    Add : usize => $uty [ .wrapping_add ] ,
                    Add : i32   => $sty [ .wrapping_add_signed ],
                    Add : i64   => $sty [ .wrapping_add_signed ],
                    Add : isize => $sty [ .wrapping_add_signed ],
                    Sub : u32   => $uty [ .wrapping_sub ],
                    Sub : u64   => $uty [ .wrapping_sub ],
                    Sub : usize => $uty [ .wrapping_sub ],
                    Sub : i32   => $sty [ .wrapping_sub_signed ],
                    Sub : i64   => $sty [ .wrapping_sub_signed ],
                    Sub : isize => $sty [ .wrapping_sub_signed ],
                }
            }

            impl Sub for $name {
                type Output = usize;

                #[inline(always)]
                fn sub(self, rhs: Self) -> usize {
                    self.0
                        .checked_sub(rhs.0)
                        .expect("pointer subtraction underflow") as usize
                }
            }
        )+
    };
}

derive_pointer_types! {
    VMA     : (u64, i64),
    Uintptr : (usize, isize),
}
