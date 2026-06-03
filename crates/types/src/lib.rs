pub mod config;

#[derive(Debug, Clone)]
pub struct Backend {
    pub host: String,
    pub port: u16,
    pub health: bool,
}
