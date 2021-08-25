mod service;
use futures::future;
use futures::{FutureExt, StreamExt};
use kernel_client::KernelHandle;
use service::Memory;
use std::sync::{Arc, Mutex, MutexGuard};
use tarpc::serde_transport;
use tarpc::server::incoming::Incoming;
use tarpc::server::Serve;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let address = "0.0.0.0:1414";
    let transport =
        serde_transport::tcp::listen(&address, tarpc::tokio_serde::formats::Bincode::default)
            .await?
            .filter_map(|r| future::ready(r.ok()));
    log::info!("kernel-rs server listening on {}", address);

    let handle = kernel_client::KernelHandle::new()?;
    let server = MemoryServer(Arc::new(Mutex::new(handle)));

    transport
        .map(tarpc::server::BaseChannel::with_defaults)
        .execute(server.serve())
        .await;

    Ok(())
}

#[derive(Clone)]
struct MemoryServer(Arc<Mutex<KernelHandle>>);

impl MemoryServer {
    fn handle(&self) -> MutexGuard<'_, KernelHandle> {
        self.0.lock().unwrap()
    }
}

#[tarpc::server]
impl Memory for MemoryServer {
    async fn ping(self, _: tarpc::context::Context) -> kernel_client::KernelResult<()> {
        self.handle().ping()
    }

    async fn module_info(
        self,
        _: tarpc::context::Context,
        pid: u64,
    ) -> kernel_client::KernelResult<Vec<kernel_client::ModuleInfo>> {
        self.handle().module_info(pid)
    }

    async fn get_peb_address(
        self,
        _: tarpc::context::Context,
        pid: u64,
    ) -> kernel_client::KernelResult<u64> {
        self.handle().get_peb_address(pid)
    }

    async fn read_memory(
        self,
        _: tarpc::context::Context,
        pid: u64,
        address: u64,
        len: u64,
    ) -> kernel_client::KernelResult<Vec<u8>> {
        let mut buf = vec![0u8; len as usize];
        self.handle().read_memory(pid, address, &mut buf)?;
        Ok(buf)
    }

    async fn write_memory(
        self,
        _: tarpc::context::Context,
        pid: u64,
        address: u64,
        buf: Vec<u8>,
    ) -> kernel_client::KernelResult<()> {
        self.handle().write_memory(pid, address, &buf)
    }

    async fn get_pid_by_name(self, _: tarpc::context::Context, name: String) -> Option<u64> {
        winutil::get_pid_by_name(&name).map(|n| n as u64)
    }
}
