#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use crate::util::log::KernelLogger;
use log::*;

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
static GLOBAL: kernel_alloc::KernelAlloc = kernel_alloc::KernelAlloc;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

static LOG_LEVEL: LevelFilter = LevelFilter::Trace;

#[no_mangle]
pub extern "system" fn driver_entry() -> u32 {
    KernelLogger::init(LOG_LEVEL);

    info!("Hello world! 1+1={}", 1+1);

    0xdeadbeef
}
