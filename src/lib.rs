#![feature(atomic_ptr_null)]
#![feature(cstr_display)]
#![feature(debug_closure_helpers)]
#![feature(error_generic_member_access)]
#![feature(macro_metavar_expr)]
#![feature(maybe_uninit_as_bytes)]
#![cfg_attr(target_arch = "aarch64", feature(portable_simd))]
#![cfg_attr(target_arch = "aarch64", feature(simd_ffi))]
#![feature(slice_shift)]

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
pub mod image;
pub(crate) mod macros;
pub mod mem;
pub mod utils;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
use aarch64::ffi;
#[cfg(target_arch = "x86_64")]
use x86_64::ffi;

pub type Unit = Maybe<()>;
pub type Maybe<T> = Result<T, anyhow::Error>;

#[macro_export]
macro_rules! hv_call {
    ($name:ident ( $($arg:expr),* $(,)? )) => {{
        #[allow(clippy::macro_metavars_in_unsafe)]
        #[allow(unused_unsafe)]
        match unsafe { $name($($arg),*) } {
            $crate::ffi::HV_SUCCESS => {}
            $crate::ffi::HV_ERROR => panic!(concat!(stringify!($name), ": generic hypervisor error")),
            $crate::ffi::HV_BUSY => panic!(concat!(stringify!($name), ": hypervisor is busy")),
            $crate::ffi::HV_BAD_ARGUMENT => panic!(concat!(stringify!($name), ": bad arguments")),
            $crate::ffi::HV_ILLEGAL_GUEST_STATE => panic!(concat!(stringify!($name), ": illegal guest state")),
            $crate::ffi::HV_NO_RESOURCES => panic!(concat!(stringify!($name), ": insufficient resources")),
            $crate::ffi::HV_NO_DEVICE => panic!(concat!(stringify!($name), ": no devices")),
            $crate::ffi::HV_DENIED => panic!(concat!(stringify!($name), ": denied")),
            #[cfg(target_arch = "x86_64")]
            $crate::ffi::HV_FAULT => panic!(concat!(stringify!($name), ": fault")),
            #[cfg(target_arch = "aarch64")]
            $crate::ffi::HV_EXISTS => panic!(concat!(stringify!($name), ": exists")),
            $crate::ffi::HV_UNSUPPORTED => panic!(concat!(stringify!($name), ": unsupported operation")),
            err => panic!("{}: unknown error: {}", stringify!($name), err),
        }
    }};
}

#[macro_export]
macro_rules! io_error {
    ($kind:ident, $msg:literal) => {
        std::io::Error::new(std::io::ErrorKind::$kind, format!($msg))
    };
    ($kind:ident, $expr:expr) => {
        std::io::Error::new(std::io::ErrorKind::$kind, $expr)
    };
    ($kind:ident, $msg:literal, $($arg:tt)*) => {
        std::io::Error::new(std::io::ErrorKind::$kind, format!($msg, $($arg)*))
    };
}
