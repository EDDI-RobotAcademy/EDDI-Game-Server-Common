use tokio::net::TcpListener;
use std::sync::Arc;
use crate::acceptor::repository::acceptor_repository::AcceptorRepository;
use crate::acceptor::service::acceptor_service::AcceptorService;

pub struct AcceptorServiceImpl {
    repo: Arc<dyn AcceptorRepository>,
}

impl AcceptorServiceImpl {
    pub fn new(repo: Arc<dyn AcceptorRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl AcceptorService for AcceptorServiceImpl {
    async fn run_accept_loop(&self, listener: TcpListener) {
        loop {
            if let Some(socket) = self.repo.accept(&listener).await {
                println!("[ACCEPTOR_SERVICE_IMPL] accepted connection");
                tokio::spawn(async move {
                    println!("[CONNECTION_HANDLER] socket = {:?}", socket);
                });
            }
        }
    }
}
