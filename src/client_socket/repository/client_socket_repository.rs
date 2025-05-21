use async_trait::async_trait;
use tokio::net::TcpStream;
use crate::client_socket::entity::accepted_client_socket::AcceptedClientSocket;

#[async_trait]
pub trait ClientSocketRepository: Send + Sync {
    async fn register(&self, socket: AcceptedClientSocket);
}
