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


/**
    A backend pool is an implementation of, structuring backends into a structure, capable of handing 
    the worker threads requirnments for load balancing. 
**/
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
            /**
                A load balancer will have multiple threads reading the backend list constantly to route traffic.
                However, if a server goes down, a thread needs to write to the list to mark it health: false. 
                RwLock allows many threads to read at the exact same time, but safely blocks them if one thread needs to make an update.
            **/
            backends: Arc::new(RwLock::new(backends)),
            strategy,
        }
    }
    
}
