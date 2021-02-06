use anyhow::*;
use log::*;
use winapi::um::winnt::{HANDLE, LPCSTR};
use std::ptr::null_mut;
use winapi::shared::minwindef::BOOL;
use std::io;

extern "C" {
    fn ClearEventLogA(hEventLog: HANDLE, lpBackupFileName: LPCSTR) -> BOOL;
    fn OpenEventLogA(service_name: LPCSTR, source_name: LPCSTR) -> HANDLE;
}

macro_rules! c_string {
    ($str:expr) => {
        std::ffi::CString::new($str).unwrap().as_ptr()
    };
}

pub fn clean_event_logs() -> Result<()> {
    debug!("Cleaning event logs");
    unsafe {
        let handle = OpenEventLogA(null_mut(), c_string!("System"));
        if handle.is_null() {
            bail!("Could not open event log: {}", io::Error::last_os_error());
        }
        let success = ClearEventLogA(handle, null_mut());
        if success == 0 {
            bail!("Could not clear event log: {}", io::Error::last_os_error());
        }
    }

    debug!("Cleaned event logs");

    Ok(())
}