// Copyright (c) 2026 Jay 'jay-1409' Shah. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for details.
//
// File: src/network/listner.rs
// Purpose: Handles high-concurrency client accept loops with SO_REUSEPORT.

use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use std::net::SocketAddr;
use crate::managers::pool::BackendPool;

/// TODO: Refactor listner to listener

/*
    This struct implements the listener functionality, which is to listen for incoming 
    connections on a specified port.

    It delegates the part of actually forwarding the connection data to threads.

    This makes it very efficient, as multiple threads are spawned, and they do their work.

    PROS:
        - Fully Supported Protocols (Any TCP-based protocol)
        - HTTP/1.1 and HTTP/2: Standard web traffic.
        - HTTPS: Secure web traffic. Because it is a raw byte tunnel, it forwards the encrypted SSL/TLS handshake directly to the backend. The backend decrypts it.
        - WebSockets: Supported out of the box! Since copy_bidirectional keeps the tunnel open as long as both sides want, WebSocket connections stay open and work perfectly.
        - Databases: You can proxy database connections like Postgres, MySQL, Redis, and MongoDB.
        - gRPC: Modern API protocol running over HTTP/2.
        - SSH / SFTP / FTP: Secure shell and file transfer protocols.
        - Email (SMTP, IMAP): Mail protocols.

    CONS:
        - UDP Traffic:
            - It cannot route protocols that rely on UDP (like DNS queries, WebRTC, or HTTP/3 / QUIC).
        - Layer 7 (Application Layer) Features:
            - Path-based routing, header modification, SSL termination, sticky sessions.

    PERFORMANCE:
        - Uses SO_REUSEPORT: spawns one accept loop per CPU core, each with its own kernel
          socket. The kernel distributes incoming connections across all workers without any
          userspace coordination, eliminating the single-accept-loop bottleneck and matching
          nginx's multi-worker architecture.
*/
pub struct Listener {
    port: u16,
    pool: BackendPool,
    tcp_nodelay: bool,
}

impl Listener {
    pub fn init(port: u16, pool: BackendPool, tcp_nodelay: bool) -> Self {
        Self { port, pool, tcp_nodelay }
    }

    /** - Binds a new TCP socket to `addr` with SO_REUSEPORT so multiple listeners
        -can share the same port and the kernel load-balances connections between them.
            - one might argue, that what if the threads become busy with dealing with 
    **/
    fn bind_reuseport(addr: SocketAddr) -> Result<TcpListener, Box<dyn std::error::Error>> {
        use socket2::{Domain, Protocol, Socket, Type};
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        socket.set_reuse_port(true)?;
        socket.set_reuse_address(true)?;
        socket.set_nonblocking(true)?;
        socket.bind(&addr.into())?;
        socket.listen(4096)?;
        Ok(TcpListener::from_std(socket.into())?)
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("0.0.0.0:{}", self.port).parse()?;
        let num_workers = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        
        println!(
            "Oxygen is listening on {} ({} SO_REUSEPORT workers)",
            addr, num_workers
        );

        let mut set = tokio::task::JoinSet::new();
        for _ in 0..num_workers {
            let listener = Self::bind_reuseport(addr)?;
            let pool = self.pool.clone();
            let tcp_nodelay = self.tcp_nodelay;
            set.spawn(async move {
                loop {
                    let (client_stream, client_addr) = match listener.accept().await {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    tokio::spawn(Self::handle(client_stream, client_addr, pool.clone(), tcp_nodelay));
                }
            });
        }

        // Workers run forever; wait for the first (unexpected) exit.
        set.join_next().await;
        Ok(())
    }

    /*
        Handles a single client connection:
          1. Rate-limit check
          2. Pick a healthy backend (round-robin, zero-alloc SocketAddr)
          3. TCP_NODELAY on both ends to disable Nagle buffering across the proxy hop
          4. Bidirectional byte tunnel via copy_bidirectional
    */
    async fn handle(mut client_stream: TcpStream, client_addr: SocketAddr, pool: BackendPool, tcp_nodelay: bool) {
        let client_ip = client_addr.ip().to_string();
        if !pool.check_rate_limit(&client_ip) {
            eprintln!("Rate limit exceeded for client {}", client_addr);
            crate::network::responses::send_429(&mut client_stream).await;
            return;
        }

        let backend_addr = match pool.next_backend() {
            Some(addr) => addr,
            None => {
                eprintln!("Error: No healthy backends available for client {}", client_addr);
                crate::network::responses::send_503(&mut client_stream).await;
                return;
            }
        };

        // Apply TCP_NODELAY if configured
        if tcp_nodelay {
            let _ = client_stream.set_nodelay(true);
        }

        let mut backend_stream = match TcpStream::connect(backend_addr).await {
            Ok(stream) => stream,
            Err(e) => {
                /*
                    It is possible that a backend went unhealthy after it was allocated.
                    Delegate marking it unhealthy to the backend pool manager.
                */
                eprintln!("Failed to connect to backend {}: {}", backend_addr, e);
                pool.mark_unhealthy_by_addr(backend_addr);
                return;
            }
        };

        // Apply TCP_NODELAY if configured
        if tcp_nodelay {
            let _ = backend_stream.set_nodelay(true);
        }

        /*
            The backend is connected to the client via a duplex link. This allows both
            to communicate as if there is no proxy in between.
        */
        let _ = io::copy_bidirectional(&mut client_stream, &mut backend_stream).await;
        // Tunnel closed (connection reset, client disconnect, etc.) —
        // not logged here to avoid blocking Tokio threads on stderr I/O
        // under high connection churn.
    }
}
