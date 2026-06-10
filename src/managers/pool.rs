use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::net::SocketAddr;
use crate::strategies::limiting::{LimitingStrategy, LimitingStrategyFactory};
use crate::strategies::routing::{self, RoutingStrategy};
use crate::types::Backend;
use crate::types::config::Config;

/*
    A backend pool is an implementation of, structuring backends into a structure, capable of handing
    the worker threads requirnments for load balancing.

*/
#[derive(Clone)]
pub struct BackendPool {
    pub(crate) backends: Arc<RwLock<Vec<Backend>>>,
    strategy: Arc<dyn RoutingStrategy>,
    rate_limiter: Arc<dyn LimitingStrategy>,
    limit_rate: u32,
}

impl BackendPool {
    pub fn init(config: &Config) -> Self {
        let mut backends = Vec::new();
        for b_config in &config.backend {
            let host = &b_config.backend_host;
            for port in &b_config.ports {
                // Pre-compute the SocketAddr once at startup so the hot path
                // never needs format!() or string parsing per connection.
                let addr: SocketAddr = format!("{}:{}", host, port)
                    .parse()
                    .expect("Invalid backend address in config");
                backends.push(Backend {
                    host: host.to_string(),
                    port: *port,
                    health: true,
                    addr,
                });
            }
        }
        let strategy_type = &config.load_balancing.strategy;
        let strategy = routing::init(strategy_type);
        // Initialize the rate limiter with the configured window duration and memory budget
        let rate_limiter = LimitingStrategyFactory::init(
            &config.limiting.strategy,
            Duration::from_secs(config.limiting.window_secs),
            config.limiting.memory_budget_mb,
        );
        let limit_rate = config.limiting.rate;
        Self {
            /*
                A load balancer will have multiple threads reading the backend list constantly to route traffic.
                However, if a server goes down, a thread needs to write to the list to mark it health: false.
                RwLock allows many threads to read at the exact same time, but safely blocks them if one thread needs to make an update.
            */
            backends: Arc::new(RwLock::new(backends)),
            strategy,
            rate_limiter,
            limit_rate,
        }
    }

    /*
        Implements the stategy of next backedn choice, according to the startegy used, in runtine,
        essentially delegates this task to startgy.rs
    */
    /// Returns the pre-computed SocketAddr of the next healthy backend.
    /// Zero allocations on the hot path.
    pub fn next_backend(&self) -> Option<SocketAddr> {
        let backends = self.backends.read().unwrap();
        self.strategy.next(&backends).map(|b| b.addr)
    }

    /*
        Marks the backend in questionm kicks out of the active backend pool
    */
    pub fn mark_unhealthy(&self, host: &str, port: u16) {
        let mut backends = self.backends.write().unwrap();
        /*
            We iterate over the list of backends that we have, and mark the backend in Question as inactive,
            not the most effective one, but how many backends can be running ona nignix server,
            i think ideally cannot be more than a 1000 or 10000

            although the incomming traffic will be blocked for that much time,
            should have a buffer mechanism for that.

            FUTURE SCOPE:
                - impelement this method in a way that is non blocking
                - While it becomes non blocking it should not end up being deadlock friendly.
        */
        for backend in backends.iter_mut() {
            if backend.host.eq(host) && backend.port == port {
                backend.health = false;
                break;
            }
        }
    }

    /*
        Marks the backend unhealthy by its SocketAddr.
    */
    pub fn mark_unhealthy_by_addr(&self, addr: std::net::SocketAddr) {
        self.mark_unhealthy(&addr.ip().to_string(), addr.port());
    }

    /*
        This function spawns a background task that periodically checks the health of inactive backends.
        It runs checks concurrently using `tokio::task::JoinSet` and throttles concurrent connection attempts
        to a maximum of 100 using a `tokio::sync::Semaphore`. It supports both raw TCP ping checks and HTTP GET checks.

        TODO: They method that is used for checking the health based on different type of selection of type, can be moved
        into a strategy, and seperated from this.
            - Makes it easier to update
            - seperation of concern
            - Helps us get closer to Single responsibilty principlesgi

    */
    pub fn spawn_health_pooler(&self, config: &crate::types::config::HealthCheck) {
        let pool_clone = self.clone();
        let interval = Duration::from_secs(config.interval_secs);
        let check_type = config.check_type.clone();
        let path = config.path.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            let permit_bucket = Arc::new(tokio::sync::Semaphore::new(100));

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

                if targets.is_empty() {
                    continue;
                }

                let mut set = tokio::task::JoinSet::new();

                for (idx, addr) in targets {
                    let permit_bucket = permit_bucket.clone();
                    let check_type = check_type.clone();
                    let path = path.clone();

                    set.spawn(async move {
                        // Acquire a permit before making the network call.
                        // If 100 checks are already running, this will pause here until one finishes.
                        let _permit = permit_bucket.acquire().await.unwrap();
                        let is_healthy = match check_type.as_str() {
                            "http" => {
                                match tokio::time::timeout(Duration::from_secs(1), async {
                                    let mut stream = tokio::net::TcpStream::connect(&addr).await?;
                                    let request = format!(
                                        "GET {} HTTP/1.1\r\n\
                                             Host: {}\r\n\
                                             Connection: close\r\n\r\n",
                                        path, addr
                                    );
                                    tokio::io::AsyncWriteExt::write_all(
                                        &mut stream,
                                        request.as_bytes(),
                                    )
                                    .await?;
                                    let mut response_buf = [0u8; 1024];
                                    let n = tokio::io::AsyncReadExt::read(
                                        &mut stream,
                                        &mut response_buf,
                                    )
                                    .await?;
                                    let response_str = String::from_utf8_lossy(&response_buf[..n]);
                                    if let Some(status_line) = response_str.lines().next() {
                                        let parts: Vec<&str> =
                                            status_line.split_whitespace().collect();
                                        if parts.len() >= 2
                                            && (parts[0].starts_with("HTTP/1.1")
                                                || parts[0].starts_with("HTTP/1.0"))
                                        {
                                            if let Ok(status_code) = parts[1].parse::<u16>() {
                                                Ok::<bool, std::io::Error>(
                                                    status_code >= 200 && status_code < 400,
                                                )
                                            } else {
                                                Ok::<bool, std::io::Error>(false)
                                            }
                                        } else {
                                            Ok::<bool, std::io::Error>(false)
                                        }
                                    } else {
                                        Ok::<bool, std::io::Error>(false)
                                    }
                                })
                                .await
                                {
                                    Ok(Ok(healthy)) => healthy,
                                    _ => false,
                                }
                            }
                            _ => {
                                // Default "tcp" check
                                matches!(
                                    tokio::time::timeout(
                                        Duration::from_secs(1),
                                        tokio::net::TcpStream::connect(&addr)
                                    )
                                    .await,
                                    Ok(Ok(_))
                                )
                            }
                        };

                        // The permit is automatically returned to the bucket when `_permit` goes out of scope.
                        (idx, addr, is_healthy)
                    });
                }

                // Process the results as they finish
                while let Some(res) = set.join_next().await {
                    if let Ok((idx, addr, is_healthy)) = res {
                        if is_healthy {
                            let mut backends = pool_clone.backends.write().unwrap();
                            if idx < backends.len() {
                                if !backends[idx].health {
                                    backends[idx].health = true;
                                    println!("Backend {} is back online", addr);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /*
        Checks if a client's IP is allowed by the rate limiter.
    */
    pub fn check_rate_limit(&self, ip: &str) -> bool {
        self.rate_limiter.check_limit(ip, self.limit_rate)
    }
}
