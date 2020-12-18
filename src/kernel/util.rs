use core::ffi::c_void;
use core::ptr::null_mut;

use log::*;

use crate::include::{_RTL_PROCESS_MODULES, ZwQuerySystemInformation, RTL_PROCESS_MODULE_INFORMATION};
use crate::util::VariableSizedBox;

use super::KernelError;
use super::ToKernelResult;
use cstr_core::CStr;

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
