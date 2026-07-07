mod exceptions;
mod general;
mod sctlr_el1;
mod system;
mod tcr_el1;

pub use exceptions::*;
pub use general::*;
pub use sctlr_el1::*;
pub use system::*;
pub use tcr_el1::*;

pub const MDSCR_SS: u64 = 1 << 0;
pub const CPACR_FPEN: u64 = 3 << 20;

pub const PSTATE_SS: u64 = 1 << 21;
pub const PSTATE_NZCV: u64 = 0b1111 << 28;
