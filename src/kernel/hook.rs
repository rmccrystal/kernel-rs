use alloc::boxed::Box;
use core::ffi::c_void;
use core::mem;
use core::ptr::copy;

use log::*;

use crate::kernel::safe_copy;

unsafe fn jump_to_address_bytecode(dest: *mut c_void) -> [u8; 12] {
    let mov_rax = [ 0x48_u8, 0xB8 ];
    let jmp_rax = [ 0xFF_u8, 0xE0 ];

    let mut opcodes: [u8; 12] = mem::zeroed();

    opcodes[0..=1].clone_from_slice(&mov_rax);
    opcodes[2..=9].clone_from_slice(&(dest as *const () as u64).to_le_bytes());
    opcodes[10..=11].clone_from_slice(&jmp_rax);

    opcodes
}

unsafe fn trampoline_bytecode(original_bytes: &[u8; 12], original_func: *mut c_void) -> [u8; 28] {
    let mut opcodes: [u8; 28] = mem::zeroed();

    // first 12 bytes of the opcodes are the bytes from the original function
    // that we overwrote with the hook
    opcodes[0..=11].clone_from_slice(&original_bytes[..]);

    // nop for 16 byte alignment
    opcodes[12..=15].clone_from_slice(&[0x90, 0x90, 0x90, 0x90]);

    // we want to jump to the the function address + 12 b/c our
    // 12 byte hook is at the original func address.

    // the location that we are going to jump to.
    let func_location = original_func as u64 + 12;
    let jmp_bytecode = jump_to_address_bytecode(func_location as _);
    opcodes[16..=27].clone_from_slice(&jmp_bytecode[..]);

    /*
     * opcodes[0-11] -> bytes of original function that was replaced by our hook
     * opcodes[12-15] -> nop so jmp can be aligned
     * opcodes[16-27] -> jump back to original function
     * Total: 28 bytes
     * once opcodes is allocated it should functionally be the same as our hooked function
     */

    opcodes
}

pub unsafe fn hook_function(address: *mut c_void, hook_fn: unsafe fn(*mut c_void)) -> *mut c_void {
    debug!("Hooking function {:p} with hook {:p}", address, hook_fn);
    // backup the original bytes
    let mut original_bytes: [u8; 12] = mem::zeroed();
    copy(hook_fn as _, original_bytes.as_mut_ptr(), 12);

    let original_func_bytecode = trampoline_bytecode(&original_bytes, address);
    let original_func = Box::new(original_func_bytecode);

    // create hook bytecode
    let hook_bytecode = jump_to_address_bytecode(hook_fn as _);

    if let Err(e) = safe_copy(hook_bytecode.as_ptr(), address as _, mem::size_of_val(&hook_bytecode)) {
        error!("Error copying hook: {:?}", e);
    };

    trace!("Original func bytecode at address {:p}: {:X?}", original_func.as_ptr(), original_func);

    Box::into_raw(original_func) as _
}