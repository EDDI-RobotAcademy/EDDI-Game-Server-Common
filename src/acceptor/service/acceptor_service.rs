use tokio::net::TcpListener;
use async_trait::async_trait;

#[async_trait]
pub trait AcceptorService: Send + Sync {
    async fn run_accept_loop(&self, listener: TcpListener);
}
