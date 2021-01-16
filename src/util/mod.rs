pub use list_entry::*;
pub use string::*;
pub use vsb::VariableSizedBox;

pub use self::alloc::KernelAlloc;
use crate::include::MmIsAddressValid;

#[macro_use]
pub mod log;
mod string;
mod alloc;
mod vsb;
mod list_entry;

pub fn is_address_valid<T>(address: *const T) -> bool {
    unsafe { MmIsAddressValid(address as *const u8 as _) }
}