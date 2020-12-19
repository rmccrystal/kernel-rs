use crate::include::{ObfDereferenceObject, PsLookupProcessByProcessId, PsGetProcessPeb, PEPROCESS, KeStackAttachProcess, _KAPC_STATE, KeUnstackDetachProcess, _LDR_DATA_TABLE_ENTRY};
use super::Result;
use super::ToKernelResult;
use crate::kernel::KernelError;
use log::*;
use crate::util::ListEntryIterator;
use winapi::_core::intrinsics::transmute;
use cstr_core::{CString, CStr};
use crate::util::string::unicode_string_to_string;
use alloc::string::String;

#[derive(Clone, Debug)]
pub struct ModuleInfo {
    pub base_address: u64,
    pub size: u64,
    pub module_name: String
}

pub struct Process {
    pub process: PEPROCESS,
}

impl Process {
    pub fn by_id(process_id: u64) -> Result<Self> {
        let mut process = core::ptr::null_mut();
        unsafe { PsLookupProcessByProcessId(process_id as _, &mut process).to_kernel_result()? };
        Ok(Self { process: process as _ })
    }

    pub fn get_module_info_64(&self, module_name: &str) -> Result<ModuleInfo> {
        let attach = unsafe { ProcessAttachment::attach(self.process) };
        let peb = unsafe { PsGetProcessPeb(self.process) };

        if peb.is_null() {
            return Err(KernelError::Message("peb was null"));
        }

        unsafe {
            let ldr = (*peb).Ldr;
            if peb.is_null() {
                return Err(KernelError::Message("peb ldr was null"));
            }

            let iter: ListEntryIterator<_LDR_DATA_TABLE_ENTRY, 0> = ListEntryIterator::new(&mut (*ldr).ModuleListLoadOrder);
            for entry in iter {
                let name = unicode_string_to_string(&entry.BaseDllName);
                if name.to_lowercase() == module_name.to_lowercase() {
                    return Ok(ModuleInfo{
                        base_address: entry.DllBase as _,
                        size: entry.SizeOfImage as _,
                        module_name: name
                    })
                }
                trace!("{}: {:?}", entry.BaseDllName.Length, name);
            }
        };

        Err(KernelError::Message("module not found"))
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if !self.process.is_null() {
            unsafe { ObfDereferenceObject(self.process as _) }
        }
    }
}

pub struct ProcessAttachment {
    process: PEPROCESS,
    state: _KAPC_STATE,
}

impl ProcessAttachment {
    pub unsafe fn attach(process: PEPROCESS) -> Self {
        let mut state: _KAPC_STATE = core::mem::zeroed();
        KeStackAttachProcess(process, &mut state as _);
        trace!("Attached to process");
        Self { process, state }
    }
}

impl Drop for ProcessAttachment {
    fn drop(&mut self) {
        unsafe { KeUnstackDetachProcess(&mut self.state as _) };
        trace!("Detached from process");
    }
}