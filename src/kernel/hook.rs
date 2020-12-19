use core::ffi::c_void;
use core::mem;
use crate::kernel::safe_copy;
use log::*;

pub unsafe fn hook_function(address: *mut c_void, hook_fn: unsafe fn(*mut c_void)) {
    let mov_rax = [ 0x48_u8, 0xB8 ];
    let jmp_rax = [ 0xFF_u8, 0xE0 ];

    // let mut original_fn = [ 0x00_u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];
    let mut original_fn: [u8; 12] = mem::zeroed();

    original_fn[0..=1].clone_from_slice(&mov_rax);
    original_fn[2..=9].clone_from_slice(&(hook_fn as *const () as u64).to_le_bytes());
    original_fn[10..=11].clone_from_slice(&jmp_rax);

    if let Err(e) = safe_copy(original_fn.as_ptr(), address as _, mem::size_of_val(&original_fn)) {
        error!("Error copying hook: {:?}", e);
    }
}