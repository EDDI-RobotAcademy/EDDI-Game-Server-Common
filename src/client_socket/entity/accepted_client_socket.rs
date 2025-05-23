use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

static CLIENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub struct AcceptedClientSocket {
    pub id: usize,
    pub user_token: Option<String>,
    pub reader: OwnedReadHalf,
    pub writer: OwnedWriteHalf,
}

impl AcceptedClientSocket {
    /// ID는 내부에서 자동 증가하며 user_token은 None으로 초기화
    pub fn new(stream: TcpStream) -> Self {
        let id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let (reader, writer) = stream.into_split();
        Self {
            id,
            user_token: None,
            reader,
            writer,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn set_user_token(&mut self, token: String) {
        self.user_token = Some(token);
    }

    pub fn user_token(&self) -> Option<&str> {
        self.user_token.as_deref()
    }

    pub fn is_authenticated(&self) -> bool {
        self.user_token.is_some()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::io::{AsyncWriteExt, AsyncReadExt};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_accepted_client_socket_creation() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // 클라이언트: 연결 후 데이터 전송
        let handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream.write_all(b"hello").await.unwrap();
        });

        // 서버: 클라이언트 수신 및 AcceptedClientSocket 생성
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = AcceptedClientSocket::new(stream);

        // ID는 자동 할당되며 1 이상
        assert!(socket.id() >= 1, "ID should be auto-generated and >= 1");

        // 토큰 설정 및 확인
        socket.set_user_token("test_token".to_string());
        assert_eq!(socket.user_token(), Some("test_token"));
        assert!(socket.is_authenticated());

        // 읽기 테스트
        let mut buf = vec![0u8; 10];
        let size = socket.reader.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..size], b"hello");

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_unique_ids_for_multiple_clients() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let mut client_handles = vec![];

        for _ in 0..5 {
            let addr = addr.clone();
            client_handles.push(tokio::spawn(async move {
                TcpStream::connect(addr).await.unwrap()
            }));
        }

        let mut ids = vec![];

        for _ in 0..5 {
            let (stream, _) = listener.accept().await.unwrap();
            let socket = AcceptedClientSocket::new(stream);
            ids.push(socket.id());
        }

        for handle in client_handles {
            let _ = handle.await.unwrap();
        }

        let original_len = ids.len();
        ids.sort_unstable();
        ids.dedup();

        assert_eq!(ids.len(), original_len, "Client IDs should be unique");
    }
}
