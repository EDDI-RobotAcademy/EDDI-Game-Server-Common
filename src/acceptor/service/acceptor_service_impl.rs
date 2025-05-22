use tokio::net::TcpListener;
use std::sync::Arc;
use async_trait::async_trait;
use crate::acceptor::repository::acceptor_repository::AcceptorRepository;
use crate::acceptor::repository::acceptor_repository_impl::AcceptorRepositoryImpl;
use crate::acceptor::service::acceptor_service::AcceptorService;
use crate::server_socket::repository::server_socket_repository::ServerSocketRepository;
use crate::server_socket::repository::server_socket_repository_impl::ServerSocketRepositoryImpl;
use crate::tokio_thread::repository::tokio_thread_repository::TokioThreadRepository;
use crate::tokio_thread::repository::tokio_thread_repository_impl::TokioThreadRepositoryImpl;
// pub struct AcceptorServiceImpl {
//     repo: Arc<dyn AcceptorRepository>,
// }
//
// impl AcceptorServiceImpl {
//     pub fn new(repo: Arc<dyn AcceptorRepository>) -> Self {
//         Self { repo }
//     }
// }
//
// #[async_trait::async_trait]
// impl AcceptorService for AcceptorServiceImpl {
//     async fn run_accept_loop(&self, listener: TcpListener) {
//         loop {
//             if let Some(socket) = self.repo.accept(&listener).await {
//                 println!("[ACCEPTOR_SERVICE_IMPL] accepted connection");
//                 tokio::spawn(async move {
//                     println!("[CONNECTION_HANDLER] socket = {:?}", socket);
//                 });
//             }
//         }
//     }
// }

pub struct AcceptorServiceImpl {
    thread_repo: Arc<dyn TokioThreadRepository>,
    socket_repo: Arc<dyn ServerSocketRepository>,
    acceptor_repo: Arc<dyn AcceptorRepository>,
}

impl AcceptorServiceImpl {
    pub fn new() -> Self {
        let socket_repo = Arc::new(ServerSocketRepositoryImpl::new());
        let acceptor_repo = Arc::new(AcceptorRepositoryImpl::new());
        let thread_repo = Arc::new(TokioThreadRepositoryImpl::new());

        Self {
            socket_repo,
            acceptor_repo,
            thread_repo,
        }
    }
}

#[async_trait]
impl AcceptorService for AcceptorServiceImpl {
    async fn run(self: Arc<Self>) {  // self를 Arc<Self>로 받음
        if let Some(server_socket) = self.socket_repo.bind("127.0.0.1:8080").await {
            let listener = Arc::new(server_socket.into_listener());

            println!("🚀 [SERVICE] Bind success, starting accept loop...");

            let this = self.clone();  // clone Arc<Self>

            this.thread_repo.spawn(0, Box::pin({
                let this = this.clone();  // async move 내부에서 새로 clone
                let listener = listener.clone();  // listener도 Arc이니 clone

                async move {
                    loop {
                        if let Some(socket) = this.acceptor_repo.accept(&listener).await {
                            println!("🔌 [SERVICE] Connection accepted");

                            tokio::spawn(async move {
                                println!("🎯 [SERVICE] Handling connection {:?}", socket);
                            });
                        }
                    }
                }
            })).await;
        } else {
            eprintln!("❌ [SERVICE] Failed to bind to socket.");
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
        // 서비스 생성 및 Arc wrapping
        let service = Arc::new(AcceptorServiceImpl::new());

        // 서버 실행 (백그라운드 task)
        let service_clone = service.clone();
        tokio::spawn(async move {
            service_clone.run().await;
        });

        // 서버가 바인딩되어 리스닝 준비할 시간을 잠시 줌
        sleep(Duration::from_millis(100)).await;

        // 100명의 클라이언트를 동시에 접속 시도
        let mut handles = Vec::new();
        for _ in 0..100 {
            let handle = tokio::spawn(async {
                match TcpStream::connect("127.0.0.1:8080").await {
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
