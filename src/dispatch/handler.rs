use log::*;
use core::ffi::c_void;
use crate::include::MmIsAddressValid;

/// The raw hook that is called
pub fn hook(request: *mut c_void) {
    unsafe {
        if !MmIsAddressValid(request) {
            return;
        }

        info!("handler called with address {:p}", request);

        handler(core::mem::transmute(&mut *(request)))
    }
}

fn handler(data: &mut i32) {
    info!("num: {}", data);
    *data = 1;
}