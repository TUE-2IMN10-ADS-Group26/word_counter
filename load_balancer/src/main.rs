use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use prometheus::{Encoder, TextEncoder};
use tracing_appender::non_blocking::WorkerGuard;
use warp::Filter;

use crate::consts::{CONFIG_PATH_ENDPOINTS, CONFIG_PATH_LOAD_BALANCER, CONFIG_PATH_SERVER, ROUND_ROBIN, WEIGHTED_ROUND_ROBIN};
use crate::endpoint::{Endpoint, WordCountServer};
use crate::load_balancer::{LoadBalancer, LoadBalancerImpl};
use crate::model::endpoints_config::EndpointPoolConfig;
use crate::model::load_balancer_config::LBConfig;
use crate::model::server_config::ServerConfig;
use crate::server::LBServer;
use crate::strategy::round_robin::RoundRobin;
use crate::strategy::RouteStrategy;
use crate::strategy::weighted_round_robin::WeightedRoundRobin;

mod endpoint;
mod load_balancer;
mod strategy;
mod server;
mod consts;
mod metrics;

mod model {
    pub mod endpoints_config;
    pub mod server_config;

    pub mod load_balancer_config;
}

#[tokio::main]
async fn main() {
    // init ctrlc handler
    let running = Arc::new(AtomicBool::new(true));
    init_graceful_exit(running.clone());

    // init logger
    let _guard = init_logger();
    tracing::info!("logger initiated");

    // metrics data server
    let metrics_task = tokio::spawn(async {
        AppBuilder::start_metrics_server().await;
        tracing::info!("metrics server exit");
    });

    // load balance server
    let lb_task = tokio::spawn(async {
        let mut server = AppBuilder::build_lb_server().await.unwrap_or_else(|e| {
            panic!("server init failed with error: {:?}", e)
        });
        server.start(running).await;
        tracing::info!("load balance server exit");
    });

    let _ = tokio::join!(metrics_task, lb_task);
}

fn init_logger() -> WorkerGuard {
    let (non_blocking, _guard) = tracing_appender::non_blocking(
        tracing_appender::rolling::hourly("output/", "requests.log")
    );
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .init();
    _guard
}

fn init_graceful_exit(running: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        running.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
}

struct AppBuilder {}

impl AppBuilder {
    async fn build_lb_server() -> Result<LBServer> {
        let lb_config = LBConfig::load(Path::new(CONFIG_PATH_LOAD_BALANCER))?;
        let pool_config = EndpointPoolConfig::load(Path::new(CONFIG_PATH_ENDPOINTS), lb_config.strategy().as_str())?;

        let strategy = Self::strategy(&lb_config);
        let endpoints = Self::endpoints(pool_config).await;

        let lb = Self::load_balancer(endpoints, strategy);
        LBServer::build(Arc::new(lb)).await
    }

    pub async fn start_metrics_server() {
        let server_config = ServerConfig::load(Path::new(CONFIG_PATH_SERVER)).expect("load server config failed");
        let metrics = warp::path!("metrics").map(|| {
            let encoder = TextEncoder::new();
            let mut buffer = vec![];
            let _ = encoder.encode(&prometheus::gather(), &mut buffer).unwrap();
            warp::reply::with_header(buffer, "Context-Type", encoder.format_type())
        });
        warp::serve(metrics).run(server_config.get_metrics_addr()).await;
    }

    fn strategy(config: &LBConfig) -> Box<dyn RouteStrategy> {
        Self::create_strategy(config)
    }

    fn load_balancer(endpoints: Vec<Arc<Box<dyn Endpoint>>>, strategy: Box<dyn RouteStrategy>) -> Box<dyn LoadBalancer> {
        Box::new(LoadBalancerImpl::new(endpoints, strategy))
    }

    async fn endpoints(config: EndpointPoolConfig) -> Vec<Arc<Box<dyn Endpoint>>> {
        let mut endpoints = vec![];
        for config in config.endpoint_configs() {
            let mut endpoint = WordCountServer::new(config.clone());
            endpoint.build().await.unwrap_or_else(|err| {
                tracing::error!(?config, ?err, "build endpoint failed.")
            });
            let endpoint: Box<dyn Endpoint> = Box::new(endpoint);
            endpoints.push(Arc::new(endpoint))
        }
        endpoints
    }

    fn create_strategy(config: &LBConfig) -> Box<dyn RouteStrategy> {
        let strategy = config.strategy();

        tracing::info!(strategy, "strategy created");
        match strategy.as_str() {
            WEIGHTED_ROUND_ROBIN => Box::new(WeightedRoundRobin::new()),
            ROUND_ROBIN | _ => Box::new(RoundRobin::new(None)),
        }
    }
}