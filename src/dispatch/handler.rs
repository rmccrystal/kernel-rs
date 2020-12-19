use log::*;
use core::ffi::c_void;
use crate::include::MmIsAddressValid;

/// The raw hook that is called
pub unsafe fn hook(request: *mut c_void) {
    if !MmIsAddressValid(request) {
        return;
    }

    info!("handler called with address {:p}", request);

    // convert `request` to a mutable borrowed type of handler
    handler(&mut *(&mut *(request) as *mut core::ffi::c_void as *mut i32))
}

fn handler(data: &mut i32) {
    info!("num: {}", data);
    *data = 1;
}