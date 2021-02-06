use crate::KernelHandle;
use log::*;
use std::{process, thread};
use std::process::Command;
use std::time::Duration;

fn init() {
    let _ = env_logger::builder().is_test(true).filter_level(LevelFilter::Trace).try_init();
}

// Runs init() and gets a handle (or panics)
fn get_handle() -> KernelHandle {
    init();

    KernelHandle::new().unwrap()
}

struct Process{
    proc: process::Child,
    name: String
}

impl Process {
    pub fn new(process_name: &str) -> Self {
        let mut proc = Command::new(&process_name).spawn().unwrap();
        thread::sleep(Duration::from_millis(50));

        Self{proc, name: process_name.to_owned()}
    }

    pub fn notepad() -> Self {
        Self::new("notepad.exe")
    }

    fn name(&self) -> &str {
        &self.name
    }

    pub fn base(&self, handle: &KernelHandle) -> u64 {
        handle.module_info(self.pid())
            .unwrap()
            .iter()
            .find(|m| m.module_name.to_lowercase() == self.name())
            .expect("Could not find module base")
            .base_address
    }

    pub fn pid(&self) -> u64 {
        self.proc.id() as _
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.proc.kill().unwrap();
    }
}

#[test]
fn test_map_driver() {
    crate::kdmapper::map_driver().unwrap();
}

#[test]
fn test_clean_events() {
    crate::cleaner::clean_event_logs().unwrap();
}

#[test]
fn test_create_handle() {
    init();

    KernelHandle::new().unwrap();
}

#[test]
fn test_invalid_pid() {
    init();

    let handle = KernelHandle::new().unwrap();
    assert!(handle.write_memory(99999999, 0, &[0]).is_err());
}

#[test]
fn test_modules() {
    let handle = get_handle();
    let process = Process::notepad();

    let modules = handle.module_info(process.pid()).unwrap();
    debug!("Found {} modules", modules.len());
    assert!(!modules.is_empty());
}

#[test]
fn test_peb_base() {
    let handle = get_handle();
    let process = Process::notepad();

    let peb = handle.get_peb_address(process.pid()).unwrap();
    dbg!(peb);
    assert_ne!(peb, 0);
}

#[test]
fn test_process_bitness() {
    let handle = get_handle();
    let process = Process::notepad();

    let bitness = handle.get_process_bitness(process.pid()).unwrap();
    dbg!(bitness);
    assert_eq!(bitness, 64);

    let bitness = handle.get_process_bitness(14120).unwrap();
    dbg!(bitness);
    assert_eq!(bitness, 32);
}

#[test]
fn test_read_memory() {
    let handle = get_handle();
    let process = Process::notepad();

    let base = process.base(&handle);

    // Read the first 64 bytes from base
    let mut buf = vec![0u8; 64];
    handle.read_memory(process.pid(), base, &mut buf).unwrap();
    dbg!(buf);
}

#[test]
fn test_write_memory() {
    let handle = get_handle();
    let process = Process::notepad();

    let base = process.base(&handle);

    let test_data = [1u8, 2, 3, 4, 5, 6];

    handle.write_memory(process.pid(), base, &test_data).unwrap();

    let mut actual_data = vec![0u8; test_data.len()];
    handle.read_memory(process.pid(), base, &mut actual_data).unwrap();

    assert_eq!(&test_data[..], &actual_data);
}