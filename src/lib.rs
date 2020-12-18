#![no_std]
#![feature(alloc_error_handler)]
#![feature(const_generics)]

extern crate alloc;

use crate::util::log::KernelLogger;
use log::*;
use crate::kernel::get_kernel_module;
use alloc::vec::Vec;

pub mod include;
pub mod kernel;
pub mod util;

/// When using the alloc crate it seems like it does some unwinding. Adding this
/// export satisfies the compiler but may introduce undefined behaviour when a
/// panic occurs.
#[no_mangle]
pub extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 { unimplemented!() }

/// Explanation can be found here: https://github.com/Trantect/win_driver_example/issues/4
#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

#[global_allocator]
static GLOBAL: util::alloc::KernelAlloc = util::alloc::KernelAlloc;
// static GLOBAL: kernel_alloc::KernelAlloc = kernel_alloc::KernelAlloc;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("panic: {:?}", info);
    loop {}
}

static LOG_LEVEL: LevelFilter = LevelFilter::Trace;

#[no_mangle]
pub extern "system" fn driver_entry() -> u32 {
    if let Err(e) = KernelLogger::init(LOG_LEVEL) {
        println!("Error setting logger: {:?}", e);
    }
    info!("kernel-rs loaded");

    // let result = unsafe { get_kernel_module("\\SystemRoot\\System32\\drivers\\dxgkrnl.sys") };
    // info!("{:?}", result);

    0xdeadbeef
}

