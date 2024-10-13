use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::consts::WEIGHTED_ROUND_ROBIN;

#[derive(Default, Debug, Deserialize)]
pub struct EndpointPoolConfig {
    endpoints: Vec<EndpointConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EndpointConfig {
    name: String,
    ip: Ipv4Addr,
    port: u16,
    weight: Option<u8>,
}

impl EndpointPoolConfig {
    pub fn load(path: &Path, strategy: &str) -> Result<Self> {
        let config_content = fs::read_to_string(path)
            .with_context(|| format!("failed to read endpoints config file:{:?}", path))?;
        let mut new: EndpointPoolConfig = toml::from_str(&config_content)
            .with_context(|| format!("failed to parse endpoints config file:{:?}", path))?;

        new.filter(strategy);
        new.check()?;

        Ok(new)
    }

    pub fn endpoint_configs(self) -> Vec<EndpointConfig> {
        self.endpoints
    }

    fn check(&self) -> Result<()> {
        // checkers
        Ok(())
    }

    fn filter(&mut self, strategy: &str) {
        self.filter_by_weight(strategy);
    }

    fn filter_by_weight(&mut self, strategy: &str) {
        if strategy != WEIGHTED_ROUND_ROBIN { return; }
        self.endpoints = self.endpoints.iter()
            .filter(
                |&config|
                config.weight.is_some() && (config.weight.unwrap() <= 100)
            )
            .cloned()
            .collect();
    }
}

impl EndpointConfig {
    pub fn get_socket_addr(&self) -> SocketAddr {
        SocketAddr::new((self.ip).into(), self.port)
    }

    #[allow(dead_code)]
    pub fn ip(&self) -> Ipv4Addr {
        self.ip
    }
    #[allow(dead_code)]
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn weight(&self) -> Option<u8> {
        self.weight
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::consts::DEFAULT_STRATEGY;
    use crate::model::endpoints_config::EndpointPoolConfig;

    use super::*;

    #[test]
    fn test_load() {
        let pool_config = EndpointPoolConfig::load(Path::new("src/config/endpoints_test.toml"), DEFAULT_STRATEGY);
        assert!(pool_config.is_ok());
        let dataset = vec![
            EndpointConfig {
                name: "s1".to_string(),
                ip: "192.168.1.1".parse().unwrap(),
                port: 8080,
                weight: Some(80),
            },
            EndpointConfig {
                name: "s2".to_string(),
                ip: "192.168.1.2".parse().unwrap(),
                port: 8081,
                weight: Some(10),
            },
            EndpointConfig {
                name: "s3".to_string(),
                ip: "192.168.1.3".parse().unwrap(),
                port: 8082,
                weight: Some(10),
            },
        ];
        let server_configs = pool_config.unwrap().endpoints;
        assert_eq!(dataset.len(), server_configs.len());
        for (i, config) in server_configs.iter().enumerate() {
            assert_eq!(config.ip, dataset[i].ip);
            assert_eq!(config.name, dataset[i].name);
            assert_eq!(config.port, dataset[i].port);
            assert_eq!(config.weight, dataset[i].weight);
        }
    }

    #[test]
    fn test_load_failed() {
        assert!(EndpointPoolConfig::load(Path::new("src/config/endpoints_test_invalid.toml"), DEFAULT_STRATEGY).is_err());
    }
}