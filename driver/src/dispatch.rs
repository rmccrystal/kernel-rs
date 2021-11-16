use winkernel::basedef::ntstatus;
use winkernel::kernel::{is_valid_ptr, PhysicalMap, read_physical_memory};
use crate::CONTEXT;
use crate::shared::*;

/// Returns true if the request was handled
pub unsafe fn dispatch(req: *mut Dispatch) -> bool {
    if !is_valid_ptr(req) {
        log::error!("Invalid pointer given to dispatch");
        return false;
    }
    let req = match req.as_mut() {
        Some(n) => n,
        None => {
            log::error!("req.as_mut() failed");
            return false;
        }
    };
    if req.handled {
        return false;
    }

    log::info!("Got dispatch: {:#X?}", req);
    if let Data::Request(request) = &req.data {
        let resp = handler(request);
        req.data = Data::Response(resp);
    } else {
        req.data = Data::Response(Err(KernelError::InvalidRequest))
    }

    req.handled = true;
    true
}

pub unsafe fn handler(request: &Request) -> Result<Response, KernelError> {
    match *request {
        Request::Ping => Ok(Response::Ping),
        Request::ReadPhysical { address, buf } => {
            let buf = buf.as_mut().ok_or(KernelError::InvalidRequest)?;
            // let map = PhysicalMap::new(address as _, buf.len()).ok_or(KernelError::MmMapIoSpace { address, len: buf.len() })?;
            // buf.copy_from_slice(&map);
            read_physical_memory(address, buf).map_err(|e| match e {
                (ntstatus::STATUS_PARTIAL_COPY, n) => KernelError::PartialCopy { address, len: buf.len(), read: n },
                (e, _) => KernelError::NtStatus(e)
            })?;
            Ok(Response::ReadPhysical)
        }
        Request::WritePhysical { address, buf } => {
            let buf = buf.as_ref().ok_or(KernelError::InvalidRequest)?;
            let mut map = PhysicalMap::new(address as _, buf.len()).ok_or(KernelError::MmMapIoSpace { address, len: buf.len() })?;
            map.copy_from_slice(buf);
            Ok(Response::WritePhysical)
        }
        Request::Unregister => {
            log::info!("Unregistered callback");
            CONTEXT.callback.unregister().map_err(KernelError::NtStatus)?;
            Ok(Response::Unregister)
        }
    }
}