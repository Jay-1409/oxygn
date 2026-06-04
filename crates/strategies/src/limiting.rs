use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use mini_moka::sync::Cache;

/**
    Oxygen should support multiple rate limiting strategies
        - some basic strategies are 
            - ~Basic Counting strategy~
            - Token bucket strategy
            - leaky bucket strategy
            - fixed window counter strategy
            - sliding window log
            - sliding window counter

        - better? ig in terms of implementation complexity atleast
            - mini-moka based
            - redis based  ?
                - it turns out that anything over tcp at this level will result in 
                    some latency which is not acceptable for this use cases
                    thus we rule out redis/ mini-redis based strategies


        TODO:
            - implement other basic strategies 
            - suggest some better way to do it? 
                better than the mini-moka implementation

**/

pub trait LimitingStrategy: Send + Sync { 
    fn check_limit(&self, ip: &str, limit_rate: u32) -> bool;
}


/**
 * Bypass strategy for when no rate limiting strategy has been choosen. 
**/
pub struct NoLimiting;

impl NoLimiting {
    pub fn init() -> Self {
        Self {}
    }
}

impl LimitingStrategy for NoLimiting {
    fn check_limit(&self, _ip: &str, _limit_rate: u32) -> bool {
        true
    }
}

/**
    I seriously dont know why i added this implementation,
    but as it stands, we have it, its not efficient, as it blocks the entire thread
    for every request, haha
**/        
pub struct BasicCounter {
    records: Mutex<HashMap<String, (u32, Instant)>>,
    window_duration: Duration,
}

impl BasicCounter {
    pub fn init(window_duration: Duration) -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
            window_duration,
        }
    }
}   

impl LimitingStrategy for BasicCounter {
    fn check_limit(&self, ip: &str, limit_rate: u32) -> bool {
        let mut records = self.records.lock().unwrap();
        let now = Instant::now();        
        let (count, window_start) = records.entry(ip.to_string())
            .or_insert((0, now));
        if now.duration_since(*window_start) >= self.window_duration {
            *count = 1;
            *window_start = now;
            return true;
        }
        if *count < limit_rate {
            *count += 1;
            true 
        } else {
            false 
        }
    }
}

/**
    
**/
pub struct MokaCounter {
    records: Cache<String, u32>,
    window_duration: Duration,
}

impl MokaCounter {
    pub fn new(window_duration: Duration) -> Self {
        Self {
            records: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(window_duration * 2) 
                .build(),
            window_duration,
        }
    }
}

impl LimitingStrategy for MokaCounter {
    fn check_limit(&self, ip: &str, limit_rate: u32) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let window_id = now / self.window_duration.as_millis();
        let key = format!("{}:{}", ip, window_id);

        if let Some(count) = self.records.get(&key) {
            if count < limit_rate {
                self.records.insert(key, count + 1);
                true 
            } else {
                false 
            }
        } else {
            self.records.insert(key, 1);
            true 
        }
    }
}

pub struct LimitingStrategyFactory;

impl LimitingStrategyFactory {
    pub fn init(strategy_name: &str, window_duration: Duration) -> Arc<dyn LimitingStrategy> {
        match strategy_name {
            "no_limiting" => Arc::new(NoLimiting::init()),
            "basic_counter" => Arc::new(BasicCounter::init(window_duration)),
            "moka_counter" => Arc::new(MokaCounter::new(window_duration)),
            _ => Arc::new(MokaCounter::new(window_duration)),
        }
    }
}
