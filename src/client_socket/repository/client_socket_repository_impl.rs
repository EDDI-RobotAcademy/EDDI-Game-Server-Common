use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;

use crate::client_socket::entity::accepted_client_socket::AcceptedClientSocket;
use crate::client_socket::repository::client_socket_repository::ClientSocketRepository;

pub struct ClientSocketRepositoryImpl {
    clientHashMap: Arc<RwLock<HashMap<usize, AcceptedClientSocket>>>,
}

impl ClientSocketRepositoryImpl {
    pub fn new() -> Self {
        Self {
            clientHashMap: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ClientSocketRepository for ClientSocketRepositoryImpl {
    async fn register(&self, socket: AcceptedClientSocket) {
        let id = socket.id();
        let mut map = self.clientHashMap.write().await;
        map.insert(id, socket);
        println!("[REPO] Registered client: {}", id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_register_adds_client_to_repository() {
        let repo = ClientSocketRepositoryImpl::new();

        // 임시 listener 생성
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // 클라이언트 연결 비동기로 시뮬레이션
        let handle = tokio::spawn(async move {
            TcpStream::connect(addr).await.unwrap()
        });

        // 서버 측에서 stream 수락
        let (stream, _) = listener.accept().await.unwrap();
        let socket = AcceptedClientSocket::new(stream);
        let id = socket.id();

        repo.register(socket).await;

        // 내부 상태 확인
        let map = repo.clientHashMap.read().await;
        assert!(map.contains_key(&id));
        println!("[TEST] Client with id {} is registered", id);

        handle.await.unwrap(); // 클라이언트 종료 대기
    }
}
