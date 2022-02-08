#![cfg_attr(test, feature(test))]

use anyhow::*;
use memflow::os::Process;
use memflow::prelude::*;
use memflow::prelude::cache::TimedCacheValidator;
use memflow_win32::prelude::*;
use memlib::{Module, ProcessAttach};
use crate::driver::DriverHandle;

pub mod driver;
#[path = "../../driver/src/shared.rs"]
pub mod shared;

#[cfg(test)]
mod tests;

type MemflowKernel = Win32Kernel<CachedPhysicalMemory<'static, DriverHandle, TimedCacheValidator>, CachedVirtualTranslate<DirectTranslate, TimedCacheValidator>>;
type MemflowProcess = Win32Process<VirtualDma<CachedPhysicalMemory<'static, DriverHandle, TimedCacheValidator>, CachedVirtualTranslate<DirectTranslate, TimedCacheValidator>, Win32VirtualTranslate>>;

pub struct KernelHandle(MemflowKernel);

impl KernelHandle {
    pub fn new() -> anyhow::Result<Self> {
        let driver = unsafe { DriverHandle::new() }?;

        let kernel = Win32KernelBuilder::new(driver)
            .build_default_caches()
            .build()?;

        Ok(Self(kernel))
    }

    pub fn attach_pid(&self, pid: u64) -> anyhow::Result<KernelProcess> {
        Ok(KernelProcess(self.0.clone().into_process_by_pid(pid as _)?))
    }
}

impl Default for KernelHandle {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl memlib::ProcessAttach for KernelHandle {
    type ProcessType = KernelProcess;

    fn attach(&self, process_name: &str) -> anyhow::Result<Self::ProcessType> {
        Ok(KernelProcess(self.0.clone().into_process_by_name(process_name)?))
    }
}

pub struct KernelProcess(MemflowProcess);

impl KernelProcess {
    pub fn get_main_module(&self) -> Module {
        self.0.clone().primary_module()
            .map(|m| memlib::Module { name: m.name.to_string(), base: m.base.to_umem(), size: m.size })
            .unwrap()
    }
}

impl memlib::MemoryRead for KernelProcess {
    fn try_read_bytes_into(&self, address: u64, buffer: &mut [u8]) -> Option<()> {
        self.0.virt_mem
            .clone()
            .read_raw_into(Address::from(address), buffer)
            .ok()
    }
}

impl memlib::MemoryWrite for KernelProcess {
    fn try_write_bytes(&self, address: u64, buffer: &[u8]) -> Option<()> {
        self.0.virt_mem
            .clone()
            .write_raw(Address::from(address), buffer)
            .ok()
    }
}

impl memlib::ModuleList for KernelProcess {
    fn get_module_list(&self) -> Vec<Module> {
        self.0.clone().module_list().unwrap()
            .into_iter()
            .map(|m| memlib::Module { name: m.name.to_string(), base: m.base.to_umem(), size: m.size })
            .collect()
    }
}

impl memlib::ProcessInfo for KernelProcess {
    fn process_name(&self) -> String {
        self.0.clone().primary_module().unwrap().name.to_string()
    }

    fn peb_base_address(&self) -> u64 {
        self.0.clone().proc_info.peb().unwrap().to_umem()
    }
}
