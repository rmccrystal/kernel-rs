use anyhow::*;
use memflow::prelude::*;
use memflow::prelude::cache::TimedCacheValidator;
use memflow_win32::prelude::*;
use pelite::pe::exports::GetProcAddress;
use crate::driver::Driver;

pub mod driver;
#[path = "../../driver/src/shared.rs"]
pub mod shared;

#[cfg(test)]
mod tests;

pub struct KernelHandle {
    pub kernel: Win32Kernel<CachedPhysicalMemory<'static, Driver, TimedCacheValidator>, CachedVirtualTranslate<DirectTranslate, TimedCacheValidator>>,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub base_address: u64,
    pub size: u64,
}

impl KernelHandle {
    pub fn new() -> anyhow::Result<Self> {
        let driver = unsafe { Driver::new() }?;

        let kernel = Win32KernelBuilder::new(driver)
            .build_default_caches()
            .build()?;

        Ok(Self { kernel })
    }

    pub fn get_modules(&self, pid: u64) -> anyhow::Result<Vec<ModuleInfo>> {
        let mods = self.kernel.clone().into_process_by_pid(pid as _)?
            .module_list()?
            .into_iter()
            .map(|m| ModuleInfo { base_address: m.base.to_umem() as _, name: m.name.to_string(), size: m.size as _ })
            .collect();

        Ok(mods)
    }

    pub fn read_memory(&self, pid: u64, address: u64, buf: &mut [u8]) -> anyhow::Result<()> {
        self.kernel.clone().into_process_by_pid(pid as _)?
            .virt_mem
            .read_raw_into(Address::from(address), buf)
            .map_err(|e| anyhow!("{}", e.as_str()))
    }

    pub fn write_memory(&self, pid: u64, address: u64, buf: &[u8]) -> anyhow::Result<()> {
        self.kernel.clone().into_process_by_pid(pid as _)?
            .virt_mem
            .write_raw(Address::from(address), buf)
            .map_err(|e| anyhow!("{}", e.as_str()))
    }

    pub fn get_peb_base(&self, pid: u64) -> anyhow::Result<u64> {
        self.kernel.clone().into_process_by_pid(pid as _)?
            .proc_info
            .peb()
            .ok_or_else(|| anyhow!("Could not find peb"))
            .map(|a| a.to_umem() as u64)
    }
}
