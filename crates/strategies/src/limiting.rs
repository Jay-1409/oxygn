use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/**
    Oxygen should support multiple rate limiting strategies 
        - Basic Counting strategy
        - Token bucket strategy 
        - leaky bucket strategy 
        - fixed window counter strategy
        - sliding window log 
        - sliding window counter 


**/
pub trait LimitingStrategy: Send + Sync { 
    fn check_limit(&self, ip: &str, limit_rate: u32) -> bool;
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
    Below is an interface for dynanmically, choosing an routing strategy based 
    on the config.yaml file.
**/
pub struct LimitingStrategyFactory;

impl LimitingStrategyFactory {
    pub fn init(strategy_name: &str) -> Arc<dyn LimitingStrategy> {
        match strategy_name {
            "basic_counter" => Arc::new(BasicCounter::new(Duration::from_secs(60))),
            _ => panic!("Unknown rate limiting strategy requested in config!"),
        }
    }
}