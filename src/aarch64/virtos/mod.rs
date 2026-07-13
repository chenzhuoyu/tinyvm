#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod commpage;
mod shared_cache;
mod task;
mod vm;

pub use commpage::*;
pub use shared_cache::*;
pub use task::*;
pub use vm::*;

pub trait HalProvider {
    fn flush_tlb_range(&mut self, start: u64, num_pages: usize);
}
