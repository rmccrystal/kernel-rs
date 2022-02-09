use crate::KernelHandle;
use log::*;
use std::{process, thread};
use std::process::Command;
use std::time::Duration;
use memlib::*;

extern crate test;

use test::Bencher;

fn init() {
    let _ = env_logger::builder().is_test(true).filter_level(LevelFilter::Debug).try_init();
}

// Runs init() and gets a handle (or panics)
fn get_handle() -> KernelHandle {
    init();

    KernelHandle::new().unwrap()
}

struct Process {
    proc: process::Child,
    name: String,
}

impl Process {
    pub fn new(process_name: &str) -> Self {
        let mut proc = Command::new(&process_name).spawn().unwrap();
        thread::sleep(Duration::from_millis(50));

        Self { proc, name: process_name.to_owned() }
    }

    pub fn notepad() -> Self {
        Self::new("notepad.exe")
    }

    fn name(&self) -> &str {
        &self.name
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
fn test_create_handle() {
    init();

    KernelHandle::new().unwrap();
}

#[test]
fn test_modules() {
    let handle = get_handle();
    let notepad = Process::notepad();
    let proc = handle.attach_pid(notepad.pid()).unwrap();

    let modules = proc.get_module_list();
    debug!("Found {} modules", modules.len());
    assert!(!modules.is_empty());
}

#[test]
fn test_peb_base() {
    let handle = get_handle();
    let notepad = Process::notepad();
    let proc = handle.attach_pid(notepad.pid()).unwrap();

    let peb = proc.peb_base_address();
    dbg!(peb);
    assert_ne!(peb, 0);
}

#[test]
fn test_read_memory() {
    let handle = get_handle();
    let notepad = Process::notepad();
    let proc = handle.attach_pid(notepad.pid()).unwrap();

    let base = proc.get_main_module().base;

    // Read the first 64 bytes from base
    let mut buf = vec![0u8; 64];
    proc.try_read_bytes_into(base, &mut buf).unwrap();
    dbg!(buf);
}

#[test]
fn test_dump_module() {
    let handle = get_handle();
    let notepad = Process::notepad();
    let proc = handle.attach_pid(notepad.pid()).unwrap();

    let main_mod = proc.get_main_module();

    let mut buf = vec![0u8; main_mod.size as _];
    proc.try_read_bytes_into(main_mod.base, &mut buf).unwrap();
    assert_eq!(buf[0..2], [0x4Du8, 0x5A]);
}

#[test]
fn test_write_memory() {
    let handle = get_handle();
    let notepad = Process::notepad();
    let proc = handle.attach_pid(notepad.pid()).unwrap();

    let base = proc.get_main_module().base;

    let test_data = [1u8, 2, 3, 4, 5, 6];

    let mut orig_data = vec![0u8; test_data.len()];
    proc.try_read_bytes_into(base, &mut orig_data).unwrap();

    proc.try_write_bytes(base, &test_data).unwrap();

    let mut actual_data = vec![0u8; test_data.len()];
    proc.try_read_bytes_into(base, &mut actual_data).unwrap();

    proc.try_write_bytes(base, &orig_data).unwrap();

    assert_eq!(&test_data[..], &actual_data);
}

#[bench]
fn bench_read(b: &mut Bencher) {
    let handle = get_handle();
    let notepad = Process::notepad();
    let proc = handle.attach_pid(notepad.pid()).unwrap();
    let base = proc.get_main_module().base;

    let mut buf = [0u8; 0x100];
    b.iter(|| {
        proc.try_read_bytes_into(base, &mut buf);
    });
}

#[bench]
fn bench_create_handle(b: &mut Bencher) {
    init();

    b.iter(|| {
        KernelHandle::new().unwrap();
    });
}