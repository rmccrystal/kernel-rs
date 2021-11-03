#![no_std]
#![feature(alloc_error_handler)]
#![allow(clippy::missing_safety_doc)]
#![allow(incomplete_features)]
#![feature(core_intrinsics)]
#![feature(alloc_prelude)]
#![feature(option_result_contains)]

extern crate alloc;

use core::intrinsics::abort;
use alloc::prelude::v1::*;
use log::*;
use winkernel::allocator::KernelAlloc;
use winkernel::string::UnicodeString;
use winkernel::log::KernelLogger;
use winkernel::basedef::{ntstatus, NTSTATUS, PVOID};
use winkernel::dbg;
use winkernel::kernel::{get_kernel_modules, get_process_list, RegNotifyClass, create_registry_callback, safe_copy, RegistryCallbackFunc, RegistryCallback, RegSetValueKeyInformation, get_object_name, query_performance_counter};
use winkernel::basedef::winapi::ctypes::c_void;
use core::mem;
use core::ptr::null_mut;
use winkernel::process::Process;

/// When using the alloc crate it seems like it does some unwinding. Adding this
/// export satisfies the compiler but may introduce undefined behaviour when a
/// panic occurs.
#[no_mangle]
pub extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 { unimplemented!() }

#[global_allocator]
static GLOBAL: KernelAlloc = KernelAlloc;

/// Explanation can be found here: https://github.com/Trantect/win_driver_example/issues/4
#[used]
#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("panic: {:?}", info);
    #[allow(unused_unsafe)]
    loop {}
}

static LOG_LEVEL: LevelFilter = LevelFilter::Trace;

pub struct Context {
    pub count: u64,
    pub callback: RegistryCallback,
}

static mut CONTEXT: Context = Context { count: 0, callback: RegistryCallback(0) };

unsafe extern "C" fn handler(ctx: &mut Context, class: RegNotifyClass, operation: *mut PVOID) -> NTSTATUS {
    ctx.count += 1;

    let mut status = ntstatus::STATUS_SUCCESS;

    if ctx.count > 15000 {
        info!("unregistered");
        ctx.callback.unregister();
        return status;
    }

    // info!("class = {:?}", class);
    // return status;

    // if operation.is_null() {
    //     return status;
    // }

    // let object = *operation;
    // info!("{:p}", object);
    // return status;
    // let name = get_object_name(object);
    // if name.is_err() {
    //     return status;
    // }
    // let name = name.unwrap();

    // info!("name = {}", name);
    // return status;

    if class != RegNotifyClass::RegNtPreSetValueKey {
        return status;
    }

    let set_value = match (operation as *mut RegSetValueKeyInformation).as_ref() {
        None => return status,
        Some(n) => n
    };
    // info!("val = {:p}", set_value.object);
    // let name = get_object_name(set_value.object);
    // info!("name: {:?}", name);
    info!("{:?}", set_value);

    info!("data: {:?}", set_value.data());

    if set_value.value_name.try_to_string().contains(&"TestKey1") {
        info!("{}", Process::current().file_name());
        return ntstatus::STATUS_ACCESS_DENIED;
    }

    0
}

unsafe fn main() -> Result<u32, NTSTATUS> {
    info!("kernel-rs loaded");
    // RegistryCallback(132803733244794080).unregister();
    // RegistryCallback(132803733244794081).unregister();
    // return Ok(0);

    let vigembus = get_kernel_modules()?.into_iter().find(|n| n.full_path().ends_with("ViGEmBus.sys"));
    if vigembus.is_none() {
        error!("Could not find ViGEmBus.sys")
    }
    let vigembus = vigembus.unwrap();

    dbg!(vigembus.image_base);

    const OFFSET: usize = 0x10D2A;
    let codecave = vigembus.image_base + OFFSET;
    info!("codecave 1: {:X?}", core::ptr::read(codecave as *const [u8; 30]));

    let dest = handler as *const () as usize;
    let add_offset = query_performance_counter() as u16;
    let rax_start = dest.wrapping_sub(add_offset as _);
    // rax_start + add_offset = dest

    // FIXME: Do we need to store rax?
    let mut bytecode = [
        0x48_u8, 0xB8,  // mov rax, rax_start
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x48, 0x05, // add rax, add_offset
        0x00, 0x00, 0x00, 0x00,
        0xFF, 0xE0 // jmp rax
    ];
    bytecode[2..10].clone_from_slice(&rax_start.to_le_bytes());
    bytecode[12..16].clone_from_slice(&add_offset.to_le_bytes());
    info!("bytecode: {:X?}", bytecode);

    safe_copy(bytecode.as_ptr(), codecave as _, bytecode.len())?;

    let func: RegistryCallbackFunc<_> = mem::transmute(codecave);
    let callback = create_registry_callback(func, &mut CONTEXT);
    CONTEXT.callback = callback;

    // let status = func(&mut CONTEXT, RegNotifyClass::RegNtDeleteKey, null_mut());
    info!("callback: {}", callback.0);

    Ok(0)
}

#[no_mangle]
pub extern "system" fn driver_entry(driver_object: *mut c_void, _registry_path: *const UnicodeString) -> u32 {
    if let Err(e) = KernelLogger::init(LOG_LEVEL, "kernel-rs") {
        error!("Error setting logger: {:?}", e);
    }

    match unsafe { main() } {
        Ok(code) => code,
        Err(err) => {
            error!("{:#X}", err as u32);
            1
        }
    }
}
