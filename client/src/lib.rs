use anyhow::*;
use memflow::{CachedMemoryAccess, CachedVirtualTranslate, DirectTranslate, PhysicalMemory, TimedCacheValidator, VirtualMemory};
use memflow_win32::Kernel;
use memflow_win32::kernel_builder::KernelBuilder;
use crate::driver::Driver;

pub mod driver;
#[path = "../../driver/src/shared.rs"]
pub mod shared;

#[cfg(test)]
mod tests;

pub struct KernelHandle<'a> {
    pub kernel: Kernel<CachedMemoryAccess<'a, Driver, TimedCacheValidator>, CachedVirtualTranslate<DirectTranslate, TimedCacheValidator>>,
}

impl KernelHandle<'_> {
    pub fn new() -> Result<Self> {
        let driver = unsafe { Driver::new() }?;

        let mut kernel = KernelBuilder::new(driver)
            .build_default_caches()
            .build()?;

        dbg!(&kernel);
        let eprocs = kernel.eprocess_list().unwrap();
        let mut proc = kernel.process("explorer.exe").unwrap();

        Ok(Self { kernel })
    }
}
