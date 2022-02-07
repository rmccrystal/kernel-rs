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

    if let Dispatch { handled: false, data: Data::Request(Request::ReadPhysical { address: 1761280, .. }) } = req {
        log::trace!("Hello :D: {:#X?}", req);
    }

    #[cfg(debug_assertions)]
    log::trace!("Got dispatch: {:#X?}", req);

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
        Request::ReadPhysical { address, buf, len } => {
            let buf = buf.as_mut().ok_or(KernelError::InvalidRequest)?;
            // let map = PhysicalMap::new(address as _, buf.len()).ok_or(KernelError::MmMapIoSpace { address, len: buf.len() })?;
            // buf.copy_from_slice(&map);
            let buf = core::slice::from_raw_parts_mut(buf, len);
            read_physical_memory(address, buf).map_err(|e| match e {
                (ntstatus::STATUS_PARTIAL_COPY, n) => KernelError::PartialCopy { address, len: buf.len(), read: n },
                (e, _) => KernelError::NtStatus(e)
            })?;
            Ok(Response::ReadPhysical)
        }
        Request::WritePhysical { address, buf, len } => {
            let mut map = PhysicalMap::new(address as _, len).ok_or(KernelError::MmMapIoSpace { address, len })?;
            map.copy_from_slice(core::slice::from_raw_parts(buf, len));
            Ok(Response::WritePhysical)
        }
        Request::Unregister => {
            // TODO: Remove this, it crashes your pc
            log::info!("Unregistered callback");
            CONTEXT.callback.unregister().map_err(KernelError::NtStatus)?;
            Ok(Response::Unregister)
        }
    }
}