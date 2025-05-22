mod acceptor;
mod client_socket;
mod server_socket;
mod tokio_thread;
mod transmitter;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

pub async fn receive(socket: &mut TcpStream) -> Option<Vec<u8>> {
    let mut buf = vec![0; 1024];
    match socket.read(&mut buf).await {
        Ok(0) => None,
        Ok(n) => {
            println!(
                "[RECEIVE] Received: {:?}, thread_id: {:?}",
                &buf[..n],
                thread::current().id()
            );
            Some(buf[..n].to_vec())
        }
        Err(e) => {
            eprintln!("[RECEIVE] Error: {}", e);
            None
        }
    }
}

pub async fn transmit(socket: &mut TcpStream, _request: Vec<u8>) {
    let response = b"Hello from server";
    if let Err(e) = socket.write_all(response).await {
        eprintln!("[TRANSMIT] Error sending response: {}", e);
    } else {
        println!("[TRANSMIT] Response sent, thread_id: {:?}", thread::current().id());
    }
}

async fn handle_connection(mut socket: TcpStream) {
    println!("[HANDLE_CONNECTION] thread_id: {:?}", std::thread::current().id());

    if let Some(request) = receive(&mut socket).await {
        transmit(&mut socket, request).await;
    }

    if let Err(e) = socket.shutdown().await {
        eprintln!("[HANDLE_CONNECTION] shutdown error: {}", e);
    } else {
        println!("[HANDLE_CONNECTION] socket shutdown success");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use std::net::TcpStream as StdTcpStream;
    use std::io::{Read, Write};
    use std::thread;
    use std::time::Duration;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_multiple_connections_concurrently() {
        println!("[TEST] main thread id: {:?}", thread::current().id());

        let listener = TcpListener::bind("127.0.0.1:7082").await.unwrap();

        // 서버 accept 루프는 tokio task로 동작
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((socket, _)) => {
                        println!("[ACCEPT] new connection");

                        tokio::spawn(async move {
                            handle_connection(socket).await;
                        });
                    }
                    Err(e) => {
                        eprintln!("[ACCEPT ERROR] {}", e);
                    }
                }
            }
        });

        // 클라이언트 수
        let client_count = 5;
        let mut handles = vec![];

        // 클라이언트들 std::thread로 동시에 요청
        for i in 0..client_count {
            let handle = thread::spawn(move || {
                thread::sleep(Duration::from_millis(200)); // 연결 시간 간격 조절
                if let Ok(mut stream) = StdTcpStream::connect("127.0.0.1:7082") {
                    let msg = format!("Hello from client {}", i);
                    stream.write_all(msg.as_bytes()).unwrap();

                    let mut buf = [0u8; 128];
                    let n = stream.read(&mut buf).unwrap();
                    let response = String::from_utf8_lossy(&buf[..n]);
                    println!("[CLIENT {}] received: {}", i, response);
                    assert_eq!(response, "Hello from server");
                } else {
                    panic!("[CLIENT {}] connection failed", i);
                }
            });
            handles.push(handle);
        }

        // 모든 클라이언트 작업 완료 대기
        for handle in handles {
            handle.join().expect("Client thread failed");
        }

        println!("[TEST] all clients done");
    }
}