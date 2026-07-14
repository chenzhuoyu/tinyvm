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
            $(
                $(#[doc = $doc:expr])*
                $item:ident
            ),*
            $(,)?
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
                        $(#[doc = $doc])*
                        $item = [< $prefix $item >],
                    )*
                }

                impl $name {
                    #[allow(dead_code)]
                    #[inline(always)]
                    pub fn all() -> impl Iterator<Item = Self> {
                        [$( Self::$item ),*].into_iter()
                    }
                }

                impl $name {
                    #[allow(dead_code)]
                    #[inline(always)]
                    pub const fn [< $name:snake >](self) -> $real_ty {
                        self as $real_ty
                    }
                }

                impl From<$repr_ty> for $name {
                    #[inline]
                    fn from(value: $repr_ty) -> Self {
                        match value {
                            $( [< $prefix $item >] => Self::$item, )*
                            value => panic!(concat!("invalid value of ", stringify!($name), ": {:#x}"), value),
                        }
                    }
                }
            )*
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
        paste::paste! {
            $(
                $(#[$attr])*
                #[repr(transparent)]
                #[allow(non_camel_case_types)]
                #[allow(clippy::upper_case_acronyms)]
                #[derive(Clone, Copy)]
                $vis struct $name(pub $repr);

                impl $name {
                    const BV: [u32; ${count($nbits)}] = $crate::macros::__offsets![$($nbits),*];
                }

                #[allow(dead_code)]
                impl $name {
                    #[inline]
                    pub const fn builder() -> [< $name Builder >] {
                        $name(0).into_builder()
                    }

                    #[inline]
                    pub const fn into_builder(self) -> [< $name Builder >] {
                        [< $name Builder >](self)
                    }
                }

                #[allow(dead_code)]
                impl $name {
                    #[doc = concat!("Get the underlying value of ", stringify!($name))]
                    #[inline(always)]
                    pub const fn value(self) -> $repr {
                        self.0
                    }
                }

                #[allow(dead_code)]
                impl $name {
                    $(
                        $(#[doc = $doc])*
                        #[inline]
                        #[allow(non_snake_case)]
                        #[allow(clippy::upper_case_acronyms)]
                        pub const fn $field(self) -> $repr {
                            (self.0 >> Self::BV[${index()}]) & ((1 << $nbits) - 1)
                        }

                        $(#[doc = $doc])*
                        #[inline]
                        #[allow(non_snake_case)]
                        #[allow(non_camel_case_types)]
                        #[allow(clippy::upper_case_acronyms)]
                        pub const fn [< set_ $field >](&mut self, value: $repr) {
                            assert!(value & !((1 << $nbits) - 1) == 0);
                            self.0 &= !(((1 << $nbits) - 1) << Self::BV[${index()}]);
                            self.0 |= (value & ((1 << $nbits) - 1)) << Self::BV[${index()}];
                        }
                    )+
                }

                impl ::std::fmt::Debug for $name {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        f.debug_struct(stringify!($name))
                        $(
                            .field_with(stringify!($field), |f| {
                                let value = self.$field();
                                const BWIDTH: usize = $nbits;
                                const HWIDTH: usize = BWIDTH.div_ceil(4);
                                write!(f, "{value:0BWIDTH$b} (0x{value:0HWIDTH$x})")
                            })
                        )+
                            .finish()
                    }
                }

                #[repr(transparent)]
                #[allow(non_camel_case_types)]
                #[allow(clippy::upper_case_acronyms)]
                #[derive(Clone, Copy)]
                $vis struct [< $name Builder >]($name);

                #[allow(dead_code)]
                impl [< $name Builder >] {
                    #[doc = concat!("Build the final ", stringify!($name))]
                    #[inline]
                    pub const fn build(self) -> $name {
                        self.0
                    }
                }

                #[allow(dead_code)]
                impl [< $name Builder >] {
                    $(
                        $(#[doc = $doc])*
                        #[inline]
                        #[allow(non_snake_case)]
                        #[allow(non_camel_case_types)]
                        #[allow(clippy::upper_case_acronyms)]
                        pub const fn $field(mut self, value: $repr) -> Self {
                            self.0.[< set_ $field >](value);
                            self
                        }
                    )*
                }
            )*
        }
    };
}

pub(crate) use __offsets;
pub(crate) use declare_friendly_enum;
pub(crate) use define_bit_field;
