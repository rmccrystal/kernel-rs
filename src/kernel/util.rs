use alloc::vec::Vec;
use core::ffi::c_void;
use core::ptr::null_mut;

use cstr_core::CStr;
use log::*;

use crate::include::{_RTL_PROCESS_MODULE_INFORMATION, _RTL_PROCESS_MODULES, PRTL_PROCESS_MODULES, RTL_PROCESS_MODULE_INFORMATION, ZwQuerySystemInformation};

use super::KernelError;
use super::ToKernelResult;

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


    let module_list: *mut _RTL_PROCESS_MODULES = Vec::with_capacity(size as _).as_mut_ptr();
    trace!("Allocated {:X} bytes", size);

    return Err(KernelError::Message("success"));

    ZwQuerySystemInformation(
        0x0B, // SystemModuleInformation
        module_list as _,
        size,
        &mut size
    ).to_kernel_result()?;


    /*
    let modules: *mut RTL_PROCESS_MODULE_INFORMATION = (*module_list).Modules.as_mut_ptr();

    let mut module_base = None;

    for i in 0..(*module_list).NumberOfModules as isize {
        let module = modules.offset(i);
        let name = CStr::from_ptr((*module).FullPathName.as_ptr() as _).to_str().unwrap();
        trace!("Found kernel module {}", name);

        if name == module_name {
            module_base = Some((*module).ImageBase)
        }
    }

    if let Some(base) = module_base {
        debug!("Found module base address for {}: {:p}", module_name, base);
    }

    module_base.ok_or(KernelError::Message("could not find module"))
     */
}
