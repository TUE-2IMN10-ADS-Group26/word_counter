use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::consts::{DEFAULT_IP_ADDR, DEFAULT_METRICS_PORT, DEFAULT_PORT};
use crate::endpoint::word_counter::WordCountResponse;

#[derive(Default, Debug, Deserialize, Clone)]
pub struct ServerConfig {
    ip: Option<Ipv4Addr>,
    port: Option<u16>,
    metrics_port: Option<u16>,
    enable_fault_tolerance: Option<bool>,
}

impl ServerConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let config_content = fs::read_to_string(path)
            .with_context(|| format!("failed to read server config file:{:?}", path))?;
        toml::from_str(&config_content)
            .with_context(|| format!("failed to parse server config file:{:?}", path))
    }
    pub fn ip(&self) -> &Ipv4Addr {
        self.ip.as_ref().unwrap_or_else(|| {
            tracing::error!("IP address is None, using default value");
            &DEFAULT_IP_ADDR
        })
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap_or_else(|| {
            tracing::error!("port is None, using default value");
            DEFAULT_PORT
        })
    }

    pub fn metrics_port(&self) -> u16 {
        self.metrics_port.unwrap_or_else(|| {
            tracing::error!("metrics port is None, using default value");
            DEFAULT_METRICS_PORT
        })
    }

    pub fn get_socket_addr(&self) -> SocketAddr {
        SocketAddr::new((*self.ip()).into(), self.port())
    }

    pub fn get_metrics_addr(&self) -> SocketAddr {
        SocketAddr::new((*self.ip()).into(), self.metrics_port())
    }

    pub fn fault_tolerance(&self) -> bool {
        self.enable_fault_tolerance.unwrap_or_default()
    }
}

impl WordCountResponse {
    pub fn failed_resp() -> Self {
        WordCountResponse {
            count: 0,
            status_code: -1,
            status_message: "some error occurred...".to_string(),
            log_id: "0".to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_load() {
        let server_config = ServerConfig::load(Path::new("src/config_test/server_test.toml"));
        assert!(server_config.is_ok());
        let server_config = server_config.unwrap();
        let expected = ServerConfig {
            ip: Some("192.168.1.1".parse().unwrap()),
            port: Some(8080),
            metrics_port: Some(8081),
            enable_fault_tolerance: Some(true),
        };
        assert_eq!(server_config.ip, expected.ip);
        assert_eq!(server_config.port, expected.port);
        assert_eq!(server_config.metrics_port, expected.metrics_port);
        assert_eq!(server_config.enable_fault_tolerance, expected.enable_fault_tolerance);
    }
}