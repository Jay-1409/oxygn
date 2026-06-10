use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub backend: Vec<BackendConfig>,
    pub limiting: Limiting,
    pub load_balancing: LoadBalancing,
    pub oxygen: Oxygen,
    #[serde(default)]
    pub health_check: HealthCheck,
    #[serde(default)]
    pub networking: Networking,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Oxygen {
    pub port: u16
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendConfig {
    pub backend_host: String,
    pub ports: Vec<u16>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Limiting {
    #[serde(default = "default_limiting_strategy")]
    pub strategy: String,
    pub rate: u32,
    #[serde(default = "default_window_secs")]
    pub window_secs: u64,
    #[serde(default = "default_memory_budget_mb")]
    pub memory_budget_mb: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoadBalancing {
    #[serde(default = "default_strategy")]
    pub strategy: String,
}
// TODO: Move the defaults to a new defaults.rs
fn default_limiting_strategy() -> String {
    "no_limiting".to_string()
}

fn default_window_secs() -> u64 {
    1
}

fn default_memory_budget_mb() -> u64 {
    100
}

fn default_strategy() -> String {
    "round_robin".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct HealthCheck {
    #[serde(default = "default_health_check_interval_secs")]
    pub interval_secs: u64,
    #[serde(default = "default_health_check_type")]
    pub check_type: String,
    #[serde(default = "default_health_check_path")]
    pub path: String,
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            interval_secs: default_health_check_interval_secs(),
            check_type: default_health_check_type(),
            path: default_health_check_path(),
        }
    }
}

fn default_health_check_interval_secs() -> u64 {
    2
}

fn default_health_check_type() -> String {
    "tcp".to_string()
}

fn default_health_check_path() -> String {
    "/health".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct Networking {
    #[serde(default = "default_tcp_nodelay")]
    pub tcp_nodelay: bool,
}

impl Default for Networking {
    fn default() -> Self {
        Self {
            tcp_nodelay: default_tcp_nodelay(),
        }
    }
}

fn default_tcp_nodelay() -> bool {
    true
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let yaml_str = std::fs::read_to_string("config.yaml")?;
    let config: Config = serde_yaml::from_str(&yaml_str)?;
    Ok(config)
}

/*
    Lets have the ymal structure to be like 
    backend:
    - backend_host: "127.0.0.1".    <-- multiple host machines can be configured, and on each machine multiple ports can be configured. 
        ports:
        - 8080
        - 8081
    - backend_host: "127.0.0.2" 
        ports:
        - 9090

    limiting:   <--- optional feature, rate limiting [in scope]
        strategy:           <---- choose from multiple rate limiting strategies, by defeult no rate limmiting is done
        rate: 100

    load_balancing:
        strategy: "round_robin" <--- user can pass in the load balancing strategy that they want to use

    oxygen:
        port: 8000    <---- this is the port on which oxygen runs on the host machine
*/
