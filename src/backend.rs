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
            /*
                A load balancer will have multiple threads reading the backend list constantly to route traffic.
                However, if a server goes down, a thread needs to write to the list to mark it health: false. 
                RwLock allows many threads to read at the exact same time, but safely blocks them if one thread needs to make an update.
            */
            backends: Arc::new(RwLock::new(backends)),
            strategy,
        }
    }
    /**
        Implements the stategy of next backedn choice, according to the startegy used, in runtine, 
        essentially delegates this task to startgy.rs
    **/
    pub fn next_backend(&self) -> Option<String> {
        let backend = self.strategy.next(self)?;
        let url = format!("{}:{}", backend.host, backend.port);
        Some(url)
    }
    /**
        Marks the backend in questionm kicks out of the active backend pool
    **/
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

    /**
        This function spawn a tokio process on a different thread, which will pool through the services,  
        and if the backend is in-active, it will try to ping it and if it responds, it will mark it as healthy, 
        otherwise it will keep it inactive.
        
        in a production environment, i believe that the best way to check the health of a backend is to see if it responds to health check 
        endpoints.  


        Its entire job is to sit in the background, wake up every few seconds (the interval),
        look for any servers that were previously marked dead (health: false),
        and check if they've come back to life. If a server responds, 
        it flips its health status back to true.  ---> This would again be an blocking call --> performance drops here 

        A better implementation would again be a non blocking call, but i have no idea how we can implement that? 

    **/
    pub fn spawn_health_pooler(&self, interval: Duration) {
        let pool_clone = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                let targets: Vec<(usize, String)> = {
                    let backends = pool_clone.backends.read().unwrap();
                    /*
                        Here we are creating a vector of tuples, where each tuple contains the index of the backend in the pool,  
                        and the address of the backend 

                        We enumerate, which returns an iterator of `(index, element)` tuples,  
                        then filter only the ones where health is false     

                        it returns a target -> idx, host:port

                        We collect them into a vector 


                        OPTIMIZATION: 
                            - probably pre allocation the space, -> lesser heap access -> lesser heap access overheads
                    */
                    backends
                        .iter()
                        .enumerate()
                        .filter(|(_, b)| !b.health)
                        .map(|(idx, b)| (idx, format!("{}:{}", b.host, b.port)))
                        .collect()
                };
                /*
                    OPTIMIZATION / STRATEGY / CONFIG: 
                        - For each unhealthy backemd in concurrent spawn the health check tasks 
                                i.e ping them concurently instead of doing it one by one, 
                        - this asks for a better implementation strategy, 
                        - probably put this into an interface, and let the user decide in the configurations what
                                strategy they wish to use
                */
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
