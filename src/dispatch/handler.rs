use log::*;
use core::ffi::c_void;
use crate::include::MmIsAddressValid;
use super::types::*;
use crate::kernel::{KernelError, Process};

/// The raw hook that is called
pub unsafe fn hook(buf: *mut c_void) {
    if !MmIsAddressValid(buf) {
        return;
    }

    info!("handler called with address {:p}", buf);

    let data: &mut Data = &mut *(buf as *mut _);

    let response = if let Data::Request(req) = data {
        handler(&req)
    } else {
        Err(KernelError::Message("kernel received response type as a request"))
    };
}

fn handler(request: &Request) -> Result<Response, KernelError> {
    Ok(match request {
        Request::ModuleInfo(pid) => {
            Response::ModuleInfo(Process::by_id(*pid)?.get_modules_64()?)
        }
    })
}