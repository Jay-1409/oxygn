use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("HTTP Inspector listening on 127.0.0.1:8000");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        
        tokio::spawn(async move {
            let mut buf = [0; 4096]; // Buffer to read request headers
            
            match socket.read(&mut buf).await {
                Ok(0) => return, // Connection closed
                Ok(n) => {
                    // We only read the first chunk of the request.
                    // Now we parse it using httparse:
                    let mut headers = [httparse::EMPTY_HEADER; 64];
                    let mut req = httparse::Request::new(&mut headers);
                    match req.parse(&buf[..n]) {
                        Ok(status) => {
                            if status.is_complete() {
                                let method = req.method.unwrap_or("UNKNOWN");
                                let path = req.path.unwrap_or("/");
                                let host = req.headers
                                    .iter()
                                    .find(|h| h.name.eq_ignore_ascii_case("Host"))
                                    .and_then(|h| std::str::from_utf8(h.value).ok())
                                    .unwrap_or("unknown-host");       
                                
                                println!("--- New HTTP Request ---");
                                println!("Method: {}", method);
                                println!("Path:   {}", path);
                                println!("Host:   {}", host);    
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to parse HTTP headers: {}", e);
                        }
                    }
                    // TODO: 
                    // 1. Call `req.parse(&buf[..n])` to parse the request headers.
                    // 2. The `parse` method returns a Result<httparse::Status<usize>, httparse::Error>.
                    // 3. If it succeeds and is Complete, print:
                    //    - The Request Method (req.method)
                    //    - The Request Path (req.path)
                    //    - The "Host" header (loop through req.headers to find the one named "Host")
                }
                Err(e) => {
                    eprintln!("Failed to read request: {}", e);
                }
            }
        });
    }
}
