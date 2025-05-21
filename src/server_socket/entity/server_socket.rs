use tokio::net::TcpListener;

#[derive(Debug)]
pub struct ServerSocket {
    pub listener: TcpListener,
}

impl ServerSocket {
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub fn listener(&self) -> &TcpListener {
        &self.listener
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_server_socket_bind_and_accept() {
        // 임시 바인딩 주소
        let addr = "127.0.0.1:0"; // OS가 사용 가능한 포트 자동 할당
        let listener = TcpListener::bind(addr).await.expect("Failed to bind");
        let local_addr = listener.local_addr().expect("Failed to get local address");

        let server_socket = ServerSocket::new(listener);

        // 클라이언트가 비동기로 연결 시도
        let connect_future = TcpStream::connect(local_addr);

        // 서버는 비동기로 연결을 수락
        let accept_future = server_socket.listener().accept();

        // 동시에 수행
        let (client_result, server_result) = tokio::join!(connect_future, accept_future);

        // 클라이언트 연결 성공
        assert!(client_result.is_ok(), "Client failed to connect");

        // 서버가 클라이언트 연결을 수락
        assert!(server_result.is_ok(), "Server failed to accept connection");

        let (socket, _) = server_result.unwrap();
        assert!(socket.peer_addr().is_ok(), "Accepted socket has no peer address");
    }
}
