use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::TcpStream;

static CLIENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1); // 시작값 1

#[derive(Debug)]
pub struct AcceptedClientSocket {
    pub id: usize,
    pub stream: TcpStream,
}

impl AcceptedClientSocket {
    pub fn new(stream: TcpStream) -> Self {
        let id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self { id, stream }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio::io::{AsyncWriteExt, AsyncReadExt};
    use std::time::Duration;

    #[tokio::test]
    async fn test_accepted_client_socket_creation() {
        // 로컬 테스트용 listener 바인딩
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // 클라이언트 시뮬레이션
        let handle = tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream.write_all(b"hello").await.unwrap();
        });

        // 서버 accept
        let (stream, _) = listener.accept().await.unwrap();
        let socket = AcceptedClientSocket::new(stream);

        // 클라이언트 식별자와 스트림 테스트
        assert!(socket.id() >= 1);
        assert!(socket.stream().peer_addr().is_ok());

        // 읽기 테스트 (선택)
        let mut buf = vec![0u8; 10];
        let mut s = socket.stream;
        let size = s.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..size], b"hello");

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_unique_ids_for_multiple_clients() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let mut handles = vec![];

        for _ in 0..5 {
            let addr = addr.clone();
            handles.push(tokio::spawn(async move {
                TcpStream::connect(addr).await.unwrap()
            }));
        }

        let mut ids = vec![];

        for _ in 0..5 {
            let (stream, _) = listener.accept().await.unwrap();
            let socket = AcceptedClientSocket::new(stream);
            ids.push(socket.id());
        }

        for h in handles {
            h.await.unwrap();
        }

        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 5, "All client IDs must be unique");
    }
}
