#![allow(clippy::missing_safety_doc)]

use std::ffi::CString;
use std::os::raw::c_void;
use std::thread;
use std::time::Duration;
use anyhow::*;
use rand::distributions::Alphanumeric;
use rand::Rng;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS};
use winreg::RegKey;
use crate::shared::*;

pub struct Driver {
    key: String,
    reg_folder: RegKey,
}

#[cfg(debug_assertions)]
const DRIVER_BYTES: &[u8] = include_bytes!("../../target/x86_64-pc-windows-msvc/debug/driver.dll").as_slice();
#[cfg(not(debug_assertions))]
const DRIVER_BYTES: &[u8] = include_bytes!("../../target/x86_64-pc-windows-msvc/release/driver.dll").as_slice();

impl Driver {
    pub unsafe fn new() -> Result<Self> {
        // let key: String = rand::thread_rng()
        //     .sample_iter(&Alphanumeric)
        //     .take(10)
        //     .map(char::from)
        //     .collect();
        let key = "TestKey1".to_string();

        let key_cstr = CString::new(key.clone()).unwrap();

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let reg_folder = hklm.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\Dwm", winreg::enums::KEY_ALL_ACCESS)?;
        // let reg_key = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\Dwm")?;

        let driver = Self {
            key,
            reg_folder,
        };

        driver.unregister();

        if !driver.ping() {
            log::info!("Could not ping driver, mapping");
            kdmapper::kdmapper(DRIVER_BYTES, false, true, false, 0, 0).ok_or_else(|| anyhow!("Could not map driver"))?;
            if !driver.ping() {
                bail!("Could not ping driver after loading");
            }
        }

        Ok(driver)
    }

    pub unsafe fn ping(&self) -> bool {
        self.send_request(Request::Ping) == Some(Ok(Response::Ping))
    }

    pub unsafe fn unregister(&self) {
        self.send_request(Request::Unregister);
    }

    unsafe fn call_hook(&self, ptr: *mut c_void) -> Option<()> {
        let result = self.reg_folder.set_value(&self.key, &(ptr as u64));
        if result.is_err() { Some(()) } else { None }
    }

    pub unsafe fn send_request(&self, req: Request) -> Option<core::result::Result<Response, KernelError>> {
        let mut dispatch = Dispatch::Request(req);
        self.call_hook(&mut dispatch as *mut _ as _)?;
        match dispatch {
            Dispatch::Request(_) => {
                None
            }
            Dispatch::Response(r) => Some(r)
        }
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        // let _ = self.reg_folder.delete_value(&self.key);
    }
}