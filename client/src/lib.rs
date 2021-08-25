use std::ffi::c_void;

use winapi::um::libloaderapi::LoadLibraryA;

pub use types::*;
use log::*;
use anyhow::*;
use crate::driver::HOOKED_FN_NAME;

mod driver;
mod types;
mod util;
mod kdmapper;
pub mod cleaner;

#[cfg(test)]
mod tests;

pub type KernelResult<T> = Result<T, KernelError>;

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

        let handle = Self {
            hook,
        };

        if let Err(_) = handle.ping() {
            info!("Could not ping driver. Mapping driver");
            crate::kdmapper::map_driver().context("Could not map driver")?;
        }
        Ok(handle)
    }
}

macro_rules! request {
    ($self:ident, $req:expr, $resp_type:path) => {{
        let resp = $self.send_request($req);
        match resp {
            Err(err) => Err(err),
            Ok(resp) => {
                let result: KernelResult<_> = if let $resp_type(result) = resp {
                    Ok(result)
                } else {
                    Err(KernelError::text(&format!("received invalid response type. received {:?}, expected {:?}", resp, stringify!($resp_type))))
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
            Err(err) => Err(err),
            Ok(resp) => {
                let result: KernelResult<_> = if let $resp_type = resp {
                    Ok(())
                } else {
                    Err(KernelError::text(&format!("received invalid response type. received {:?}, expected {:?}", resp, stringify!($resp_type))))
                };
                result
            }
        }
    }};
}

impl KernelHandle {
    pub fn ping(&self) -> KernelResult<()> {
        request_no_resp!(
            self,
            Request::Ping,
            Response::Pong
        )
    }

    pub fn module_info(&self, pid: u64) -> KernelResult<Vec<ModuleInfo>> {
        request!(
            self,
            Request::ModuleInfo(pid),
            Response::ModuleInfo
        )
    }

    pub fn get_peb_address(&self, pid: u64) -> KernelResult<u64> {
        request!(
            self,
            Request::GetPebAddress(pid),
            Response::PebAddress
        )
    }

    /// Returns 64 if 64 bit or 32 if 32 bit
    pub fn get_process_bitness(&self, pid: u64) -> KernelResult<u16> {
        request!(
            self,
            Request::GetProcessBitness(pid),
            Response::ProcessBitness
        )
    }

    pub fn read_memory(&self, pid: u64, address: u64, buf: &mut [u8]) -> KernelResult<()> {
        let resp = self.send_request(Request::ReadMemory {pid, address, buf});
        request_no_resp!(
            self,
            Request::ReadMemory{pid, address, buf},
            Response::ReadMemory
        )
    }

    pub fn write_memory(&self, pid: u64, address: u64, buf: &[u8]) -> KernelResult<()> {
        request_no_resp!(
            self,
            Request::WriteMemory {pid, address, buf},
            Response::WriteMemory
        )
    }
}
