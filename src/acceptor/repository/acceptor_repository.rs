use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};

#[async_trait]
pub trait AcceptorRepository: Send + Sync {
    async fn accept(&self, listener: &TcpListener) -> Option<TcpStream>;
}
