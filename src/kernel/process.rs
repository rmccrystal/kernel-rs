use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use log::*;
use serde::{Deserialize, Serialize};
use winapi::km::wdm::KPROCESSOR_MODE::KernelMode;

use crate::include::{_KAPC_STATE, _LDR_DATA_TABLE_ENTRY, IoGetCurrentProcess, KeStackAttachProcess, KeUnstackDetachProcess, LDR_DATA_TABLE_ENTRY32, MmCopyVirtualMemory, MmIsAddressValid, ObfDereferenceObject, PEPROCESS, PPEB, PPEB32, PPEB_LDR_DATA32, PsGetProcessPeb, PsGetProcessWow64Process, PsLookupProcessByProcessId};
use crate::kernel::KernelError;
use crate::util::{ListEntryIterator, ListEntryIterator32, is_address_valid};

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

    pub fn get_modules(&self) -> Result<Vec<ModuleInfo>> {
        let modules_64 = self.get_modules_64();
        let modules_32 = self.get_modules_32();

        if let Err(err) = &modules_64 {
            if modules_32.is_err() {
                return Err(err.clone());
            }
        }

        let mut modules = Vec::new();
        if let Ok(m) = &modules_64 {
            modules.extend(m.clone());
        }
        if let Ok(m) = &modules_32 {
            modules.extend(m.clone());
        }

        Ok(modules)
    }

    fn get_modules_64(&self) -> Result<Vec<ModuleInfo>> {
        let _attach = self.attach();
        let peb = unsafe { _attach.get_peb() };

        if peb.is_null() {
            return Err(KernelError::text("peb was null"));
        }

        unsafe {
            let ldr = (*peb).Ldr;
            if ldr.is_null() {
                return Err(KernelError::text("peb ldr was null"));
            }

            let iter: ListEntryIterator<_LDR_DATA_TABLE_ENTRY, 0> = ListEntryIterator::new(&mut (*ldr).ModuleListLoadOrder);

            Ok(iter.map(|entry| ModuleInfo {
                base_address: entry.DllBase as _,
                size: entry.SizeOfImage as _,
                module_name: entry.BaseDllName.to_string(),
            }).collect())
        }
    }

    fn get_modules_32(&self) -> Result<Vec<ModuleInfo>> {
        let _attach = self.attach();
        let peb32: PPEB32 = unsafe { PsGetProcessWow64Process(self.process) as _ };

        if peb32.is_null() {
            return Err(KernelError::text("peb32 was null"));
        }

        unsafe {
            let ldr: PPEB_LDR_DATA32 = (*peb32).Ldr as _;
            if ldr.is_null() {
                return Err(KernelError::text("peb32 ldr was null"));
            }

            let iter: ListEntryIterator32<LDR_DATA_TABLE_ENTRY32, 0> = ListEntryIterator32::new(&mut (*ldr).InLoadOrderModuleList);

            Ok(iter.map(|entry| ModuleInfo {
                base_address: entry.DllBase as _,
                size: entry.SizeOfImage as _,
                module_name: entry.BaseDllName.to_string(),
            }).collect())
        }
    }

    pub fn read_memory(&self, address: u64, buf: &mut [u8]) -> Result<()> {
        let _attach = self.attach();

        if !is_address_valid(address as *const ()) {
            return Err(KernelError::text(&format!("{:X} is not a valid address", address)));
        }

        if !is_address_valid((address + buf.len() as u64 - 1) as *const ()) {
            return Err(KernelError::text(
                &format!("{:X} was valid, but {:X} + {:X} (size) - 1 = {:X} was not", address, address, buf.len(), (address + buf.len() as u64 - 1))));
        }

        let mut bytes_copied: u64 = 0;

        unsafe {
            MmCopyVirtualMemory(
                self.process,
                address as _,
                IoGetCurrentProcess(),
                buf.as_mut_ptr() as _,
                buf.len() as _,
                KernelMode as _,
                &mut bytes_copied as _,
            ).to_kernel_result()?;
        }

        if bytes_copied == 0 {
            return Err(KernelError::text("no bytes were copied"));
        }

        if bytes_copied < buf.len() as _ {
            warn!("Reading {} bytes from {:?} only returned {} bytes", buf.len(), address, bytes_copied);
        }

        Ok(())
    }

    pub fn write_memory(&self, address: u64, buf: &[u8]) -> Result<()> {
        let _attach = self.attach();

        if !is_address_valid(address as *const ()) {
            return Err(KernelError::text(&format!("{:X} is not a valid address", address)));
        }

        if !is_address_valid((address + buf.len() as u64 - 1) as *const ()) {
            return Err(KernelError::text(
                &format!("{:X} was valid, but {:X} + {:X} (size) - 1 = {:X} was not", address, address, buf.len(), (address + buf.len() as u64 - 1))));
        }

        let mut bytes_copied: u64 = 0;

        unsafe {
            MmCopyVirtualMemory(
                IoGetCurrentProcess(),
                buf.as_ptr() as _,
                self.process,
                address as _,
                buf.len() as _,
                KernelMode as _,
                &mut bytes_copied as _,
            ).to_kernel_result()?;
        }

        if bytes_copied == 0 {
            return Err(KernelError::text("no bytes were copied"));
        }

        if bytes_copied < buf.len() as u64 {
            warn!("Writing {} bytes to {:?} only wrote {} bytes", buf.len(), address, bytes_copied);
        }

        Ok(())
    }

    pub fn get_peb(&self) -> PPEB {
        unsafe { ProcessAttachment::attach(self.process).get_peb() }
    }

    pub fn attach(&self) -> ProcessAttachment {
        unsafe { ProcessAttachment::attach(self.process) }
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