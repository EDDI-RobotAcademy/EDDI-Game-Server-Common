use std::sync::Arc;
use async_trait::async_trait;

use crate::utility::env_detector::EnvDetector;

use crate::acceptor::repository::acceptor_repository::AcceptorRepository;
use crate::acceptor::repository::acceptor_repository_impl::AcceptorRepositoryImpl;
use crate::acceptor::service::acceptor_service::AcceptorService;
use crate::client_socket::entity::accepted_client_socket::AcceptedClientSocket;
use crate::client_socket::repository::client_socket_repository::ClientSocketRepository;
use crate::client_socket::repository::client_socket_repository_impl::ClientSocketRepositoryImpl;
use crate::server_socket::repository::server_socket_repository::ServerSocketRepository;
use crate::server_socket::repository::server_socket_repository_impl::ServerSocketRepositoryImpl;
use crate::tokio_thread::repository::tokio_thread_repository::TokioThreadRepository;
use crate::tokio_thread::repository::tokio_thread_repository_impl::TokioThreadRepositoryImpl;

pub struct AcceptorServiceImpl {
    thread_repo: Arc<dyn TokioThreadRepository>,
    socket_repo: Arc<dyn ServerSocketRepository>,
    acceptor_repo: Arc<dyn AcceptorRepository>,
    client_socket_repo: Arc<dyn ClientSocketRepository>,
}

impl AcceptorServiceImpl {
    pub fn new() -> Self {
        let socket_repo = Arc::new(ServerSocketRepositoryImpl::new());
        let acceptor_repo = Arc::new(AcceptorRepositoryImpl::new());
        let thread_repo = Arc::new(TokioThreadRepositoryImpl::new());
        let client_socket_repo = Arc::new(ClientSocketRepositoryImpl::new());

        Self {
            socket_repo,
            acceptor_repo,
            thread_repo,
            client_socket_repo
        }
    }
}


// Env 추가

#[async_trait]
impl AcceptorService for AcceptorServiceImpl {
    async fn run(self: Arc<Self>) {
        // Env에서 bind address 가져오기
        let bind_address = match EnvDetector::get_bind_address() {
            Some(addr) => addr,
            None => {
                eprintln!("❌ [SERVICE] Failed to get bind address from Env.");
                return;
            }
        };

        if let Some(server_socket) = self.socket_repo.bind(&bind_address).await {
            let listener = Arc::new(server_socket.into_listener());

            println!("🚀 [SERVICE] Bind success at {}, starting accept loop...", bind_address);

            let this = self.clone();
            let client_socket_repo = self.client_socket_repo.clone();

            this.thread_repo.spawn(0, Box::pin({
                let this = this.clone();
                let listener = listener.clone();

                async move {
                    loop {
                        if let Some(socket) = this.acceptor_repo.accept(&listener).await {
                            println!("🔌 [SERVICE] Connection accepted");

                            let client_socket_repo = client_socket_repo.clone();

                            tokio::spawn(async move {
                                println!("🎯 [SERVICE] Handling connection {:?}", socket);
                                let client_socket = AcceptedClientSocket::new(socket);
                                client_socket_repo.register(client_socket).await;
                            });
                        }
                    }
                }
            })).await;
        } else {
            eprintln!("❌ [SERVICE] Failed to bind to socket: {}", bind_address);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    use tokio::net::TcpStream;
    use std::sync::Arc;

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn test_concurrent_accepts_from_100_clients() {
        let service = Arc::new(AcceptorServiceImpl::new());

        // 서버 실행 (백그라운드 task)
        let service_clone = service.clone();
        tokio::spawn(async move {
            service_clone.run().await;
        });

        // 서버가 바인딩되어 리스닝 준비할 시간을 잠시 줌
        sleep(Duration::from_millis(100)).await;

        // Env에서 bind address 가져오기
        let bind_address = EnvDetector::get_bind_address().expect("Env bind address must be set");

        // bind_address가 0.0.0.0이면 클라이언트용으로 127.0.0.1로 변경
        let connect_address = if bind_address.starts_with("0.0.0.0") {
            bind_address.replacen("0.0.0.0", "127.0.0.1", 1)
        } else {
            bind_address.clone()
        };

        // 100명의 클라이언트를 동시에 접속 시도
        let mut handles = Vec::new();
        for _ in 0..100 {
            let addr = connect_address.clone();
            let handle = tokio::spawn(async move {
                match TcpStream::connect(&addr).await {
                    Ok(_) => println!("✅ Client connected"),
                    Err(e) => eprintln!("❌ Failed to connect: {}", e),
                }
            });
            handles.push(handle);
        }

        // 모든 클라이언트 접속 완료 대기
        for handle in handles {
            handle.await.unwrap();
        }

        // 서버가 접속 로그 출력할 시간을 약간 기다림
        sleep(Duration::from_millis(500)).await;
    }
}
