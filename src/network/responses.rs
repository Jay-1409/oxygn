// Copyright (c) 2026 Jay 'jay-1409' Shah. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for details.
//
// File: src/network/responses.rs
// Purpose: Helper functions for sending raw HTTP responses (e.g., 429 Too Many Requests, 503 Service Unavailable).

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

/// Sends a 429 Too Many Requests HTTP response to the client stream.
pub async fn send_429(stream: &mut TcpStream) {
    let response = b"HTTP/1.1 429 Too Many Requests\r\n\
                     Content-Type: text/plain\r\n\
                     Content-Length: 17\r\n\
                     Connection: close\r\n\
                     \r\n\
                     Too Many Requests";
    let _ = stream.write_all(response).await;
}
