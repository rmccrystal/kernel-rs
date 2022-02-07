#![allow(clippy::missing_safety_doc)]

use std::ffi::CString;
use std::os::raw::c_void;
use std::thread;
use std::time::Duration;
use rand::Rng;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS};
use winreg::RegKey;
use crate::shared::*;

pub struct Driver {
    key: String,
    reg_folder: RegKey,
}

unsafe impl Send for Driver {}

impl Clone for Driver {
    fn clone(&self) -> Self {
        unsafe {
            let mut reg_folder = std::mem::zeroed();
            std::ptr::copy(&self.reg_folder, &mut reg_folder, std::mem::size_of::<RegKey>());
            Self { key: self.key.clone(), reg_folder }
        }
    }
}

#[cfg(debug_assertions)]
const DRIVER_BYTES: &[u8] = include_bytes!("../../target/x86_64-pc-windows-msvc/debug/driver.dll").as_slice();
#[cfg(not(debug_assertions))]
const DRIVER_BYTES: &[u8] = include_bytes!("../../target/x86_64-pc-windows-msvc/release/driver.dll").as_slice();

impl Driver {
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
            key: KEY.to_string(),
            reg_folder,
        };

        // driver.unregister();
        // log::info!("Unregistered driver");
        // std::thread::sleep(Duration::from_millis(1000));
        // log::info!("sleep end");

        if !driver.ping() {
            log::info!("Could not ping driver, mapping");
            kdmapper::kdmapper(DRIVER_BYTES, false, true, false, 0, 0).ok_or_else(|| anyhow::anyhow!("Could not map driver"))?;
            if !driver.ping() {
                anyhow::bail!("Could not ping driver after loading");
            }
        }

        log::info!("Driver initialized");
        Ok(driver)
    }

    pub unsafe fn ping(&self) -> bool {
        self.send_request(Request::Ping) == Some(Ok(Response::Ping))
    }

    pub unsafe fn unregister(&self) {
        self.send_request(Request::Unregister);
    }

    pub unsafe fn read_physical(&self, address: u64, buf: &mut [u8]) -> core::result::Result<(), KernelError> {
        // println!("Reading {:#X} into {:p}", address, buf);
        // std::thread::sleep(Duration::from_secs_f32(0.5));
        self.send_request(Request::ReadPhysical { address, buf: buf.as_mut_ptr(), len: buf.len() }).expect("Could not send request to kernel").map(|_| ())
    }

    const CHUNK_SIZE: usize = 0x1000;

    pub unsafe fn read_physical_chunked(&self, address: u64, buf: &mut [u8]) {
        if buf.len() < Self::CHUNK_SIZE {
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
        self.send_request(Request::WritePhysical { address, buf: buf.as_ptr(), len: buf.len() }).expect("Could not send request to kernel").map(|_| ())
    }

    pub unsafe fn call_hook(&self, ptr: *mut c_void) -> Option<()> {
        let result = self.reg_folder.set_value(&self.key, &(ptr as u64));
        if result.is_err() { Some(()) } else { None }
    }

    pub unsafe fn send_request(&self, req: Request) -> Option<core::result::Result<Response, KernelError>> {
        let mut dispatch = Dispatch { handled: false, data: Data::Request(req) };
        let prev = dispatch.clone();
        self.call_hook(&mut dispatch as *mut _ as _)?;
        if dispatch == prev {
            dbg!(prev, dispatch);
            panic!("why");
        }
        match dispatch.data {
            Data::Request(_) => {
                #[cfg(debug_assertions)]
                log::error!("Could not send request to kernel. dispatch = {:?}", dispatch);
                None
            }
            Data::Response(r) => Some(r)
        }
    }
}

use memflow::prelude::*;

impl PhysicalMemory for Driver {
    fn phys_read_raw_iter<'a>(&mut self, data: CIterator<PhysicalReadData<'a>>, out_fail: &mut ReadFailCallback<'_, 'a>) -> memflow::prelude::Result<()> {
        unsafe {
            for MemData(addr, mut out) in data {
                self.read_physical_chunked(addr.address().to_umem(), out.as_bytes_mut());
            }
            Ok(())
        }
    }

    fn phys_write_raw_iter<'a>(&mut self, data: CIterator<PhysicalWriteData<'a>>, out_fail: &mut WriteFailCallback<'_, 'a>) -> memflow::prelude::Result<()> {
        unsafe {
            for MemData(addr, out) in data {
                self.write_physical(addr.address().to_umem(), out.as_bytes()).unwrap();
            }
            Ok(())
        }
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        PhysicalMemoryMetadata { readonly: false, ideal_batch_size: 0x1000, real_size: 0xFFFFFFFFFFFFFFFF, max_address: 0xFFFFFFFFFFFFFFFF_u64.into() }
    }

    fn set_mem_map(&mut self, mem_map: &[PhysicalMemoryMapping]) {
        dbg!(mem_map);
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        let _ = self.reg_folder.delete_value(&self.key);
    }
}