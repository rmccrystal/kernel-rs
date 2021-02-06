#![no_std]
#![feature(alloc_error_handler)]
#![allow(clippy::missing_safety_doc)]
#![allow(incomplete_features)]
#![feature(core_intrinsics)]
#![feature(const_generics)]

extern crate alloc;

use core::intrinsics::abort;


use log::*;

use crate::include::{PDRIVER_OBJECT, PUNICODE_STRING};
use crate::kernel::{find_kernel_module, get_kernel_module_export, get_kernel_modules, KernelError};
use crate::util::{KernelAlloc, is_address_valid};
use crate::util::log::KernelLogger;


pub mod include;
pub mod kernel;
#[macro_use]
pub mod util;
pub mod dispatch;
pub mod hooks;
pub mod interop;

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

unsafe fn main() -> Result<u32, KernelError> {
    info!("kernel-rs loaded");

    // info!("Cleaning piddb cache");
    // interop::bindings::clean_piddb_cache();

    let modules = get_kernel_modules()?;

    let dxgkrnl = find_kernel_module(&modules, "dxgkrnl.sys").ok_or("could not find dxgkrnl")?;
    let address = get_kernel_module_export(dxgkrnl, "NtQueryCompositionSurfaceStatistics")
        .ok_or("could not find NtQueryCompositionSurfaceStatistics")?;

    kernel::hook_function(address, dispatch::hook);

    // hooks::init_hooks()?;

    Ok(0)
}

#[no_mangle]
pub extern "system" fn driver_entry(driver_object: PDRIVER_OBJECT, _registry_path: PUNICODE_STRING) -> u32 {
    if let Err(e) = KernelLogger::init(LOG_LEVEL) {
        error!("Error setting logger: {:?}", e);
    }

    // Only set driver unload if we're not manual mapping
    if is_address_valid(driver_object) {
        unsafe { (*driver_object).DriverUnload = Some(driver_unload) };
    }

    match unsafe { main() } {
        Ok(code) => code,
        Err(err) => {
            error!("{:?}", err);
            1
        }
    }
}

pub unsafe extern "C" fn driver_unload(_driver_object: PDRIVER_OBJECT) {
    info!("kernel-rs unloaded");
}

