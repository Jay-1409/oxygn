use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub backend: Vec<BackendConfig>,
    pub limiting: Limiting,
    pub load_balancing: LoadBalancing,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendConfig {
    pub backend_host: String,
    pub ports: Vec<u16>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Limiting {
    pub rate: u32,
}

#[derive(Debug, Deserialize, Clone)]
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
