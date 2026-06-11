// Copyright (c) 2026 Jay 'jay-1409' Shah. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for details.
//
// File: src/main.rs
// Purpose: Application entry point. Loads configuration and runs the proxy server.

pub mod types;
pub mod strategies;
pub mod managers;
pub mod network;

use crate::types::config;
use crate::managers::pool::BackendPool;
use crate::network::listner::Listener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load_config()?;
    let pool = BackendPool::init(&config);
    pool.spawn_health_pooler(&config.health_check);
    let listener = Listener::init(config.oxygen.port, pool, config.networking.tcp_nodelay);
    println!("Loaded config successfully:\n{:#?}", config);
    listener.run().await?;    
    Ok(())
}
