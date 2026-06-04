use types::config;
use managers::backend::BackendPool;
use network::listner::Listener;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load_config()?;
    let pool = BackendPool::init(&config);
    pool.spawn_health_pooler(Duration::from_secs(2));
    let listener = Listener::init(config.oxygen.port, pool);
    println!("Loaded config successfully:\n{:#?}", config);
    listener.run().await?;    
    Ok(())
}
