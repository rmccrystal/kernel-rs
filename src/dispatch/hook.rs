use log::*;
use core::ffi::c_void;
use crate::include::MmIsAddressValid;
use super::types::*;
use crate::kernel::{KernelError, Process};
use alloc::vec::Vec;
use winapi::_core::intrinsics::copy;


/// Holds the response after RunRequest is called
// TODO: Maybe add some kind of ID so we can verify integrity
static mut RESPONSE_HOLD: Option<Vec<u8>> = None;

/// The raw hook that is called
pub unsafe fn hook(buf: *mut c_void) {
    if !MmIsAddressValid(buf as _) {
        return;
    }
    let data_size = core::mem::size_of::<Data>();
    if !MmIsAddressValid((buf as *mut u8).add(data_size - 1) as _) {
        error!("The buffer start address was valid but the buffer end address was not valid (Data struct size = {})", data_size);
        return;
    }

    let data: &mut Data = &mut *(buf as *mut _);

    trace!("Received data from hooked fn: {:?}", &data);

    match data {
        Data::RunRequest { req, response } => {
            trace!("Received request: {:?}", req);
            // handle the request
            let resp = req.handle();

            trace!("Handled request with response {:?}", resp);

            // true if response has elements of String or Vec or whatever.
            // if this is true we need to serialize and create a buffer from usermode.
            let dynamic_response_size = match &resp {
                Ok(response_data) => match response_data {
                    Response::Pong => false,
                    _ => true
                },
                Err(_) => true
            };

            if dynamic_response_size {
                // serialize the response
                let response_buf = match postcard::to_allocvec(&resp) {
                    Ok(buf) => buf,
                    Err(err) => {
                        error!("Error serializing response: {:?}", err);
                        return;
                    }
                };

                trace!("Serialized {} bytes", response_buf.len());

                // set the serialized length
                **response = RunRequestResponse::AllocBuffer(response_buf.len());

                // write the response hold
                RESPONSE_HOLD = Some(response_buf);
            } else {
                **response = RunRequestResponse::Response(resp);
            }
        },
        Data::WriteBuffer { buffer } => {
            if RESPONSE_HOLD.is_none() {
                error!("WriteBuffer was called before RunRequest");
                return;
            }

            let resp = RESPONSE_HOLD.as_ref().unwrap();
            if buffer.capacity() < resp.len() {
                error!("The buffer capacity was too small! buffer.capacity() = {}, resp.len() = {}", buffer.capacity(), resp.len());
                return;
            }

            // Copy the response buffer into the buffer from usermode
            copy(resp.as_ptr(), buffer.as_mut_ptr(), resp.len());

            // set the length of the usermode buffer
            buffer.set_len(resp.len());

            trace!("Wrote {} bytes to the WriteBuffer", resp.len());
        }
    }
}

