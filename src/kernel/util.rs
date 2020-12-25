use core::ffi::c_void;
use core::ptr::null_mut;

use cstr_core::{CStr, CString};
use log::*;
use winapi::km::wdm::KPROCESSOR_MODE::KernelMode;
use winapi::shared::ntdef::FALSE;

use crate::include::*;
use crate::util::VariableSizedBox;

use super::KernelError;
use super::ToKernelResult;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub unsafe fn safe_copy(src: *const u8, dst: *mut u8, len: usize) -> Result<(), KernelError> {
    let mdl = IoAllocateMdl(dst as _, len as _, FALSE, FALSE, null_mut());
    if mdl.is_null() {
        return Err(KernelError::text("could not allocate mdl"));
    }

    MmProbeAndLockPages(mdl, KernelMode as _, _LOCK_OPERATION_IoReadAccess);
    let map = MmMapLockedPagesSpecifyCache(
        mdl,
        KernelMode as _,
        _MEMORY_CACHING_TYPE_MmNonCached,
        null_mut(),
        FALSE as u32,
        16 // NormalPagePriority
    );

    MmProtectMdlSystemAddress(mdl, 0x04 /* PAGE_READWRITE */).to_kernel_result()?;

    core::ptr::copy_nonoverlapping(src, map as _, len);
    {
        let bytes = core::slice::from_raw_parts(src, len);
        trace!("Copied {:X?} to {:p}", bytes, dst);
    }

    MmUnmapLockedPages(map, mdl);
    MmUnlockPages(mdl);
    IoFreeMdl(mdl);

    Ok(())
}

pub struct KernelModule {
    pub name: String,
    pub address: *mut c_void
}

pub unsafe fn get_kernel_modules() -> Result<Vec<KernelModule>, KernelError> {
    // get size of system information
    let mut size = 0;
    ZwQuerySystemInformation(
        0x0B, // SystemModuleInformation
        null_mut(),
        size,
        &mut size
    );

    if size == 0 {
        return Err(KernelError::text("getting ZwQuerySystemInformation size failed"));
    }

    let mut buf: VariableSizedBox<_RTL_PROCESS_MODULES> = VariableSizedBox::new(size as _);

    ZwQuerySystemInformation(
        0x0B, // SystemModuleInformation
        buf.as_mut_ptr() as _,
        size,
        &mut size
    ).to_kernel_result()?;

    let module_list = buf.as_ref();

    trace!("Found {} modules", module_list.NumberOfModules);

    let modules = core::slice::from_raw_parts(module_list.Modules.as_ptr(), module_list.NumberOfModules as _);

    Ok(modules.iter().map(|module| KernelModule {
        name: CStr::from_ptr(module.FullPathName.as_ptr() as _).to_str().unwrap().to_string(),
        address: module.ImageBase
    }).collect())
}

pub unsafe fn find_kernel_module(modules: &[KernelModule], module_name: &str) -> Option<*mut c_void> {
    Some(modules
        .iter()
        .find(|&module| module.name.contains(&module_name))?
        .address)
}

pub unsafe fn get_kernel_module_export(module_base: *mut c_void, func_name: &str) -> Option<*mut c_void> {
    if !MmIsAddressValid(module_base) {
        error!("Tried to get the export {} from module base {:p} but the module base was not valid", func_name, module_base);
        return None;
    }
    let addr = RtlFindExportedRoutineByName(module_base, CString::new(func_name).unwrap().as_ptr());
    if addr.is_null() {
        None
    } else {
        debug!("Found address for {}: {:p}", func_name, addr);
        Some(addr)
    }
}