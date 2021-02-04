use core::ffi::c_void;
use core::ptr::null_mut;

use cstr_core::{CStr, CString};
use log::*;
use winapi::km::wdm::KPROCESSOR_MODE::KernelMode;
use winapi::shared::ntdef::FALSE;

use crate::include::*;
use crate::util::{VariableSizedBox, is_address_valid};
use super::Result;

use super::KernelError;
use super::ToKernelResult;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::include::_SYSTEM_INFORMATION_CLASS;

pub unsafe fn safe_copy(src: *const u8, dst: *mut u8, len: usize) -> Result<()> {
    let mdl = IoAllocateMdl(dst as _, len as _, FALSE, FALSE, null_mut());
    if mdl.is_null() {
        return Err(KernelError::text("could not allocate mdl"));
    }

    MmProbeAndLockPages(mdl, KernelMode as _, _LOCK_OPERATION::IoReadAccess);
    let map = MmMapLockedPagesSpecifyCache(
        mdl,
        KernelMode as _,
        _MEMORY_CACHING_TYPE::MmNonCached,
        null_mut(),
        FALSE as u32,
        16, // NormalPagePriority
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

pub unsafe fn query_system_information<T>(info_class: _SYSTEM_INFORMATION_CLASS) -> Result<VariableSizedBox<T>> {
    // get size of system information
    let mut size = 0;
    ZwQuerySystemInformation(
        info_class as _,
        null_mut(),
        size,
        &mut size,
    );

    if size == 0 {
        return Err(KernelError::text("getting size failed"));
    }

    let mut buf: VariableSizedBox<T> = VariableSizedBox::new(size as _);

    ZwQuerySystemInformation(
        info_class as _,
        buf.as_mut_ptr() as _,
        size,
        &mut size,
    ).to_kernel_result()?;

    trace!("ZwQuerySystemInformation size: {:X} bytes, info_class: {}", size, info_class as i32);

    Ok(buf)
}

pub struct KernelModule {
    pub name: String,
    pub address: *mut c_void,
}

pub unsafe fn get_kernel_modules() -> Result<Vec<KernelModule>> {
    let buf = query_system_information::<_RTL_PROCESS_MODULES>(_SYSTEM_INFORMATION_CLASS::SystemModuleInformation as _)?;

    let module_list = buf.as_ref();

    trace!("Found {} modules", module_list.NumberOfModules);

    let modules = core::slice::from_raw_parts(module_list.Modules.as_ptr(), module_list.NumberOfModules as _);

    Ok(modules.iter().map(|module| KernelModule {
        name: CStr::from_ptr(module.FullPathName.as_ptr() as _).to_str().unwrap().to_string(),
        address: module.ImageBase,
    }).collect())
}

pub struct ProcessInfo {
    pub name: String,
    pub pid: u64,
    pub number_of_threads: u32,
    pub base_priority: u32
}

pub unsafe fn get_process_list() -> Result<Vec<ProcessInfo>> {
    let buf = query_system_information::<SYSTEM_PROCESS_INFO>(_SYSTEM_INFORMATION_CLASS::SystemProcessInformation)?;

    let mut info = buf.as_ptr();
    let information_structs = {
        let mut info_structs = Vec::new();
        loop {
            info_structs.push(ProcessInfo{
                name: (*info).ImageName.to_string(),
                pid: (*info).ProcessId as _,
                number_of_threads: (*info).NumberOfThreads,
                base_priority: (*info).BasePriority
            });

            // increment
            let offset = (*info).NextEntryOffset;
            if offset == 0 {
                break info_structs;
            } else {
                info = (info as usize + offset as usize) as _;
            }
        }
    };

    Ok(information_structs)
}

pub fn find_kernel_module(modules: &[KernelModule], module_name: &str) -> Option<*mut c_void> {
    Some(modules
        .iter()
        .find(|&module| module.name.contains(&module_name))?
        .address)
}

pub fn get_kernel_module_export(module_base: *mut c_void, func_name: &str) -> Option<*mut c_void> {
    if !is_address_valid(module_base) {
        error!("Tried to get the export {} from module base {:p} but the module base was not valid", func_name, module_base);
        return None;
    }
    let addr = unsafe { RtlFindExportedRoutineByName(module_base, CString::new(func_name).unwrap().as_ptr()) };

    if addr.is_null() {
        None
    } else {
        debug!("Found address for {}: {:p}", func_name, addr);
        Some(addr)
    }
}