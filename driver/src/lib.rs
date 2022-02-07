#![no_std]
#![feature(alloc_error_handler)]
#![allow(clippy::missing_safety_doc)]
#![allow(incomplete_features)]
#![feature(core_intrinsics)]
#![feature(option_result_contains)]

extern crate alloc;

use alloc::string::String;
use core::convert::TryInto;
use core::mem;

use log::*;
use winkernel::allocator::KernelAlloc;
use winkernel::basedef::{ntstatus, NTSTATUS, PVOID};
use winkernel::kernel::{create_registry_callback, get_kernel_modules, get_object_name, get_process_list, is_address_valid, is_valid_ptr, query_performance_counter, RegistryCallback, RegistryCallbackFunc, RegNotifyClass, RegSetValueKeyInformation, safe_copy};
use winkernel::log::KernelLogger;

pub mod dispatch;
pub mod shared;

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

static LOG_LEVEL: LevelFilter = LevelFilter::Debug;

pub struct Context {
    pub count: u64,
    pub callback: RegistryCallback,
}

static mut CONTEXT: Context = Context { count: 0, callback: RegistryCallback(0) };

unsafe extern "C" fn handler(ctx: &mut Context, class: RegNotifyClass, operation: *mut PVOID) -> NTSTATUS {
    let status = ntstatus::STATUS_SUCCESS;

    if class != RegNotifyClass::RegNtPreSetValueKey {
        return status;
    }

    let set_value = match (operation as *mut RegSetValueKeyInformation).as_ref() {
        Some(n) => n,
        None => return status
    };

    if !set_value.value_name.try_to_string().contains(&REGISTRY_KEY.as_deref().unwrap_or("TestKey1")) || !set_value.data().len() == 8 {
        return status;
    }

    let ptr = u64::from_ne_bytes(set_value.data().try_into().unwrap());
    trace!("Received registry buf: {:#X}", ptr);

    let result = dispatch::dispatch(ptr as _);

    if result {
        ntstatus::STATUS_ACCESS_DENIED
    } else {
        status
    }
}

static mut REGISTRY_KEY: Option<String> = None;

/*
unsafe fn init_registry_key(param1: usize) -> bool {
    if param1 == 0 {
        error!("param1 was 0");
        return false;
    }

    if !is_address_valid(param1) {
        error!("{:#X} is not a valid read address for REGISTRY_KEY", param1);
        return false;
    }

    if let Ok(key) = CStr::from_ptr(param1 as _).to_str() {
        REGISTRY_KEY = Some(key.to_string());
        info!("Found registry key {}", key.to_string());
        true
    } else {
        error!("Could not parse ptr for REGISTRY_KEY");
        false
    }
}
 */

unsafe fn main(param1: usize, param2: usize) -> Result<u32, NTSTATUS> {
    info!("kernel-rs loaded");
    // RegistryCallback(132803733244794080).unregister();
    // return Ok(0);

    // if !init_registry_key(param1) {
    //     return Err(-1);
    // }

    let vigembus = get_kernel_modules()?.into_iter().find(|n| n.full_path().ends_with("ViGEmBus.sys")).ok_or(1)?;

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
    bytecode[12..14].clone_from_slice(&add_offset.to_le_bytes());
    // let bytecode = [ 0xC3_u8 ];
    info!("bytecode: {:X?}", bytecode);

    safe_copy(bytecode.as_ptr(), codecave as _, bytecode.len())?;
    info!("codecave 2: {:X?}", core::ptr::read(codecave as *const [u8; 30]));

    let func: RegistryCallbackFunc<_> = mem::transmute(codecave);
    let callback = create_registry_callback(func, &mut CONTEXT)?;
    CONTEXT.callback = callback;

    // let status = func(&mut CONTEXT, RegNotifyClass::RegNtPreDeleteKey, null_mut());
    info!("callback: {}", callback.0);

    Ok(0)
}

#[no_mangle]
pub extern "system" fn driver_entry(param1: usize, param2: usize) -> u32 {
    if let Err(e) = KernelLogger::init(LOG_LEVEL, "kernel-rs") {
        error!("Error setting logger: {:?}", e);
    }

    match unsafe { main(param1, param2) } {
        Ok(code) => code,
        Err(err) => {
            error!("{:#X}", err as u32);
            err as _
        }
    }
}
