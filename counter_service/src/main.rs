use std::env;

use deadpool_redis::{Config, Pool, Runtime};
use tonic::transport::Server;
use tonic::transport::server::Router;
use tracing_appender::non_blocking::WorkerGuard;

use crate::counter_server::CounterService;
use crate::counter_server::word_counter::counter_server::CounterServer;

mod counter_server;
mod read_counter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init logger
    let _guard = init_logger();
    tracing::info!("logger initiated");

    // init redis pool
    let pool = init_redis_conn_pool();
    tracing::info!("redis poll initiated");

    // init server
    let addr = "127.0.0.1:50051".parse().unwrap();
    let server = init_server(pool).await;
    tracing::info!("CounterServer listening on {}", addr);
    server.serve(addr)
        .await?;
    Ok(())
}

async fn init_server(pool: Pool) -> Router {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<CounterServer<CounterService>>()
        .await;
    let counter_service = CounterService::new(pool);
    Server::builder()
        .add_service(health_service)
        .add_service(CounterServer::new(counter_service))
}

fn init_logger() -> WorkerGuard {
    let (non_blocking, _guard) = tracing_appender::non_blocking(
        tracing_appender::rolling::hourly("output/", "counter.log")
    );
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .init();
    _guard
}

fn init_redis_conn_pool() -> Pool {
    let cfg = Config::from_url(env::var("REDIS__URL").expect("init redis failed"));
    cfg.create_pool(Some(Runtime::Tokio1)).unwrap()
}