use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use managers::pool::BackendPool;

/// TODO: Refactor listner to listener

/*
    This struct implements the listner functionality, which is to listen for incoming 
    connection on a specified port. 

    It delegates the part of actually forwarding the connection data to threads

    This makes is very efficient, as multiple threads are spawned, and they do their work 

    PROS:

        - Fully Supported Protocols (Any TCP-based protocol)
        - HTTP/1.1 and HTTP/2: Standard web traffic.
        - HTTPS: Secure web traffic. Because it is a raw byte tunnel, it forwards the encrypted SSL/TLS handshake directly to the backend. The backend decrypts it.
        - WebSockets: Supported out of the box! Since copy_bidirectional keeps the tunnel open as long as both sides want, WebSocket connections stay open and work perfectly.
        - Databases: You can proxy database connections like Postgres, MySQL, Redis, and MongoDB.
        - gRPC: Modern API protocol running over HTTP/2.
        - SSH (Secure Shell (i epnder who wpuld use a rps for this ?? lol)) / SFTP (Secure File Transfer Protocol) / FTP (File Transfer Protocol): Secure shell and file transfer protocols.
        - Email (SMTP, IMAP): Mail protocols.

    CONS:
        -  UDP Traffic:
            - It cannot route protocols that rely on UDP (like DNS queries, WebRTC, or HTTP/3 / QUIC), because our listener is strictly a TcpListener.
        - Layer 7 (Application Layer) Features:
            - Path-based routing: You cannot route /api to one backend and /static to another, because the proxy never reads the HTTP request headers.
            - Header Modification: You cannot inject headers like X-Forwarded-For to let the backend know the client's original IP address.
            - SSL Termination: The proxy cannot decrypt the SSL certificates. Decryption must happen at the backend.
            - Sticky Sessions / Cookie-based routing: You cannot route a user back to the same backend based on their session cookie.

*/  
pub struct Listener {
    port: u16,
    pool: BackendPool,
}

impl Listener {
    pub fn init(port: u16, pool: BackendPool) -> Self {
        Self { port, pool }
    }
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Oxygen is listening on {}", addr);
        /* 
            Exactly here we delegate any incomming connections to a seperate thread, using tokio spawn, 
            which runs the process in a seperate thread. 


        */  
        loop {
            let (mut client_stream, client_addr) = listener.accept().await?;
            let pool = self.pool.clone();
            tokio::spawn(async move {
                let client_ip = client_addr.ip().to_string();
                if !pool.check_rate_limit(&client_ip) {
                    eprintln!("Rate limit exceeded for client {}", client_addr);
                    crate::responses::send_429(&mut client_stream).await;
                    return;
                }

                let backend_addr = match pool.next_backend() {
                    Some(addr) => addr,
                    None => {
                        eprintln!("Error: No healthy backends available for client {}", client_addr);
                        crate::responses::send_503(&mut client_stream).await;
                        return; 
                    }
                };
                println!("Routing connection: {} -> {}", client_addr, backend_addr);
                let mut backend_stream = match TcpStream::connect(&backend_addr).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        /* 
                            it is possible that a backend went unhealthy after it was allocated, in that case, it is to be marked unhealthy. 
                            we delegate it to the backend pool manager. 
                        */
                        eprintln!("Failed to connect to backend {}: {}", backend_addr, e);
                        pool.mark_unhealthy_by_addr(&backend_addr);
                        return;
                    }
                };
                /*
                    The backend is connected to the client, via a duplex link, this allows both to communicate between each other as if there is no proxy in between 
                */
                if let Err(e) = io::copy_bidirectional(&mut client_stream, &mut backend_stream).await {
                    eprintln!("Tunnel closed: {}", e);
                }
            });
        }
    }
}

