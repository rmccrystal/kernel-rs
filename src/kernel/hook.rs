use core::ffi::c_void;
use core::ptr;
use core::mem;
use crate::kernel::safe_copy;
use log::*;

pub unsafe fn hook_function(address: *mut c_void, hook_fn: fn(*mut c_void)) {
    let address: *mut *mut c_void = core::mem::transmute(address);

    let mov_rax = [ 0x48_u8, 0xB8 ];
    let jmp_rax = [ 0xFF_u8, 0xE0 ];

    // let mut original_fn = [ 0x00_u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
    let mut original_fn: [u8; 12] = mem::zeroed();

    // RtlSecureZeroMemory(original_fn.as_mut_ptr() as _, original_fn.len());

    ptr::copy(mov_rax.as_ptr(), original_fn.as_mut_ptr(), mov_rax.len());

    let hook_fn_addr = hook_fn as *const () as usize;

    ptr::copy(hook_fn_addr as _, original_fn.as_mut_ptr().add(mov_rax.len()), mem::size_of_val(&hook_fn_addr));
    ptr::copy(jmp_rax.as_ptr(), original_fn.as_mut_ptr().add(mov_rax.len() + mem::size_of_val(&hook_fn_addr)), jmp_rax.len());

    info!("{:?}", original_fn);
    // safe_copy(original_fn.as_ptr(), hook_fn as _, mem::size_of_val(&original_fn));
}