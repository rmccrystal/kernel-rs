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

struct TestProcess(process::Child);

impl TestProcess {
    pub fn spawn() -> Self {
        let mut proc = Command::new("notepad.exe").spawn().unwrap();
        thread::sleep(Duration::from_millis(50));
        Self(proc)
    }

    fn name(&self) -> &'static str {
        "notepad.exe"
    }

    pub fn pid(&self) -> u64 {
        self.0.id() as _
    }
}

impl Drop for TestProcess {
    fn drop(&mut self) {
        self.0.kill().unwrap();
    }
}

#[test]
fn test_map_driver() {
    crate::kdmapper::map_driver().unwrap();
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
    let process = TestProcess::spawn();

    let modules = handle.module_info(process.pid()).unwrap();
    debug!("Found {} modules", modules.len());
    assert!(!modules.is_empty());
}

#[test]
fn test_peb_base() {
    let handle = get_handle();
    let process = TestProcess::spawn();

    let peb = handle.get_peb_address(process.pid()).unwrap();
    dbg!(peb);
    assert_ne!(peb, 0);
}

#[test]
fn test_read_memory() {
    let handle = get_handle();
    let process = TestProcess::spawn();

    let base = handle.module_info(process.pid())
        .unwrap()
        .iter()
        .find(|m| m.module_name.to_lowercase() == process.name())
        .expect("Could not find module base")
        .base_address;

    // Read the first 64 bytes from base
    let mut buf = vec![0u8; 64];
    handle.read_memory(process.pid(), base, &mut buf).unwrap();
    dbg!(buf);
}