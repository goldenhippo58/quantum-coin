use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn start_node(port: u16) {
    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("Failed to bind port");
    println!("Node running on port {}", port);

    loop {
        let (mut socket, _) = listener
            .accept()
            .await
            .expect("Failed to accept connection");
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            let n = socket.read(&mut buffer).await.expect("Failed to read data");
            println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

            socket
                .write_all(b"Message received!")
                .await
                .expect("Failed to write data");
        });
    }
}
