use core::ffi::c_void;
use core::ptr::null_mut;

use log::*;

use crate::include::{_RTL_PROCESS_MODULES, ZwQuerySystemInformation, RTL_PROCESS_MODULE_INFORMATION, RtlFindExportedRoutineByName, IoAllocateMdl, MmProbeAndLockPages, _LOCK_OPERATION_IoReadAccess, MmMapLockedPagesSpecifyCache, _MEMORY_CACHING_TYPE_MmNonCached, MmProtectMdlSystemAddress, MmUnmapLockedPages, MmUnlockPages, IoFreeMdl};
use crate::util::VariableSizedBox;

use super::KernelError;
use super::ToKernelResult;
use cstr_core::{CStr, CString};
use winapi::shared::ntdef::FALSE;
use winapi::km::wdm::KPROCESSOR_MODE::KernelMode;

pub unsafe fn safe_copy<T>(src: *const T, dst: *mut T, count: usize) -> Result<(), KernelError> {
    let mdl = IoAllocateMdl(dst as _, count as _, FALSE, FALSE, null_mut());
    if mdl.is_null() {
        return Err(KernelError::Message("could not allocate mdl"));
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

    core::ptr::copy_nonoverlapping(src, dst, count);

    MmUnmapLockedPages(map, mdl);
    MmUnlockPages(mdl);
    IoFreeMdl(mdl);

    Ok(())
}

pub unsafe fn get_kernel_module(module_name: &str) -> Result<*mut c_void, KernelError> {
    // get size of system information
    let mut size = 0;
    ZwQuerySystemInformation(
        0x0B, // SystemModuleInformation
        null_mut(),
        size,
        &mut size
    );

    if size == 0 {
        return Err(KernelError::Message("getting ZwQuerySystemInformation size failed"));
    }
    trace!("Found ZwQuerySystemInformation size: {:X}", size);

    let mut buf: VariableSizedBox<_RTL_PROCESS_MODULES> = VariableSizedBox::new(size as _);
    trace!("Allocated {:X} bytes", size);

    ZwQuerySystemInformation(
        0x0B, // SystemModuleInformation
        buf.as_mut_ptr() as _,
        size,
        &mut size
    ).to_kernel_result()?;

    let module_list = buf.as_ref();

    trace!("Found {} modules", module_list.NumberOfModules);

    let modules = core::slice::from_raw_parts(module_list.Modules.as_ptr(), module_list.NumberOfModules as _);

    let mut module_base = None;

    for module in modules {
        let name = CStr::from_ptr(module.FullPathName.as_ptr() as _).to_str().unwrap();
        trace!("Found kernel module {}", name);

        if name == module_name {
            module_base = Some(module.ImageBase)
        }
    }

    if let Some(base) = module_base {
        debug!("Found module base address for {}: {:p}", module_name, base);
    }

    module_base.ok_or(KernelError::Message("could not find module"))
}

pub unsafe fn get_kernel_module_export(module_name: &str, func_name: &str) -> Result<*mut c_void, KernelError> {
    let module = get_kernel_module(module_name)?;
    let addr = RtlFindExportedRoutineByName(module, CString::new(func_name).unwrap().as_ptr());
    if addr.is_null() {
        Err(KernelError::Message("could not find module"))
    } else {
        debug!("Found address for {}: {:p}", func_name, addr);
        Ok(addr)
    }
}