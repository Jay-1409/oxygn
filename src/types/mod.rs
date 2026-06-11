// Copyright (c) 2026 Jay 'jay-1409' Shah. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for details.
//
// File: src/types/mod.rs
// Purpose: Declares types module and defines backend structures.

pub mod config;

use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Backend {
    pub host: String,
    pub port: u16,
    pub health: bool,
    /// Pre-computed SocketAddr — avoids a format!()+parse() allocation on every connection.
    pub addr: SocketAddr,
}
