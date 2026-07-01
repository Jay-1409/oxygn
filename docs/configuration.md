
## Configuration Reference

oxygn is configured via a `config.yaml` file located in the directory where the binary is executed.

```yaml
# 1. oxygn server configuration (Required block)
oxygn:
  # The port on which the oxygn proxy listens for incoming client requests.
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

# 5. Networking configuration (Optional block)
networking:
  # Whether to use TCP_NODELAY (disables Nagle's algorithm) to forward packets immediately.
  # Set this to true for low-latency routing, or false to buffer packet chunks.
  # (Optional, Default: true)
  tcp_nodelay: true

# 6. Backend Pool configuration (Required block)
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

Suppose you have three Node.js processes running locally on ports `3001`, `3002`, and `3003`. You want oxygn to expose them on port `80`, balance traffic evenly, and ensure no client IP makes more than 50 requests per minute.

```yaml
oxygn:
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
### Setup your config.yaml file
oxygn requires a `config.yaml` file in the directory where you execute it. 
   ```bash
   # If you installed via cargo, download the example config:
   curl -o config.yaml https://raw.githubusercontent.com/Jay-1409/oxygn/main/example.config.yaml
   
   # Or if you cloned the repository:
   cp example.config.yaml config.yaml
   ```

## Next [running oxygn server](startup.md)