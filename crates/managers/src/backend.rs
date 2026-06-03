use std::sync::{Arc, RwLock};
use std::time::Duration;
use types::Backend;
use types::config::Config;
use strategies::routing::{self, RoutingStrategy};

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
        let strategy = routing::init(strategy_type);
        Self {
            backends: Arc::new(RwLock::new(backends)),
            strategy,
        }
    }

    pub fn next_backend(&self) -> Option<String> {
        let backends = self.backends.read().unwrap();
        if let Some(backend) = self.strategy.next(&backends) {
            Some(format!("{}:{}", backend.host, backend.port))
        } else {
            None
        }
    }

    pub fn mark_unhealthy(&self, host: &str, port: u16) {
        let mut backends = self.backends.write().unwrap();
        for backend in backends.iter_mut() {
            if backend.host.eq(host) && backend.port == port {
                backend.health = false;
                break;
            }
        }
    }

    pub fn spawn_health_pooler(&self, interval: Duration) {
        let pool_clone = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                let targets: Vec<(usize, String)> = {
                    let backends = pool_clone.backends.read().unwrap();
                    backends
                        .iter()
                        .enumerate()
                        .filter(|(_, b)| !b.health)
                        .map(|(idx, b)| (idx, format!("{}:{}", b.host, b.port)))
                        .collect()
                };

                for (idx, addr) in targets {
                    let is_healthy = tokio::time::timeout(
                        Duration::from_secs(1),
                        tokio::net::TcpStream::connect(&addr)
                    ).await.is_ok();
                    if is_healthy {
                        let mut backends = pool_clone.backends.write().unwrap();
                        if idx < backends.len() {
                            backends[idx].health = true;
                            println!("Backend {} is back online", addr);
                        }
                    }
                }
            }
        });
    }
}
