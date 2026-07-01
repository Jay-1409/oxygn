<p align="center">
  <img src="assets/logo.svg" alt="oxygn Logo" width="200" />
</p>

# Oxygn

> Let your servers breathe, let your clients breathe, all via a simple configuration file.

![License](https://img.shields.io/badge/license-MIT-blue)

- Oxygn is a lightweight, high-performance single-binary reverse proxy server. 

- It operates as a fast TCP tunnel that sits between your clients and backend services, allowing you to seamlessly manage traffic routing without complex setups. 

- oxygn exists to provide a simple, secure, and easily configurable proxy solution that protects your backend applications from overload while guaranteeing continuous availability.


## Features

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
* **Simple Declarative Configuration**: Managed entirely through a single, YAML file.

## Quick Start Guide

- First Install the project using the [installation guide](docs/installation.md)
- Next setup your config.yaml file using the [configuration reference](docs/configuration.md)
- Finally start the proxy using the [startup guide](docs/startup.md)  


## FAQ's And More

- Whats the architecture of this project ? (You can [view](docs/architecture.md))
- Is this project open to **open source contributions** ?  Yes checkout [contribution guide](docs/CONTRIBUTING.MD)
- How can i run benchmarks on my setup? [Using the benchmarking tool](docs/benchmarking.md)


## Benchmarks

> Benchmarks were run on localhost with oxygn and nginx both configured as **TCP stream proxies** (Layer 4), routing to the same pool of 3 Rust async HTTP backends to ensure a fair, apples-to-apples comparison.

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
| **oxygn mode** | `copy_bidirectional` TCP tunnel |

### Results Summary

| Metric | oxygn | Nginx |
|---|---|---|
| **Requests/sec** | 70,774 | 75,204 |
| **Avg Latency** | 2.9 ms | 2.7 ms |
| **Transfer Rate** | 6.16 MB/s | 6.54 MB/s |
| **Socket Errors** | 0 | 0 |

> oxygn achieves **~94% of nginx's throughput** while being a single-binary Rust application with a fraction of the configuration complexity.

### Checkout more graphs [here](docs/graphs.md)


---

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
