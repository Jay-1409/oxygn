use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use mini_moka::sync::Cache;

/*
    Oxygen should support multiple rate limiting strategies
        - some basic strategies are (for more detail: https://www.geeksforgeeks.org/system-design/rate-limiting-algorithms-system-design/)
            - ~Basic Counting strategy~ 
                - fixed window counter strategy
            - Token bucket strategy
            - leaky bucket strategy
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

*/

pub trait LimitingStrategy: Send + Sync { 
    fn check_limit(&self, ip: &str, limit_rate: u32) -> bool;
}


/**
 * Bypass strategy for when no rate limiting strategy has been choosen. 
 *
 * ### Functioning:
 * This strategy acts as a no-op rate limiter. It implements the `LimitingStrategy` trait but 
 * unconditionally returns `true` for all requests, allowing them to pass through without tracking, 
 * state-maintenance, or locking, effectively disabling rate limiting.
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

    ### Functioning:
    This strategy implements the Fixed Window algorithm using a standard `HashMap` protected by a `Mutex`.
    1. It tracks request counts and window start times per client IP address.
    2. When a request arrives, it locks the `Mutex` blocking all other concurrent requests checking limits.
    3. If the current time exceeds the stored window start time by `window_duration`, it resets the count to `1` and updates the window start time.
    4. Otherwise, it checks if the count is less than `limit_rate`. If so, it increments the count and allows the request (`true`); otherwise, it rejects it (`false`).
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
    A moka counter stategy is basically a faster redis rate limiting version only, but its faser 
    since moka cache used the same memoy as the program, thus you do not have to make tcp calls to the redis client
    this allows for a more faster and efficient rate limiting, while preserving the idea. 

    ### Functioning:
    This strategy implements the Fixed Window algorithm using `mini_moka`, an in-memory concurrent cache.
    1. It divides the current unix epoch timestamp in milliseconds by the window duration in milliseconds to get a discrete `window_id`.
    2. It creates a cache key formatted as `"{ip}:{window_id}"`.
    3. It checks `mini_moka::sync::Cache` for the key:
       - If the key exists and the count is under the limit, it increments the count, updates the cache, and returns `true`.
       - If the key exists but the count meets/exceeds the limit, it returns `false`.
       - If the key does not exist (start of a new window/first request), it inserts `1` and returns `true`.
    4. To strictly bound the proxy's RAM footprint, cache capacity is dynamically calculated based on a configurable memory budget (in MB) and an estimated 120 bytes per cache entry.
    5. To prevent memory leaks, cache entries have a TTL configured as the window duration plus a 60-second buffer (`window_duration + 60s`), allowing expired windows to be automatically evicted from memory.
    6. This is highly efficient and thread-safe without the global locking bottleneck found in `BasicCounter`.
**/
pub struct MokaCounter {
    records: Cache<String, u32>,
    window_duration: Duration,
}
/*
    The code idea for this implementation is based on 
        https://dl.acm.org/doi/epdf/10.1145/41457.37504

    
*/
impl MokaCounter {
    pub fn init(window_duration: Duration, memory_budget_mb: u64) -> Self {
        let bytes_per_entry = 120;
        let budget_in_bytes = memory_budget_mb * 1024 * 1024;
        let calculated_capacity = budget_in_bytes / bytes_per_entry;

        let buffer = Duration::from_secs(60);

        Self {
            records: Cache::builder()
                .max_capacity(calculated_capacity) // <---- we pass in the max capacity for moka counter
                .time_to_live(window_duration + buffer) 
                .build(),
            window_duration,
        }
    }
}
impl LimitingStrategy for MokaCounter {
    /*
        Math behind the Fixed Window Rate Limiting:
        1. Epoch Time (ms): `now` gets the current timestamp in milliseconds.
        2. Window ID Calculation: `window_id = now / window_duration_ms`
           Using integer division, any timestamp falling within the same duration window maps to the exact same integer ID.
           For example, if window_duration is 10,000ms (10s):
             - At 15,000ms: 15,000 / 10,000 = 1 (Window ID 1)
             - At 19,990ms: 19,990 / 10,000 = 1 (Window ID 1)
             - At 20,000ms: 20,000 / 10,000 = 2 (Window ID 2 - new window, limit resets)
        3. Cache Key: `"{ip}:{window_id}"` maps each client's IP to their specific current time window.
           When the time moves to the next window, the ID changes, looking up a new key and resetting the limit automatically.
    */
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
    pub fn init(strategy_name: &str, window_duration: Duration, memory_budget_mb: u64) -> Arc<dyn LimitingStrategy> {
        match strategy_name {
            "no_limiting" => Arc::new(NoLimiting::init()),
            "basic_counter" => Arc::new(BasicCounter::init(window_duration)),
            "moka_counter" => Arc::new(MokaCounter::init(window_duration, memory_budget_mb)),
            _ => Arc::new(MokaCounter::init(window_duration, memory_budget_mb)),
        }
    }
}
