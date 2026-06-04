use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/**
    Predefined HTTP responses for the reverse proxy.
**/

/// Sends a 503 Service Unavailable HTTP response to the client stream.
pub async fn send_503(stream: &mut TcpStream) {
    let response = b"HTTP/1.1 503 Service Unavailable\r\n\
                     Content-Type: text/plain\r\n\
                     Content-Length: 19\r\n\
                     Connection: close\r\n\
                     \r\n\
                     Service Unavailable";
    let _ = stream.write_all(response).await;
}
