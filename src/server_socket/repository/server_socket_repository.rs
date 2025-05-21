use async_trait::async_trait;
use crate::server_socket::entity::server_socket::ServerSocket;

#[async_trait]
pub trait ServerSocketRepository: Send + Sync {
    async fn bind(&self, addr: &str) -> Option<ServerSocket>;
}
