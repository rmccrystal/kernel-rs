#[tarpc::service]
pub trait Memory {
    async fn ping() -> kernel_client::KernelResult<()>;
    async fn module_info(pid: u64) -> kernel_client::KernelResult<Vec<kernel_client::ModuleInfo>>;
    async fn get_peb_address(pid: u64) -> kernel_client::KernelResult<u64>;
    async fn read_memory(pid: u64, address: u64, len: u64) -> kernel_client::KernelResult<Vec<u8>>;
    async fn write_memory(pid: u64, address: u64, buf: Vec<u8>) -> kernel_client::KernelResult<()>;
    async fn get_pid_by_name(name: String) -> Option<u64>;
}

