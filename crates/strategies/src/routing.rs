use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use types::Backend;

pub trait RoutingStrategy: Send + Sync {
    fn next(&self, backends: &[Backend]) -> Option<Backend>;
}

pub fn init(name: &str) -> Arc<dyn RoutingStrategy> {
    match name {
        "round_robin" => Arc::new(RoundRobin::new()),
        _ => Arc::new(RoundRobin::new()),
    }
}

pub struct RoundRobin {
    counter: AtomicUsize,
}

impl RoundRobin {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
}

impl RoutingStrategy for RoundRobin {
    fn next(&self, backends: &[Backend]) -> Option<Backend> {
        let len = backends.len();
        if len == 0 {
            return None;
        }
        let start_idx = self.counter.fetch_add(1, Ordering::SeqCst);
        for i in 0..len {
            let idx = (start_idx + i) % len;
            if backends[idx].health {
                return Some(backends[idx].clone());
            }
        }
        None
    }
}
