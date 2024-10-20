use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use mockall::automock;
use once_cell::sync::OnceCell;
use tonic::Request;
use tonic::transport::{Channel, Uri};
use tonic_health::pb::health_check_response::ServingStatus;
use tonic_health::pb::health_client::HealthClient;
use tonic_health::pb::HealthCheckRequest;

use word_counter::counter_client::CounterClient;
use word_counter::WordCountRequest;

use crate::metrics::QueryCounter;
use crate::model::endpoints_config::EndpointConfig;

pub mod word_counter {
    include!("generated/word_counter.rs");
}

#[async_trait]
#[automock]
pub trait Endpoint: Send + Sync {
    fn name(&self) -> String;
    #[allow(dead_code)]
    fn addr(&self) -> SocketAddr;

    // for weighted-round-robin
    fn weight(&self) -> Option<u8>;
    async fn handle(&self, req: &str) -> Result<String>;
    async fn health_check(&self);
    fn health_report(&self) -> bool;
}

pub struct WordCountServer {
    config: EndpointConfig,
    counter_client: OnceCell<CounterClient<Channel>>,
    health_client: OnceCell<HealthClient<Channel>>,
    channel: Option<Channel>,
    is_health: AtomicBool,
}

impl WordCountServer {
    pub async fn build(&mut self) -> Result<()> {
        self.connect_channel().await?;
        self.create_counter_client()?;
        self.create_health_client()?;
        Ok(())
    }
    pub fn new(config: EndpointConfig) -> Self {
        WordCountServer {
            config,
            counter_client: OnceCell::new(),
            health_client: OnceCell::new(),
            channel: None,
            is_health: AtomicBool::default(),
        }
    }

    async fn connect_channel(&mut self) -> Result<()> {
        let uri: Uri = Uri::from_str(&format!("http://{}", self.config.get_socket_addr())).context("format endpoint uri failed")?;
        // hardcoded connection configs, todo!
        let inner_endpoint = Channel::builder(uri)
            .connect_timeout(Duration::from_secs(5))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .timeout(Duration::from_secs(5));
        self.channel = Some(inner_endpoint.connect().await.context("rpc channel connect failed")?);
        Ok(())
    }

    fn create_counter_client(&mut self) -> Result<()> {
        if self.counter_client.get().is_some() {
            return Ok(());
        }
        let channel = self.channel.as_ref().ok_or(anyhow!("create CounterClient failed: channel not connected"))?;
        self.counter_client.set(CounterClient::new(channel.clone())).map_err(|e| anyhow!("Failed to set counter client: {:?}", e))?;
        Ok(())
    }

    fn create_health_client(&self) -> Result<()> {
        if self.health_client.get().is_some() {
            return Ok(());
        }
        let channel = self.channel.as_ref().ok_or(anyhow!("create HealthClient failed: channel not connected"))?;
        self.health_client.set(HealthClient::new(channel.clone())).map_err(|e| anyhow!("Failed to set health client: {:?}", e))?;
        Ok(())
    }

    fn counter_client(&self) -> Option<CounterClient<Channel>> {
        self.counter_client.get().cloned().or_else(|| {
            tracing::error!("counter client not initialized");
            None
        })
    }

    fn health_client(&self) -> Option<HealthClient<Channel>> {
        self.health_client.get().cloned().or_else(|| {
            tracing::warn!("health client not initialized");
            None
        })
    }

    fn parse(req: &str) -> Result<Request<WordCountRequest>> {
        let req: WordCountRequest = serde_json::from_str(req).context("parse request failed")?;
        Ok(Request::new(req))
    }

    fn update_health_status(&self, status: i32) {
        let updated = ServingStatus::try_from(status)
            .map_or(false, |status| status == ServingStatus::Serving);
        self.is_health.store(updated, Ordering::SeqCst);
        if !self.is_health.load(Ordering::SeqCst) {
            self.log_unhealthy_instance()
        }
    }

    fn log_unhealthy_instance(&self) {
        tracing::warn!("[LoadBalancer] unhealthy downstream instance:[name:{}, addr{}]",self.name(),self.addr());
    }
}

#[async_trait]
impl Endpoint for WordCountServer {
    fn name(&self) -> String {
        self.config.name()
    }

    fn addr(&self) -> SocketAddr {
        self.config.get_socket_addr()
    }

    fn weight(&self) -> Option<u8> {
        return self.config.weight();
    }

    async fn handle(&self, req: &str) -> Result<String> {
        // metrics
        let mut metrics_guard = QueryCounter::new(&self.name(), "WordCount");

        let req = Self::parse(req)
            .context(format!("Endpoint handle failed, endpoint name={}, addr={:?}", self.config.name(), self.config.get_socket_addr()))?;
        let mut client = self.counter_client().ok_or_else(|| anyhow!("handle request failed"))?;
        let resp = client.count(req).await.context("call count service failed")?;
        let resp = serde_json::to_string(resp.get_ref()).context("serialize response failed")?;

        metrics_guard.mark_success();
        Ok(resp)
    }

    async fn health_check(&self) {
        // metrics
        let mut metrics_guard = QueryCounter::new(&self.name(), "HealthCheck");

        let req = Request::new(HealthCheckRequest::default());
        let mut status = ServingStatus::NotServing as i32;
        if let Some(mut health_client) = self.health_client() {
            if let Ok(response) = health_client.check(req).await {
                metrics_guard.mark_success();
                status = response.into_inner().status;
            }
        }
        self.update_health_status(status);

        return;
    }

    fn health_report(&self) -> bool {
        return self.is_health.load(Ordering::SeqCst);
    }
}

#[cfg(test)]
mod test {
    use crate::endpoint::WordCountServer;

    #[test]
    fn test_parse() {
        let req = "{\"word\":\"world\", \"file_name\":\"text1.txt\"}";
        let req = WordCountServer::parse(req);
        assert!(req.is_ok());
        let req = req.unwrap().into_inner();
        assert_eq!(req.word, "world");
        assert_eq!(req.file_name, "text1.txt");
    }

    #[test]
    fn test_parse_invalid() {
        let req = "{\"foo\":\"bar\", \"file_name\":\"text1.txt\"}";
        let req = WordCountServer::parse(req);
        assert!(req.is_err());
    }
}
