use std::ffi::c_void;

use winapi::um::libloaderapi::LoadLibraryA;

use types::*;
use log::*;
use anyhow::*;
use crate::driver::HOOKED_FN_NAME;

mod driver;
mod types;
mod util;

pub struct KernelHandle {
    hook: extern "stdcall" fn(*mut c_void),
}

impl KernelHandle {
    pub fn new() -> Result<KernelHandle> {
        unsafe { Self::init_hook().context("Could not load required libraries for hooked function")? };
        let hook = unsafe { Self::get_hook().context("Could not get hooked function")? };

        debug!(
            "Found function {}: 0x{:X}",
            HOOKED_FN_NAME, hook as *const () as usize
        );

        Ok(Self {
            hook,
        })
    }
}

macro_rules! request {
    ($self:ident, $req:expr, $resp_type:path) => {{
        let resp = $self.send_request($req);
        match resp {
            Err(err) => Err(anyhow!("{:?}", err)),
            Ok(resp) => {
                let result: anyhow::Result<_> = if let $resp_type(result) = resp {
                    Ok(result)
                } else {
                    bail!("received invalid response type")
                };
                result
            }
        }
    }};
}

// when we're making a request without a response buffer (ping, write, etc)
macro_rules! request_no_resp {
    ($self:ident, $req:expr, $resp_type:path) => {{
        let resp = $self.send_request($req);
        match resp {
            Err(err) => Err(anyhow!("{:?}", err)),
            Ok(resp) => {
                let result: anyhow::Result<_> = if let $resp_type = resp {
                    Ok(())
                } else {
                    bail!("received invalid response type")
                };
                result
            }
        }
    }};
}

impl KernelHandle {
    fn ping(&self) -> Result<()> {
        request_no_resp!(
            self,
            Request::Ping,
            Response::Pong
        )
    }

    fn module_info(&self, pid: u64) -> Result<Vec<ModuleInfo>> {
        request!(
            self,
            Request::ModuleInfo(pid),
            Response::ModuleInfo
        )
    }

    fn get_peb_address(&self, pid: u64) -> Result<u64> {
        request!(
            self,
            Request::GetPebAddress(pid),
            Response::PebAddress
        )
    }

    fn read_memory(&self, pid: u64, address: u64, size: u64) -> Result<Vec<u8>> {
        request!(
            self,
            Request::ReadMemory{pid, address, size},
            Response::ReadMemory
        )
    }

    fn write_memory(&self, pid: u64, address: u64, bytes: &[u8]) -> Result<()> {
        request_no_resp!(
            self,
            Request::WriteMemory {pid, address, buf: bytes.to_vec()},
            Response::WriteMemory
        )
    }
}
