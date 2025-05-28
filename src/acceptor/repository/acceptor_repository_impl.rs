use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};

use super::acceptor_repository::AcceptorRepository;

pub struct AcceptorRepositoryImpl;

impl AcceptorRepositoryImpl {
    pub fn new() -> Self {
        AcceptorRepositoryImpl
    }
}

#[async_trait]
impl AcceptorRepository for AcceptorRepositoryImpl {
    async fn accept(&self, listener: &TcpListener) -> Option<TcpStream> {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("[ACCEPTOR_REPO_IMPL] accepted connection");
                Some(stream)
            }
            Err(e) => {
                eprintln!("[ACCEPTOR_REPO_IMPL] accept failed: {}", e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use super::super::acceptor_repository_impl::AcceptorRepositoryImpl;
    use super::super::acceptor_repository::AcceptorRepository;
    use std::sync::Arc;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::task;
    use std::thread;
    use std::time::Duration;
    use tokio::io::AsyncReadExt;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_acceptor_repository_impl_multithread_accepts_connection() {
        // given
        let listener = TcpListener::bind("127.0.0.1:7082").await.unwrap();
        let repository: Arc<dyn AcceptorRepository> = Arc::new(AcceptorRepositoryImpl);

        // spawn client in std thread to simulate external connection
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(300));
            let _ = std::net::TcpStream::connect("127.0.0.1:7082").unwrap();
        });

        // when
        let accepted_stream: Option<TcpStream> = repository.accept(&listener).await;

        // then
        assert!(accepted_stream.is_some());
        println!("[TEST] Accepted stream successfully in multithreaded environment");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_acceptor_repository_impl_handles_multiple_accepts() {
        let listener = TcpListener::bind("127.0.0.1:7083").await.unwrap();
        let repository = Arc::new(AcceptorRepositoryImpl);
        let listener = Arc::new(listener);

        for i in 0..3 {
            let port = 7083;
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(200 + i * 100));
                let _ = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
            });
        }

        let mut handles = vec![];

        for _ in 0..3 {
            let repo = repository.clone();
            let listener = listener.clone(); // ✅ Arc clone

            handles.push(task::spawn(async move {
                let stream = repo.accept(&listener).await;
                assert!(stream.is_some());
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        println!("[TEST] Multiple acceptors handled in parallel");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 16)]
    async fn test_acceptor_repository_impl_handles_100_connections() {
        let port = 8085;
        let address = format!("127.0.0.1:{}", port);
        let listener = Arc::new(TcpListener::bind(&address).await.unwrap());
        let repository = Arc::new(AcceptorRepositoryImpl);

        // 🔁 100개의 클라이언트를 spawn
        for i in 0..100 {
            let addr = address.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(50 + i));
                if let Ok(mut stream) = std::net::TcpStream::connect(&addr) {
                    let _ = stream.write_all(b"Hello").unwrap();
                }
            });
        }

        // 🧵 100개의 서버 태스크 spawn
        let mut handles = vec![];
        for _ in 0..100 {
            let listener = Arc::clone(&listener);
            let repo = Arc::clone(&repository);

            handles.push(task::spawn(async move {
                if let Some(stream) = repo.accept(&listener).await {
                    let mut buf = [0u8; 5];
                    let mut stream = tokio::io::BufReader::new(stream);
                    let _ = stream.read_exact(&mut buf).await;
                    assert_eq!(&buf, b"Hello");
                } else {
                    panic!("Failed to accept connection");
                }
            }));
        }

        // ✅ 모든 핸들 완료 확인
        for handle in handles {
            handle.await.unwrap();
        }

        println!("[TEST] Successfully accepted 100 connections");
    }
}
