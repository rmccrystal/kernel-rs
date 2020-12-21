#![no_std]
#![feature(alloc_error_handler)]
#![allow(clippy::missing_safety_doc)]
#![allow(incomplete_features)]
#![feature(core_intrinsics)]
#![feature(const_generics)]

extern crate alloc;

use core::intrinsics::abort;

use log::*;

use crate::kernel::{get_kernel_module_export};
use crate::util::KernelAlloc;
use crate::util::log::KernelLogger;

pub mod include;
pub mod kernel;
pub mod util;
pub mod dispatch;

/// When using the alloc crate it seems like it does some unwinding. Adding this
/// export satisfies the compiler but may introduce undefined behaviour when a
/// panic occurs.
#[no_mangle]
pub extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 { unimplemented!() }

#[global_allocator]
static GLOBAL: KernelAlloc = KernelAlloc;

/// Explanation can be found here: https://github.com/Trantect/win_driver_example/issues/4
#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("panic: {:?}", info);
    #[allow(unused_unsafe)]
    unsafe { abort() }
}

static LOG_LEVEL: LevelFilter = LevelFilter::Trace;


#[no_mangle]
pub extern "system" fn driver_entry() -> u32 {
    if let Err(e) = KernelLogger::init(LOG_LEVEL) {
        println!("Error setting logger: {:?}", e);
    }
    info!("kernel-rs loaded");

    let result = unsafe { get_kernel_module_export("\\SystemRoot\\System32\\drivers\\dxgkrnl.sys", "NtQueryCompositionSurfaceStatistics") };
    if result.is_err() {
        return 1;
    }
    let address = result.unwrap();

    unsafe { kernel::hook_function(address, dispatch::hook) };

    0
}
