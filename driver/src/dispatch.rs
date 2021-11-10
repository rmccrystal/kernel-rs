use winkernel::kernel::{is_valid_ptr, PhysicalMap};
use crate::CONTEXT;
use crate::shared::*;

/// Returns true if the request was handled
pub unsafe fn dispatch(req: *mut Dispatch) -> bool {
    if !is_valid_ptr(req) {
        log::error!("Invalid pointer given to dispatch");
        return false
    }
    let req = match req.as_mut() {
        Some(n) => n,
        None => {
            log::error!("req.as_mut() failed");
            return false;
        }
    };

    if let Dispatch::Request(request) = req {
        let resp = handler(request);
        *req = Dispatch::Response(resp);
    } else {
        *req = Dispatch::Response(Err(KernelError::InvalidRequest))
    }

    true
}

pub unsafe fn handler(request: &Request) -> Result<Response, KernelError> {
    match *request {
        Request::Ping => Ok(Response::Ping),
        Request::ReadPhysical { address, buf } => {
            let buf = buf.as_mut().ok_or(KernelError::InvalidRequest)?;
            let map = PhysicalMap::new(address, buf.len()).ok_or(KernelError::MmMapIoSpace)?;
            buf.copy_from_slice(&map);
            Ok(Response::ReadPhysical)
        }
        Request::WritePhysical { address, buf } => {
            let buf = buf.as_ref().ok_or(KernelError::InvalidRequest)?;
            let mut map = PhysicalMap::new(address, buf.len()).ok_or(KernelError::MmMapIoSpace)?;
            map.copy_from_slice(buf);
            Ok(Response::WritePhysical)
        }
        Request::Unregister => {
            log::info!("Unregistered callback");
            CONTEXT.callback.unregister();
            Ok(Response::Unregister)
        }
    }
}