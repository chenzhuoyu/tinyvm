use std::sync::LazyLock;

use mach2::{
    port::{mach_port_name_t, mach_port_t},
    traps::mach_task_self,
};

use crate::aarch64::virtos::HalProvider;

/// The singleton of mach-port that represents `self`.
pub(super) static TASK_SELF: LazyLock<mach_port_t> = LazyLock::new(|| unsafe { mach_task_self() });

#[inline]
pub fn task_self_trap(_hal: &impl HalProvider) -> mach_port_name_t {
    *TASK_SELF
}
