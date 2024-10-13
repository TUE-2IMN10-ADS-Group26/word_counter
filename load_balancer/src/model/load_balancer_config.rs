use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::consts::DEFAULT_STRATEGY;

#[derive(Debug, Deserialize)]
pub struct LBConfig {
    strategy: Option<String>,
}

impl LBConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let config_content = fs::read_to_string(path)?;
        toml::from_str(&config_content).context("load server.toml failed")
    }

    pub fn strategy(&self) -> String {
        self.strategy.clone().unwrap_or_else(|| {
            tracing::error!("strategy is None, using default value");
            DEFAULT_STRATEGY.to_string()
        })
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_load() {
        let lb_config = LBConfig::load(Path::new("src/config/load_balancer_test.toml"));
        assert!(lb_config.is_ok());
        let lb_config = lb_config.unwrap();
        assert_eq!(lb_config.strategy(), "WeightedRoundRobin");
    }
}