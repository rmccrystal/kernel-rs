use winapi::{km::wdm::PEPROCESS, shared::ntdef::NTSTATUS};

pub type PVOID = *mut core::ffi::c_void;
pub type HANDLE = PVOID;

extern "system" {
    pub fn MmIsAddressValid(virtual_address: PVOID) -> bool;
    pub fn PsLookupProcessByProcessId(process_id: HANDLE, process: *mut PEPROCESS) -> NTSTATUS;
    pub fn ObfDereferenceObject(object: PVOID);
}

// #[link(name = "gdi32")]
// extern "system" {
//     pub fn BitBlt(hdc: HDC, x: i32, y: i32, cx: i32, cy: i32, hdc_src: HDC, x1: i32, y1: i32, rop: u32) -> HRESULT;
// }