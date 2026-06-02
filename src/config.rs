use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub backend: Backend,
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
    r#type: String,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let yaml_str = std::fs::read_to_string("config.yaml")?;
    let config: Config = serde_yaml::from_str(&yaml_str)?;
    Ok(config)
}

/*
    Lets have the ymal structure to be like 
    backend:
        backend_host:
        ports:
    
    limiting:
        rate:
    
    load_balancing:
        type:



*/
