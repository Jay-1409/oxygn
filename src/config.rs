use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub backend: Vec<Backend>,
    pub limiting: Limiting,
    pub load_balancing: LoadBalancing,
}

#[derive(Debug, Deserialize)]
pub struct Backend {
    pub backend_host: String,
    pub ports: Vec<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Limiting {
    pub rate: u32,
}

#[derive(Debug, Deserialize)]
pub struct LoadBalancing {
    #[serde(default = "default_strategy")]
    pub strategy: String,
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
        rate: 100

    load_balancing:
        strategy: "round_robin" <--- user can pass in the load balancing strategy that they want to use

*/
