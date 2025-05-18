use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;
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
    use tokio::sync::mpsc;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_socket_pipeline_single_transaction() {
        println!("[TEST] main thread id: {:?}", thread::current().id());

        let (tx, mut rx) = mpsc::channel::<TcpStream>(1);

        // 서버 리스너 시작
        let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();

        let accept_handle = {
            let tx = tx.clone();
            tokio::spawn(async move {
                println!("[ACCEPT_LOOP] start, thread_id: {:?}", thread::current().id());
                match listener.accept().await {
                    Ok((socket, _)) => {
                        println!("[ACCEPT_LOOP] accepted 1 connection");
                        let _ = tx.send(socket).await;
                    }
                    Err(e) => eprintln!("[ACCEPT_LOOP] error: {}", e),
                }
                println!("[ACCEPT_LOOP] finished");
            })
        };

        // 워커 1개만 필요 (1건만 처리하므로)
        let worker_handle = tokio::spawn(async move {
            println!("[WORKER] start, thread_id: {:?}", thread::current().id());
            if let Some(socket) = rx.recv().await {
                println!("[WORKER] got connection");
                handle_connection(socket).await;
            }
            println!("[WORKER] done");
        });

        // 클라이언트 접속 (std thread)
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(300));
            if let Ok(mut stream) = std::net::TcpStream::connect("127.0.0.1:8081") {
                let msg = b"Hello server!";
                stream.write_all(msg).unwrap();
                let mut buf = [0; 128];
                let n = stream.read(&mut buf).unwrap();
                println!("[CLIENT] received: {}", String::from_utf8_lossy(&buf[..n]));
            } else {
                println!("[CLIENT] connection failed");
            }
        });

        accept_handle.await.unwrap();
        worker_handle.await.unwrap();

        println!("[TEST] done");
    }
}
