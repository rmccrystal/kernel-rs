use core::alloc::{GlobalAlloc, Layout};
use crate::include::{ExAllocatePoolWithTag, _POOL_TYPE_NonPagedPool, ExFreePoolWithTag};

#[repr(C)]
pub enum PoolType {
    NonPagedPool,
    NonPagedPoolExecute,
}

#[link(name = "ntoskrnl")]
extern "system" {
    pub fn ExAllocatePool(pool_type: PoolType, number_of_bytes: usize) -> *mut core::ffi::c_void;
    pub fn ExFreePool(pool: u64);
}

static ALLOC_TAG: u32 = unsafe { core::mem::transmute(*b"test") };

/// The global kernel allocator structure.
pub struct KernelAlloc;

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pool = ExAllocatePool(PoolType::NonPagedPool, layout.size());

        if pool.is_null() {
            panic!("[kernel-alloc] failed to allocate pool.");
        }

        pool as _
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        ExFreePool(ptr as _);
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("{:?} alloc memory error", layout);
}

