use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use anyhow::{anyhow, Result};
use tokio::sync::Mutex;
use tokio::task::spawn;
use async_trait::async_trait;
use futures::future::join_all;

use crate::consts::HEALTH_CHECK_INTERVAL_MS;
use crate::endpoint::Endpoint;
use crate::strategy::context::StrategyContext;
use crate::strategy::RouteStrategy;

#[async_trait]
pub trait LoadBalancer: Sync + Send
{
    #[allow(dead_code)]
    fn set_strategy(&mut self, strategy: Box<dyn RouteStrategy>);
    async fn handle(&self, req: String) -> Result<String>;
    fn health_maintain(&self);

    fn stop_health_maintain(&self);
}

pub struct LoadBalancerImpl
{
    endpoints: Arc<Vec<Arc<Box<dyn Endpoint>>>>,
    router_strategy: Mutex<Box<dyn RouteStrategy>>,
    close_signal_receiver: Arc<Mutex<Receiver<bool>>>,
    close_signal_sender: Sender<bool>,
}

impl LoadBalancerImpl
{
    pub fn new(endpoints: Vec<Arc<Box<dyn Endpoint>>>, strategy: Box<dyn RouteStrategy>) -> Self {
        let (tx, rx) = mpsc::channel();
        LoadBalancerImpl {
            endpoints: Arc::new(endpoints),
            router_strategy: Mutex::new(strategy),
            close_signal_receiver: Arc::new(Mutex::new(rx)),
            close_signal_sender: tx,
        }
    }

    async fn pick_endpoint(&self, ctx: &StrategyContext) -> Option<Arc<Box<dyn Endpoint>>> {
        let mut strategy = self.router_strategy.lock().await;
        strategy.pick(ctx, &self.filter_healthy_endpoints())
    }

    fn filter_healthy_endpoints(&self) -> Vec<Arc<Box<dyn Endpoint>>> {
        self.endpoints
            .iter()
            .filter(|endpoint| endpoint.health_report())
            .map(Arc::clone)
            .collect()
    }

    fn build_strategy_ctx(req: String) -> StrategyContext {
        StrategyContext::new(req)
    }
}

#[async_trait]
impl LoadBalancer for LoadBalancerImpl
{
    fn set_strategy(&mut self, strategy: Box<dyn RouteStrategy>) {
        self.router_strategy = Mutex::new(strategy);
    }
    async fn handle(&self, req: String) -> Result<String> {
        let endpoint = self.pick_endpoint(&Self::build_strategy_ctx(req.clone())).await
            .ok_or_else(|| anyhow!("assign endpoint failed, no proper endpoint found"))?;
        tracing::info!("[LoadBalancer] request forwarded to server [Name: {}, Addr:{}], request={}", endpoint.name(), endpoint.addr(), req);
        endpoint.handle(&req).await
    }

    fn health_maintain(&self) {
        let close_signal = Arc::clone(&self.close_signal_receiver);
        let endpoints = Arc::clone(&self.endpoints);

        spawn(async move {
            loop {
                match close_signal.lock().await.try_recv() {
                    Ok(_) => {
                        return;
                    }
                    Err(_) => {
                        let handlers = endpoints.iter().map(|endpoint| async {
                            endpoint.health_check().await
                        });
                        join_all(handlers).await;
                        thread::sleep(HEALTH_CHECK_INTERVAL_MS)
                    }
                }
            }
        });
    }

    fn stop_health_maintain(&self) {
        self.close_signal_sender.send(true).unwrap_or_else(
            |err| {
                tracing::warn!(?err, "load balancer exit failed");
                ()
            }
        )
    }
}

#[cfg(test)]
mod load_balancer_impl_tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;

    use crate::endpoint::MockEndpoint;
    use crate::strategy::MockRouteStrategy;

    use super::*;

    #[tokio::test]
    async fn test_health_maintain() {
        // healthy instances
        let mut endpoint1 = MockEndpoint::new();
        endpoint1.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
        endpoint1.expect_health_report().returning(|| true);
        let mut endpoint2 = MockEndpoint::new();
        endpoint2.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081));
        endpoint2.expect_health_report().returning(|| true);
        // unhealthy instance
        let mut endpoint3 = MockEndpoint::new();
        endpoint3.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082));
        endpoint3.expect_health_report().returning(|| false);

        let endpoints: Vec<Arc<Box<dyn Endpoint>>> = vec![
            Arc::new(Box::new(endpoint1)),
            Arc::new(Box::new(endpoint2)),
            Arc::new(Box::new(endpoint3)),
        ];
        let expectation1 = Arc::clone(&endpoints[0]);
        let expectation2 = Arc::clone(&endpoints[1]);
        let expectation3 = Arc::clone(&endpoints[2]);
        let lb = LoadBalancerImpl::new(endpoints, Box::new(MockRouteStrategy::new()));
        lb.health_maintain();
        thread::sleep(Duration::from_millis(1000));
        let endpoints_addr: Vec<SocketAddr> = lb.filter_healthy_endpoints().iter().map(|endpoint| endpoint.addr()).collect();
        assert!(endpoints_addr.contains(&expectation1.addr()));
        assert!(endpoints_addr.contains(&expectation2.addr()));
        assert!(!endpoints_addr.contains(&expectation3.addr()));
    }
}