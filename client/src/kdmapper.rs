use anyhow::*;
use log::*;
use crate::cleaner::clean_event_logs;
use std::time::Duration;
use std::thread;

mod ffi {
    #[link(name = "kdmapper", kind = "static")]
    extern "C" {
        pub fn map_driver_from_memory(data: *const u8, len: u64) -> u32;
    }
}

// #[cfg(debug_assertions)]
pub fn map_driver() -> Result<()> {
    debug!("Mapping driver");
    map_driver_from_bytes(include_bytes!("../../driver/target/x86_64-pc-windows-msvc/debug/driver.dll"))?;
    debug!("Finished mapping driver");

    thread::sleep(Duration::from_millis(1500));
    clean_event_logs().context("Could not clean event logs")?;

    crate::KernelHandle::new()

    Ok(())
}

// #[cfg(not(debug_assertions))]
// pub fn map_driver() -> Result<()> {
//     map_driver_from_bytes(include_bytes!("../../driver/target/x86_64-pc-windows-msvc/release/driver.dll"))
// }


pub fn map_driver_from_bytes(bytes: &[u8]) -> Result<()> {
    let result = unsafe { ffi::map_driver_from_memory(bytes.as_ptr(), bytes.len() as _) };

    match result {
        0 => Ok(()),
        1 => Err(anyhow!("Failed to load vulnerable driver (you probably need to run as admin)")),
        2 => Err(anyhow!("Failed to map driver")),
        _ => panic!("Received invalid response from kdmapper")
    }
}