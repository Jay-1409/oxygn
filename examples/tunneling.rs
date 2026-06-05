use tokio::net::{TcpListener, TcpStream};
use tokio::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("Proxy listening on 127.0.0.1:8000. Forwarding to 127.0.0.1:8080");

    loop {
        let (mut client_sock, addr) = listener.accept().await?;
        println!("New client connected: {}", addr);

        tokio::spawn(async move {
            // TODO:
            // 1. Establish a TCP connection to the backend server (127.0.0.1:8080)
            // 2. Use `io::copy_bidirectional` to link client_socket and backend_socket
            // 3. Handle any errors if the copy fails.
            let mut backend_sock = match TcpStream::connect("127.0.0.1:8080").await {
                Ok(stream) => stream, 
                Err(e) => {
                    eprintln!("Failed to connect to backend: {}", e);
                    return;
                }
            };
            let _ = io::copy_bidirectional(&mut client_sock, &mut backend_sock).await;
        });
    }
}
