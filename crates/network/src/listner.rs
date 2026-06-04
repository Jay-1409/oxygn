use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use managers::backend::BackendPool;

/**
    This struct implements the listner functionality, which is to listen for incoming 
    connection on a specified port. 

    It delegates the part of actually forwarding the connection data to threads

    This makes is very efficient, as multiple threads are spawned, and they do their work 

**/  
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
        loop {
            let (mut client_stream, client_addr) = listener.accept().await?;
            let pool = self.pool.clone();
            tokio::spawn(async move {
                let backend_addr = match pool.next_backend() {
                    Some(addr) => addr,
                    None => {
                        eprintln!("Error: No healthy backends available for client {}", client_addr);
                        return; 
                    }
                };
                println!("Routing connection: {} -> {}", client_addr, backend_addr);
                let mut backend_stream = match TcpStream::connect(&backend_addr).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!("Failed to connect to backend {}: {}", backend_addr, e);
                        if let Some((host, port_str)) = backend_addr.split_once(':') {
                            if let Ok(port) = port_str.parse::<u16>() {
                                pool.mark_unhealthy(host, port);
                            }
                        }
                        return;
                    }
                };
                if let Err(e) = io::copy_bidirectional(&mut client_stream, &mut backend_stream).await {
                    eprintln!("Tunnel closed: {}", e);
                }
            });
        }
    }
}
