<p align="center">
  <img src="assets/logo.svg" alt="Oxygen Logo" width="200" />
</p>

# Oxygn

> Let your servers breathe, let your clients breathe, all via a simple configuration file.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)
![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Language](https://img.shields.io/badge/language-Rust-orange)

Oxygn is a lightweight, high-performance single-binary reverse proxy server. It operates as a fast TCP tunnel that sits between your clients and backend services, allowing you to seamlessly manage traffic routing without complex setups. Oxygen exists to provide a simple, secure, and easily configurable proxy solution that protects your backend applications from overload while guaranteeing continuous availability.


## 🚀 Key Features

* **Universal Protocol Support**: Routes raw TCP traffic directly.
  * Proxies HTTP/1.1, HTTP/2, HTTPS (SSL/TLS passthrough), WebSockets, and database streams seamlessly.
* **Intelligent Load Balancing**: Distributes incoming client connections evenly across multiple backend servers.
  * Ensures no single server is overwhelmed by requests.
  * Automatically handles routing across different host machines and ports.
* **Active Health Checking**: Continuously monitors the health of your backend pool.
  * Performs fast, parallel checks to detect outages without delaying client requests.
  * Automatically removes unresponsive servers from the routing pool.
  * Supports lightweight TCP ping checks and robust HTTP application-level checks.
* **Strict Rate Limiting**: Protects your services from abuse and traffic spikes.
  * Enforces maximum request limits per client IP address within configurable time windows.
  * Operates with a strictly bounded memory footprint, ensuring the proxy never consumes excessive system resources.
* **Simple Declarative Configuration**: Managed entirely through a single, easy-to-read YAML file.

## ⚡ Quick Start

### Prerequisites
Ensure you have the Rust toolchain installed.

### Installation

**Option 1: Using Cargo (Recommended)**
If you have the Rust toolchain installed, you can install Oxygen directly from crates.io:
```bash
cargo install oxygn
```
*(Note: The package name is `oxygn`, but the binary installed is `oxygen`)*

**Option 2: Build from Source**
1. **Clone the Repository**:
   ```bash
   git clone https://github.com/Jay-1409/oxygen.git
   cd oxygen
   ```

2. **Build**:
   ```bash
   cargo build --release
   ```

### Running Oxygen

1. **Configure your Proxy**:
   Oxygen requires a `config.yaml` file in the directory where you execute it. 
   ```bash
   # If you installed via cargo, download the example config:
   curl -o config.yaml https://raw.githubusercontent.com/Jay-1409/oxygen/main/example.config.yaml
   
   # Or if you cloned the repository:
   cp example.config.yaml config.yaml
   ```

2. **Start the Proxy**:
   ```bash
   # If you installed via cargo:
   oxygen
   
   # If running from source:
   cargo run --release
   ```

## 🛠️ Configuration Reference

Oxygen is configured via a `config.yaml` file located in the directory where the binary is executed.

```yaml
# 1. Oxygen server configuration (Required block)
oxygen:
  # The port on which the Oxygen proxy listens for incoming client requests.
  # (Required)
  port: 8000

# 2. Load Balancing configuration (Required block)
load_balancing:
  # The strategy used to route traffic between healthy backend servers.
  # Options: "round_robin" (alternates requests sequentially).
  # (Optional, Default: "round_robin")
  strategy: "round_robin"

# 3. Rate Limiting configuration (Required block)
limiting:
  # The strategy used to enforce rate limits per client IP.
  # Options: 
  #   - "no_limiting" (disables rate limiting)
  #   - "moka_counter" (fast, memory-budget-aware fixed window counter)
  #   - "basic_counter" (simple fixed window counter)
  # (Optional, Default: "no_limiting")
  strategy: "moka_counter"
  
  # The maximum number of requests a single client IP can make within the window.
  # (Required, must be provided as an integer even if strategy is "no_limiting")
  rate: 100
  
  # The length of the rate limiting time window in seconds.
  # (Optional, Default: 1)
  window_secs: 60
  
  # The maximum memory footprint allowed for rate limiting data in megabytes (MB).
  # Prevents the proxy from consuming excessive RAM during high traffic.
  # (Optional, Default: 100. Applicable only for "moka_counter" strategy)
  memory_budget_mb: 50

# 4. Background Health Checking configuration (Optional block)
# If this entire block is omitted, it defaults to checking via "tcp" every 2 seconds.
health_check:
  # How often (in seconds) the proxy polls unhealthy servers to check if they are back online.
  # (Optional, Default: 2)
  interval_secs: 2
  
  # The method used to verify server health.
  # Options:
  #   - "tcp" (checks if the server accepts TCP connections)
  #   - "http" (sends an HTTP GET request and expects a 200-399 status code)
  # (Optional, Default: "tcp")
  check_type: "tcp"
  
  # The endpoint path to query (applicable only when check_type is "http").
  # (Optional, Default: "/health")
  path: "/health"

# 5. Backend Pool configuration (Required block)
# Defines the list of backend servers that Oxygen will proxy traffic to.
backend:
  # The IP address or hostname of the physical backend machine.
  - backend_host: "127.0.0.1"
    # The list of application ports running on this host.
    ports:
      - 8080
      - 8081
      - 8082
```

## 📖 Usage Examples

### Example: Proxying a Node.js Web App

Suppose you have three Node.js processes running locally on ports `3001`, `3002`, and `3003`. You want Oxygen to expose them on port `80`, balance traffic evenly, and ensure no client IP makes more than 50 requests per minute.

```yaml
oxygen:
  port: 80

load_balancing:
  strategy: "round_robin"

limiting:
  strategy: "moka_counter"
  rate: 50
  window_secs: 60
  memory_budget_mb: 10

health_check:
  interval_secs: 5
  check_type: "http"
  path: "/api/health"

backend:
  - backend_host: "127.0.0.1"
    ports:
      - 3001
      - 3002
      - 3003
```
## 🏗️ Architecture

![Architecture Diagram](assets/design.png)

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. (Check TODO.md for some ideas)

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📊 Benchmarks

> Benchmarks were run on localhost with Oxygen and nginx both configured as **TCP stream proxies** (Layer 4), routing to the same pool of 3 Rust async HTTP backends to ensure a fair, apples-to-apples comparison.

### Test Environment

| Parameter | Value |
|---|---|
| **Tool** | [`wrk`](https://github.com/wg/wrk) |
| **Duration** | 30 seconds |
| **Threads** | 4 |
| **Connections** | 200 concurrent |
| **Target** | `http://127.0.0.1:8000/` |
| **Backends** | 3 × Tokio async HTTP servers (`127.0.0.1:8080/8081/8082`) |
| **Nginx mode** | `stream {}` (TCP proxy) |
| **Oxygen mode** | `copy_bidirectional` TCP tunnel |

### Results Summary

| Metric | Oxygen | Nginx |
|---|---|---|
| **Requests/sec** | 70,774 | 75,204 |
| **Avg Latency** | 2.9 ms | 2.7 ms |
| **Transfer Rate** | 6.16 MB/s | 6.54 MB/s |
| **Socket Errors** | 0 | 0 |

> Oxygen achieves **~94% of nginx's throughput** while being a single-binary Rust application with a fraction of the configuration complexity.

### Throughput (Higher is Better)

![RPS Benchmark](assets/benchmark_rps.png)

### Latency (Lower is Better)

![Latency Benchmark](assets/benchmark_latency.png)

### Transfer Rate (Higher is Better)

![Transfer Benchmark](assets/benchmark_transfer.png)

### Memory Usage Over Time (Lower is Better)

![Memory Benchmark](assets/benchmark_memory.png)

---

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
