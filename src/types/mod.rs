pub mod config;

use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Backend {
    pub host: String,
    pub port: u16,
    pub health: bool,
    /// Pre-computed SocketAddr — avoids a format!()+parse() allocation on every connection.
    pub addr: SocketAddr,
}
