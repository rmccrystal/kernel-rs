use core::alloc::{GlobalAlloc, Layout};


use crate::include::{ExAllocatePoolWithTag, ExFreePoolWithTag, _POOL_TYPE};

static ALLOC_TAG: u32 = unsafe { core::mem::transmute(*b"test") };

/// The global kernel allocator structure.
pub struct KernelAlloc;

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pool = ExAllocatePoolWithTag(_POOL_TYPE::NonPagedPool, layout.size() as _, ALLOC_TAG);


        if pool.is_null() {
            panic!("[kernel-alloc] failed to allocate pool.");
        }

        pool as _
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        ExFreePoolWithTag(ptr as _, ALLOC_TAG);
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("{:?} alloc memory error", layout);
}
