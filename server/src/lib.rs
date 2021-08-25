use tarpc::serde_transport;
use tokio::net::ToSocketAddrs;

mod service;
pub use service::MemoryClient;
pub use tarpc::context;

pub async fn connect(addr: impl ToSocketAddrs) -> anyhow::Result<service::MemoryClient> {
    let transport = serde_transport::tcp::connect(&addr, tarpc::tokio_serde::formats::Bincode::default).await?;
    let client = service::MemoryClient::new(Default::default(), transport).spawn();

    Ok(client)
}