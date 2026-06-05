use types::config;
use managers::pool::BackendPool;
use network::listner::Listener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load_config()?;
    let pool = BackendPool::init(&config);
    pool.spawn_health_pooler(&config.health_check);
    let listener = Listener::init(config.oxygen.port, pool);
    println!("Loaded config successfully:\n{:#?}", config);
    listener.run().await?;    
    Ok(())
}
