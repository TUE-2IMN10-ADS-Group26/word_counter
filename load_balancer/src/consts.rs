use std::net::Ipv4Addr;
use std::time::Duration;

// server basic config
pub const DEFAULT_IP_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_METRICS_PORT: u16 = 8081;
pub const HEALTH_CHECK_INTERVAL_MS: Duration = Duration::from_millis(500);

// strategy
pub const DEFAULT_STRATEGY: &str = "RoundRobin";
pub const ROUND_ROBIN: &str = "RoundRobin";
pub const WEIGHTED_ROUND_ROBIN: &str = "WeightedRoundRobin";

// config files
pub const CONFIG_PATH_ENDPOINTS: &str = "src/config/endpoints.toml";
pub const CONFIG_PATH_LOAD_BALANCER: &str = "src/config/load_balancer.toml";
pub const CONFIG_PATH_SERVER: &str = "src/config/server.toml";

// metrics
pub const COUNTER_QUERY: &str = "query";
pub const COUNTER_LATENCY: &str = "latency";