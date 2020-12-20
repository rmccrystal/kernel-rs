use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use cstr_core::{CStr, CString};
use log::*;
use serde::{Deserialize, Serialize};
use winapi::_core::intrinsics::transmute;
use winapi::km::wdm::KPROCESSOR_MODE::KernelMode;

use crate::include::{_KAPC_STATE, _LDR_DATA_TABLE_ENTRY, IoGetCurrentProcess, KeStackAttachProcess, KeUnstackDetachProcess, MmCopyVirtualMemory, MmIsAddressValid, ObfDereferenceObject, PEPROCESS, PPEB, PsGetProcessPeb, PsLookupProcessByProcessId};
use crate::kernel::KernelError;
use crate::util::ListEntryIterator;
use crate::util::string::unicode_string_to_string;

use super::Result;
use super::ToKernelResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub base_address: u64,
    pub size: u64,
    pub module_name: String,
}

#[derive(Clone, Debug)]
pub struct Process {
    pub process: PEPROCESS,
}

impl Process {
    pub fn by_id(process_id: u64) -> Result<Self> {
        let mut process = core::ptr::null_mut();
        unsafe { PsLookupProcessByProcessId(process_id as _, &mut process).to_kernel_result()? };
        Ok(Self { process: process as _ })
    }

    pub fn get_modules_64(&self) -> Result<Vec<ModuleInfo>> {
        let process = unsafe { ProcessAttachment::attach(self.process) };
        let peb = unsafe { process.get_peb() };

        if peb.is_null() {
            return Err(KernelError::text("peb was null"));
        }

        unsafe {
            let ldr = (*peb).Ldr;
            if peb.is_null() {
                return Err(KernelError::text("peb ldr was null"));
            }

            let iter: ListEntryIterator<_LDR_DATA_TABLE_ENTRY, 0> = ListEntryIterator::new(&mut (*ldr).ModuleListLoadOrder);

            Ok(iter.map(|entry| ModuleInfo {
                base_address: entry.DllBase as _,
                size: entry.SizeOfImage as _,
                module_name: unicode_string_to_string(&entry.BaseDllName),
            }).collect())
        }
    }

    pub fn read_memory(&self, address: u64, size: u64) -> Result<Box<[u8]>> {
        let _process = unsafe { ProcessAttachment::attach(self.process) };

        if !unsafe { MmIsAddressValid(address as _) } {
            return Err(KernelError::text(&format!("{:X} is not a valid address", address)));
        }

        if !unsafe { MmIsAddressValid((address + size - 1) as _) } {
            return Err(KernelError::text(
                &format!("{:X} was valid, but {:X} + {:X} (size) - 1 = {:X} was not", address, address, size, (address + size - 1))));
        }

        let mut buf = vec![0u8; size as _];

        let mut bytes_copied: u64 = 0;

        unsafe {
            MmCopyVirtualMemory(
                self.process,
                address as _,
                IoGetCurrentProcess(),
                buf.as_mut_ptr() as _,
                size as _,
                KernelMode as _,
                &mut bytes_copied as _,
            ).to_kernel_result()?;
        }

        if bytes_copied == 0 {
            return Err(KernelError::text("no bytes were copied"));
        }

        if bytes_copied < size {
            warn!("Reading {} bytes from {:?} only returned {} bytes", size, address, bytes_copied);
        }

        Ok(buf.into_boxed_slice())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if !self.process.is_null() {
            unsafe { ObfDereferenceObject(self.process as _) }
        }
    }
}

struct ProcessAttachment {
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

    pub unsafe fn get_peb(&self) -> PPEB {
        PsGetProcessPeb(self.process)
    }
}

impl Drop for ProcessAttachment {
    fn drop(&mut self) {
        unsafe { KeUnstackDetachProcess(&mut self.state as _) };
        trace!("Detached from process");
    }
}