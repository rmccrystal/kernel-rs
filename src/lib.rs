#![no_std]
#![feature(alloc_error_handler)]
#![allow(clippy::missing_safety_doc)]
#![allow(incomplete_features)]
#![feature(core_intrinsics)]
#![feature(const_generics)]

extern crate alloc;

use core::intrinsics::abort;

use log::*;

use crate::kernel::{get_kernel_module_export, KernelError, get_kernel_modules, find_kernel_module};
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

unsafe fn main() -> Result<u32, KernelError> {
    info!("kernel-rs loaded");

    let modules = get_kernel_modules()?;

    let dxgkrnl = find_kernel_module(&modules, "dxgkrnl.sys").ok_or("could not find dxgkrnl")?;
    let address = get_kernel_module_export(dxgkrnl, "NtQueryCompositionSurfaceStatistics")
        .ok_or("could not find NtQueryCompositionSurfaceStatistics")?;

    kernel::hook_function(address, dispatch::hook);

    let win32k = find_kernel_module(&modules, "win32kfull.sys").ok_or("could not find win32k.sys")?;
    let nt_gdi_bit_blt = get_kernel_module_export(win32k, "NtGdiBitBlt").ok_or("could not find NtGdiBitBlt")?;
    info!("{:p}", nt_gdi_bit_blt);

    Ok(0x420)
}

#[no_mangle]
pub extern "system" fn driver_entry() -> u32 {
    if let Err(e) = KernelLogger::init(LOG_LEVEL) {
        error!("Error setting logger: {:?}", e);
    }

    match unsafe { main() } {
        Ok(code) => code,
        Err(err) => {
            error!("{:?}", err);
            1
        }
    }
}
