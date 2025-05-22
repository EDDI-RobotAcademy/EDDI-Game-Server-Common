use async_trait::async_trait;
use crate::transmitter::entity::transmit_data::TransmitData;

#[async_trait]
pub trait TransmitterRepository: Send + Sync {
    async fn send(&self, target_addr: String, data: TransmitData);
}
