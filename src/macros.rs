macro_rules! __offsets {
    [ $($x:expr),* $(,)? ] => {
        $crate::macros::__offsets!(@suffixes [] (0) [ $($x),* ])
    };
    (@suffixes [ $($out:expr),* ] ( $cur:expr ) [ $head:expr $(, $tail:expr)* ]) => {
        $crate::macros::__offsets!(@suffixes [ $($out,)* $cur ] ( $cur + $head ) [ $($tail),* ])
    };
    (@suffixes [ $($out:expr),* ] ( $cur:expr ) []) => {
        [ $($out,)* ]
    };
}

macro_rules! declare_friendly_enum {
    ($(
        pub enum $name:ident : $real_ty:ty [ $repr_ty:ty ] => $prefix:ident :: {
            $($item:ident),* $(,)?
        }),*
        $(,)?
    ) => {
        paste::paste! {
            $(
                #[repr($repr_ty)]
                #[allow(dead_code)]
                #[allow(non_camel_case_types)]
                #[allow(clippy::upper_case_acronyms)]
                #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
                pub enum $name {
                    $(
                        $item = [< $prefix $item >],
                    )*
                }

                impl $name {
                    #[allow(dead_code)]
                    #[inline(always)]
                    const fn [< $name:snake >](self) -> $real_ty {
                        self as $real_ty
                    }
                }

                impl From<$repr_ty> for $name {
                    fn from(value: $repr_ty) -> Self {
                        match value {
                            $( [< $prefix $item >] => Self::$item, )*
                            value => panic!("unknown {}: {:#x}", stringify!($name), value),
                        }
                    }
                }
            )*
        }
    };
}

macro_rules! define_accessors {
    (
        $($what:ident : $value_ty:ty = ($name:ident : $ty:ty) :: $read:ident -> $write:ident),*
        $(,)?
    ) => {
        paste::paste! {
            #[allow(dead_code)]
            impl Cpu {
                $(
                    #[inline]
                    pub fn [< read_ $what >](&self, $name: $ty) -> $value_ty {
                        let mut ret: $value_ty = unsafe { std::mem::zeroed() };
                        $crate::hv_call!($read(self.cpu, $name.[< $ty:snake >](), &raw mut ret));
                        ret
                    }

                    #[inline]
                    pub fn [< write_ $what >](&self, $name: $ty, value: $value_ty) {
                        $crate::hv_call!($write(self.cpu, $name.[< $ty:snake >](), value));
                    }
                )*
            }
        }
    };
}

macro_rules! define_bit_field {
    ($(
        $(#[$attr:meta])*
        $vis:vis struct $name:ident : $repr:ty {
            $(
                $(#[doc = $doc:expr])*
                $field:ident : $nbits:literal
            ),+
            $(,)?
        }
    )*) => {
        $(
            $(#[$attr])*
            #[repr(transparent)]
            #[derive(Clone, Copy)]
            $vis struct $name(pub $repr);

            impl $name {
                const BV: [u32; ${count($nbits)}] = $crate::macros::__offsets![$($nbits),*];
            }

            #[allow(dead_code)]
            impl $name {
                $(
                    $(#[doc = $doc])*
                    #[inline]
                    pub const fn $field(self) -> $repr {
                        (self.0 >> Self::BV[${index()}]) & ((1 << $nbits) - 1)
                    }
                )+
            }

            impl ::std::fmt::Debug for $name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    f.debug_struct(stringify!($name))
                        $(.field_with(stringify!($field), |f| write!(f, "0x{:x}", self.$field())))*
                        .finish()
                }
            }
        )*
    };
}

pub(crate) use __offsets;
pub(crate) use declare_friendly_enum;
pub(crate) use define_accessors;
pub(crate) use define_bit_field;
