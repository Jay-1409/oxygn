# Oxygen

<p align="center">
  <img src="assets/logo.svg" alt="Oxygen Logo" width="200" />
</p>

**Let your servers breathe, let your clients breathe, all via a simple configuration file.**

Introducing **Oxygen**, a lightweight, high-performance single-binary reverse proxy server built in Rust. Oxygen operates as a high-speed raw TCP tunnel, enabling seamless routing, request load balancing, concurrent health checking, and strict rate limiting.

---

## 🚀 Key Features

* **High-Performance Asynchronous Runtime**: Powered entirely by [Tokio](https://github.com/tokio-rs/tokio) for non-blocking I/O multiplexing.
* **TCP & Application Tunneling**: Supports raw TCP proxying, making it compatible with HTTP/1.1, HTTP/2, HTTPS (SSL/TLS handshakes passed through directly), WebSockets, Database streams (Postgres, MySQL, Redis), SSH, gRPC, and more.
* **Load Balancing**: Distributes incoming connections across a pool of backends using an atomic, lock-free **Round-Robin** strategy.
* **Concurrent Health Check Pooler**: 
  * Periodically polls unhealthy servers to automatically bring them back online.
  * Uses `tokio::task::JoinSet` to run checks concurrently instead of sequentially.
  * Throttled with `tokio::sync::Semaphore` to cap active network connection attempts (maximum 100).
* **Flexible Health Check types**:
  * **TCP Ping**: Fast, low-overhead check testing port connection.
  * **HTTP Check**: Performs an HTTP `GET` request and verifies the response status code is successful (`200`-`399`), ensuring the application is truly healthy.
* **Memory-Bounded Rate Limiting**: Enforces rate limiting per IP address using [Mini Moka](https://github.com/moka-rs/mini-moka) concurrent cache. The cache's memory footprint is strictly bounded based on a configurable memory budget.

---

## 🛠️ Configuration Guide

Oxygen is configured via a simple `config.yaml` file located in the workspace root. Below is a detailed breakdown of the available configuration blocks:

```yaml
# 1. Oxygen proxy listener port
oxygen:
  port: 8000                  # The port Oxygen listens on for incoming client connections.

# 2. Load Balancing Strategy
load_balancing:
  strategy: "round_robin"     # The routing strategy. Options: "round_robin".

# 3. Rate Limiting Settings
limiting:
  strategy: "moka_counter"    # Rate limit counter strategy. Options: "moka_counter", "basic_counter", "no_limiting".
  rate: 100                   # Maximum requests allowed per IP address within the window.
  window_secs: 60             # Time window size in seconds.
  memory_budget_mb: 50        # Strictly bounds memory capacity in MB for cache entries (moka_counter only).

# 4. Background Health Checking Settings
health_check:
  interval_secs: 2            # How often (in seconds) the background task polls unhealthy backends.
  check_type: "http"          # Health check protocol. Options: "tcp" (ping check) or "http" (GET request).
  path: "/"                   # Target path for GET checks (applicable only for "http" check_type).

# 5. Backend Server Pools
# You can specify multiple backend host machines, and assign multiple ports for each host.
backend:
  - backend_host: "127.0.0.1"
    ports:
      - 8080
      - 8081
      - 8082
```

---

## 📦 Core Technologies & Libraries

Oxygen leverages several industry-standard Rust libraries to achieve high concurrency and performance:

* [Tokio](https://github.com/tokio-rs/tokio): The primary asynchronous runtime for driving TCP connections, spawning tasks concurrently, and synchronization (Semaphores, Intervals, JoinSets).
* [Mini Moka](https://github.com/moka-rs/mini-moka): A fast, concurrent cache library used to implement thread-safe rate-limiting windows with configurable TTL eviction.
* [Serde](https://serde.rs/) & [Serde YAML](https://github.com/dtolnay/serde-yaml): Used for fast, zero-boilerplate configuration parsing and mapping.

---

## 🛠️ How to Build and Run

### Prerequisites
Ensure you have the Rust toolchain installed. If not, install it from [rustup.rs](https://rustup.rs/).

### Steps

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/Jay-1409/oxygen.git
   cd oxygen
   ```

2. **Configure your Proxy**:
   Copy the `example.config.yaml` to `config.yaml` and modify it:
   ```bash
   cp example.config.yaml config.yaml
   ```

3. **Build the Release Binary**:
   ```bash
   cargo build --release
   ```

4. **Start the Proxy Server**:
   ```bash
   cargo run --release
   ```
