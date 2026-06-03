use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::net::TcpStream;
use std::sync::RwLock;
use tokio::time::sleep;
use crate::strategy::*;
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Backend {
    pub host: String,
    pub port: u16, 
    pub health: bool,
}


#[derive(Clone)]
pub struct BackendPool {
    pub(crate) backends: Arc<RwLock<Vec<Backend>>>,
    strategy: Arc<dyn RoutingStrategy>,
}

impl BackendPool {
    pub fn init(config: &Config) -> Self {
        let mut backends = Vec::new();
        for b_config in &config.backend {
            let host = &b_config.backend_host;
            for port in &b_config.ports {
                backends.push(Backend {
                    host: host.to_string(),
                    port: *port,
                    health: true,
                });
            }
        }
        let strategy_type = &config.load_balancing.strategy;
        let strategy = crate::strategy::init(strategy_type);
        Self {
            backends: Arc::new(RwLock::new(backends)),
            strategy,
        }
    }
    
}
