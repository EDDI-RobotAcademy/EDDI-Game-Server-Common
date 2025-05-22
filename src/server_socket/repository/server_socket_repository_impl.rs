use async_trait::async_trait;
use tokio::net::TcpListener;

use crate::server_socket::entity::server_socket::ServerSocket;
use crate::server_socket::repository::server_socket_repository::ServerSocketRepository;

pub struct ServerSocketRepositoryImpl;

impl ServerSocketRepositoryImpl {
    pub fn new() -> Self {
        ServerSocketRepositoryImpl
    }
}

#[async_trait]
impl ServerSocketRepository for ServerSocketRepositoryImpl {
    async fn bind(&self, addr: &str) -> Option<ServerSocket> {
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("[SERVER_SOCKET_REPO_IMPL] Bound to {}", addr);
                Some(ServerSocket::new(listener))
            }
            Err(e) => {
                eprintln!("[SERVER_SOCKET_REPO_IMPL] Failed to bind {}: {}", addr, e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_socket::repository::server_socket_repository::ServerSocketRepository;

    #[tokio::test]
    async fn test_bind_success() {
        // OS가 사용 가능한 포트를 자동으로 할당하도록 127.0.0.1:0 사용
        let repo = ServerSocketRepositoryImpl;
        let result = repo.bind("127.0.0.1:0").await;

        assert!(result.is_some(), "Expected successful binding");
        let server_socket = result.unwrap();

        let addr = server_socket.listener().local_addr().unwrap();
        println!("Server bound to: {}", addr);

        // 검증: 실제로 listener가 정상 바인딩된 상태
        assert!(addr.port() != 0);
    }

    #[tokio::test]
    async fn test_bind_failure() {
        // 의도적으로 잘못된 주소를 사용하여 바인딩 실패 유도
        let repo = ServerSocketRepositoryImpl;
        let result = repo.bind("256.0.0.1:9999").await;

        assert!(result.is_none(), "Expected binding failure due to invalid address");
    }
}
