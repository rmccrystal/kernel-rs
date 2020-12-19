#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

<<<<<<< Updated upstream
use crate::{include::MmIsAddressValid, process::Process, string::create_unicode_string};
use core::panic::PanicInfo;
=======

use log::*;

use crate::kernel::{get_kernel_module, get_kernel_module_export};
use crate::util::log::KernelLogger;
use core::ffi::c_void;
>>>>>>> Stashed changes

pub mod include;
pub mod log;
pub mod process;
pub mod string;

/// When using the alloc crate it seems like it does some unwinding. Adding this
/// export satisfies the compiler but may introduce undefined behaviour when a
/// panic occurs.
#[no_mangle]
pub extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 { unimplemented!() }

#[global_allocator]
static GLOBAL: kernel_alloc::KernelAlloc = kernel_alloc::KernelAlloc;

/// Explanation can be found here: https://github.com/Trantect/win_driver_example/issues/4
#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

#[panic_handler]
<<<<<<< Updated upstream
fn panic(_info: &PanicInfo) -> ! { loop {} }

#[no_mangle]
pub extern "system" fn driver_entry() -> u32 {
    // MmIsAddressValid
    //
    let is_valid = unsafe { MmIsAddressValid(0 as _) };
    log!("MmIsAddressValid(0) returned %i", is_valid as u64);

    // String
    //
    let string = create_unicode_string(obfstr::wide!("Hello World!\0"));
    log!("String: %ws", string.Buffer);

    // Process
    //
    let process = Process::by_id(4 as _);
    log!("Process found: %i", process.is_some() as u64);

    // kernel-print
    //
    kernel_print::kernel_dbg!(2 + 2);
    kernel_print::kernel_print!("{} + {} = {}\n", 2, 2, 2 + 2);
    kernel_print::kernel_println!("{} + {} = {}", 2, 2, 2 + 2);

    0 /* STATUS_SUCCESS */
=======
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("panic: {:?}", info);
    loop {}
}

fn hook(data: *mut c_void) {
    info!("hook!");
}

static LOG_LEVEL: LevelFilter = LevelFilter::Trace;

#[no_mangle]
pub extern "system" fn driver_entry() -> u32 {
    if let Err(e) = KernelLogger::init(LOG_LEVEL) {
        println!("Error setting logger: {:?}", e);
    }
    info!("kernel-rs loaded");

    let result = unsafe { get_kernel_module_export("\\SystemRoot\\System32\\drivers\\dxgkrnl.sys", "NtQueryCompositionSurfaceStatistics") };
    info!("{:?}", result);
    if result.is_err() {
        return 1;
    }
    let address = result.unwrap();

    unsafe { kernel::hook_function(address, hook) };

    0xdeadbeef
>>>>>>> Stashed changes
}
