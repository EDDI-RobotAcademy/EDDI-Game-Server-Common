use std::net::SocketAddr;
use tokio::net::UdpSocket;
use async_trait::async_trait;
use crate::transmitter::entity::transmit_data::TransmitData;
use crate::transmitter::repository::transmitter_repository::TransmitterRepository;

pub struct TransmitterRepositoryImpl {
    socket: UdpSocket,
}

impl TransmitterRepositoryImpl {
    pub async fn new(bind_addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        Ok(Self { socket })
    }
}

#[async_trait]
impl TransmitterRepository for TransmitterRepositoryImpl {
    async fn send(&self, target_addr: String, data: TransmitData) {
        match target_addr.parse::<SocketAddr>() {
            Ok(addr) => {
                let _ = self.socket.send_to(data.transmit_content(), &addr).await;
                println!("📤 [Transmitter] Sent 1024 bytes to {}", addr);
            }
            Err(e) => {
                eprintln!("❌ [Transmitter] Invalid address {}: {:?}", target_addr, e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UdpSocket;
    use tokio::time::{timeout, Duration};
    use crate::transmitter::entity::transmit_data::TransmitData;

    #[tokio::test]
    async fn test_send_transmit_data() {
        // 수신용 소켓 바인딩
        let receiver_socket = UdpSocket::bind("127.0.0.1:6001").await.unwrap();
        let transmitter = TransmitterRepositoryImpl::new("127.0.0.1:6000")
            .await
            .expect("Failed to bind transmitter socket");

        // 송신할 데이터 설정
        let mut data = TransmitData::new();
        let payload = b"Test UDP transmission!";
        data.transmit_content_mut()[..payload.len()].copy_from_slice(payload);

        // 송신 수행
        transmitter.send("127.0.0.1:6001".to_string(), data.clone()).await;

        // 수신 대기
        let mut buf = [0u8; 1024];
        let recv_len = timeout(Duration::from_secs(1), receiver_socket.recv_from(&mut buf))
            .await
            .expect("Did not receive data in time")
            .expect("Failed to receive data")
            .0;

        // 비교
        assert_eq!(&buf[..recv_len], &data.transmit_content()[..recv_len]);
        println!("✅ Received data: {:?}", &buf[..recv_len]);
    }
}
