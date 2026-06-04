use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub backend: Vec<BackendConfig>,
    pub limiting: Limiting,
    pub load_balancing: LoadBalancing,
    pub oxygen: Oxygen,
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
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoadBalancing {
    #[serde(default = "default_strategy")]
    pub strategy: String,
}

fn default_limiting_strategy() -> String {
    "no_limiting".to_string()
}

fn default_window_secs() -> u64 {
    1
}

fn default_strategy() -> String {
    "round_robin".to_string()
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
