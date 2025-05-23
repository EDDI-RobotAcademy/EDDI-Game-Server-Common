use async_trait::async_trait;
use crate::client_socket::entity::accepted_client_socket::AcceptedClientSocket;

#[async_trait]
pub trait TransmitterService: Send + Sync {
    async fn send_welcome_message(&self, socket: AcceptedClientSocket);
}
