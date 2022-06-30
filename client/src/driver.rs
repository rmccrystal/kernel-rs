#![allow(clippy::missing_safety_doc)]

use std::ffi::CString;
use std::os::raw::c_void;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use rand::Rng;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS};
use winreg::RegKey;
use crate::shared::*;

/// Wrapper for RegKey. Deletes the key when dropped
struct TempRegKey(RegKey, String);

impl Drop for TempRegKey {
    fn drop(&mut self) {
        let _ = self.0.delete_value(&self.1);
    }
}

impl Clone for TempRegKey {
    fn clone(&self) -> Self {
        unsafe {
            let mut reg_folder = std::mem::zeroed();
            std::ptr::copy(&self.0, &mut reg_folder, 1);
            Self(reg_folder, self.1.clone())
        }
    }
}

#[derive(Clone)]
pub struct DriverHandle {
    key: Rc<TempRegKey>,
}

unsafe impl Send for DriverHandle {}

#[cfg(all(debug_assertions, feature = "debug"))]
const DRIVER_BYTES: &[u8] = include_bytes!("../../target/x86_64-pc-windows-msvc/debug/driver.dll").as_slice();
#[cfg(not(all(debug_assertions, feature = "debug")))]
const DRIVER_BYTES: &[u8] = include_bytes!("../../target/x86_64-pc-windows-msvc/release/driver.dll").as_slice();

impl DriverHandle {
    pub unsafe fn new() -> anyhow::Result<Self> {
        // let key: String = rand::thread_rng()
        //     .sample_iter(&Alphanumeric)
        //     .take(10)
        //     .map(char::from)
        //     .collect();
        // let key_cstr = CString::new(KEY).unwrap();

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let reg_folder = hklm.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\Dwm", winreg::enums::KEY_ALL_ACCESS)?;

        let driver = Self {
            key: Rc::new(TempRegKey(reg_folder, KEY.to_string()))
        };

        // driver.unregister();
        // log::info!("Unregistered driver");
        // std::thread::sleep(Duration::from_millis(1000));
        // log::info!("sleep end");

        if !driver.ping() || cfg!(feature = "remap") {
            log::info!("Mapping driver");
            kdmapper::kdmapper(DRIVER_BYTES, false, true, false, 0, 0).ok_or_else(|| anyhow::anyhow!("Could not map driver"))?;
            if !driver.ping() {
                anyhow::bail!("Could not ping driver after loading");
            }
        }

        log::info!("Driver initialized");
        Ok(driver)
    }

    pub unsafe fn ping(&self) -> bool {
        matches!(self.send_request(Request::Ping), Ok(Ok(Response::Ping)))
    }

    pub unsafe fn read_physical(&self, address: u64, buf: &mut [u8]) -> core::result::Result<(), KernelError> {
        // println!("Reading {:#X} into {:p}", address, buf);
        // std::thread::sleep(Duration::from_secs_f32(0.5));
        self.send_request(Request::ReadPhysical { address, buf: buf.as_mut_ptr(), len: buf.len() }).unwrap().map(|_| ())
    }

    const CHUNK_SIZE: usize = 0x1000;

    pub unsafe fn read_physical_chunked(&self, address: u64, buf: &mut [u8]) {
        if buf.len() <= Self::CHUNK_SIZE {
            let result = self.read_physical(address, buf);
            if let Err(e) = result {
                log::debug!("Error reading {:#X} - {:#X}: {:#X?}", address, address + buf.len() as u64, e);
                buf.fill(0);
            }

            return;
        }
        // let mut intermediate_buf = vec![0u8; CHUNK_SIZE];
        for (n, chunk) in buf.chunks_mut(Self::CHUNK_SIZE).enumerate() {
            let start = address + (n * Self::CHUNK_SIZE) as u64;
            log::trace!("Reading {:#X} - {:#X}", start, start + chunk.len() as u64);

            // let result = self.read_physical(start, &mut intermediate_buf[0..chunk.len()]);
            // chunk.copy_from_slice(&intermediate_buf[0..chunk.len()]);
            let result = self.read_physical(start, chunk);

            if let Err(e) = result {
                log::debug!("Error reading {:#X} - {:#X}: {:#X?}", start, start + chunk.len() as u64, e);
                chunk.fill(0);
            }
        }
    }

    pub unsafe fn write_physical(&self, address: u64, buf: &[u8]) -> core::result::Result<(), KernelError> {
        self.send_request(Request::WritePhysical { address, buf: buf.as_ptr(), len: buf.len() }).unwrap().map(|_| ())
    }

    pub unsafe fn call_hook(&self, ptr: *mut c_void) -> anyhow::Result<()> {
        let result = self.key.0.set_value(&self.key.1, &(ptr as u64));
        match result {
            Ok(_) => Err(anyhow::anyhow!("Success returned when writing reg key. Is the hook installed?")),
            Err(e) if e.raw_os_error() != Some(5) => Err(anyhow::anyhow!("Did not get permission denied error, instead got {:?}", e)),
            Err(_) => Ok(())
        }
    }

    pub unsafe fn send_request(&self, req: Request) -> anyhow::Result<core::result::Result<Response, KernelError>> {
        let mut dispatch = Dispatch { handled: false, data: Data::Request(req) };
        self.call_hook(&mut dispatch as *mut _ as _)?;
        match dispatch.data {
            Data::Request(_) => {
                Err(anyhow::anyhow!("Could not send request to kernel"))
            }
            Data::Response(r) => Ok(r)
        }
    }
}

impl memlib::kernel::PhysicalMemoryRead for DriverHandle {
    fn try_read_bytes_physical_into(&self, physical_address: u64, buffer: &mut [u8]) -> Option<()> {
        unsafe { self.read_physical(physical_address, buffer).ok() }
    }
}

impl memlib::kernel::PhysicalMemoryWrite for DriverHandle {
    fn try_write_bytes_physical(&self, physical_address: u64, buffer: &[u8]) -> Option<()> {
        unsafe { self.write_physical(physical_address, buffer).ok() }
    }
}
