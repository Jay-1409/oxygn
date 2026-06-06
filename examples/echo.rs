use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Fixed HTTP/1.1 response — pre-built as bytes so there's zero allocation per request.
const RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\n\
    Content-Type: text/plain\r\n\
    Content-Length: 2\r\n\
    Connection: keep-alive\r\n\
    \r\n\
    OK";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let port = args.get(1).map(|s| s.as_str()).unwrap_or("8080");
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Echo HTTP backend listening on {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];

            // Each iteration handles one HTTP request on a keep-alive connection.
            loop {
                let mut received = 0;

                // Read until we have a complete request header (\r\n\r\n).
                loop {
                    match socket.read(&mut buf[received..]).await {
                        Ok(0) => return, // Client closed connection.
                        Ok(n) => {
                            received += n;
                            // Check for end-of-headers marker.
                            if buf[..received].windows(4).any(|w| w == b"\r\n\r\n") {
                                break;
                            }
                            if received >= buf.len() {
                                break; // Buffer full — respond anyway.
                            }
                        }
                        Err(_) => return,
                    }
                }

                // Send the pre-built HTTP response.
                if socket.write_all(RESPONSE).await.is_err() {
                    return;
                }
            }
        });
    }
}