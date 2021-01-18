pub use list_entry::*;
pub use string::*;
pub use vsb::VariableSizedBox;

pub use self::alloc::KernelAlloc;
use crate::include::{MmIsAddressValid, PEPROCESS};
use crate::kernel::ProcessAttachment;

#[macro_use]
pub mod log;
mod string;
mod alloc;
mod vsb;
mod list_entry;

pub trait ToAddress {
    fn to_address(&self) -> usize;
}

impl<T> ToAddress for *const T {
    fn to_address(&self) -> usize {
        *self as _
    }
}

impl<T> ToAddress for *mut T {
    fn to_address(&self) -> usize {
        *self as _
    }
}

impl ToAddress for usize {
    fn to_address(&self) -> usize {
        *self
    }
}

impl ToAddress for u64 {
    fn to_address(&self) -> usize {
        *self as _
    }
}

pub fn is_address_valid(address: impl ToAddress) -> bool {
    unsafe { MmIsAddressValid(address.to_address() as _) }
}

pub fn is_process_address_valid(process: PEPROCESS, address: impl ToAddress) -> bool {
    let _attach = unsafe { ProcessAttachment::attach(process) };
    is_address_valid(address)
}