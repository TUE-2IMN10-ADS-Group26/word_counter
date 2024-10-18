use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::spawn;

use crate::consts::CONFIG_PATH_SERVER;
use crate::endpoint::word_counter::WordCountResponse;
use crate::load_balancer::LoadBalancer;
use crate::model::server_config::ServerConfig;

pub struct LBServer
{
    listener: TcpListener,
    load_balancer: Arc<Box<dyn LoadBalancer>>,
    config: ServerConfig,
}

impl LBServer
{
    pub async fn build(load_balancer: Arc<Box<dyn LoadBalancer>>) -> Result<Self> {
        let config = Self::load_config().with_context(|| "server started failed")?;
        let listener = Self::init_listener(&config).await.unwrap();

        Ok(LBServer {
            listener,
            load_balancer,
            config,
        })
    }

    async fn init_listener(config: &ServerConfig) -> Result<TcpListener> {
        let socket_addr = Self::get_socket_addr(config);
        TcpListener::bind(socket_addr).await
            .with_context(|| format!("bind socket {:#?} failed", socket_addr))
    }

    fn get_socket_addr(config: &ServerConfig) -> SocketAddr {
        config.get_socket_addr()
    }

    fn load_config() -> Result<ServerConfig> {
        ServerConfig::load(Path::new(CONFIG_PATH_SERVER))
            .with_context(|| "load server config failed")
    }

    pub async fn start(&mut self, running: Arc<AtomicBool>) {
        if self.config.fault_tolerance() {
            tracing::info!("[LoadBalancer] health maintain process started");
            self.load_balancer.health_maintain();
        }
        tracing::info!("[LoadBalancer] server started, serving at {:?}", self.config.get_socket_addr());
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    tracing::info!("[Load Balancer] accept new tcp connection from addr={:?}", addr);
                    let load_balancer = Arc::clone(&self.load_balancer);
                    spawn(Self::handle_connection(stream, load_balancer));
                }
                Err(err) => { tracing::error!(?err, "connection failed"); }
            }
            // graceful exit while process is killed
            if !Arc::clone(&running).load(Ordering::SeqCst) {
                tracing::info!("[LoadBalancer] gracefully exit");
                self.close();
                return;
            }
        }
    }

    pub fn close(&self) {
        if self.config.fault_tolerance() {
            self.load_balancer.stop_health_maintain();
        }
    }
    async fn read_request(stream: &mut TcpStream) -> Result<String> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.context("failed to read message length")?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buffer = vec![0; len];
        stream.read_exact(&mut buffer).await.context("failed to read message body")?;
        Ok(String::from_utf8(buffer)?)
    }

    async fn send_response(stream: &mut TcpStream, resp: &str) {
        if let Err(e) = stream.write_u32(resp.len() as u32).await {
            tracing::error!("fail to write response length, err={:?}", e);
            return;
        }

        if let Err(e) = stream.write_all(resp.as_bytes()).await {
            tracing::error!("fail to write response length, err={:?}", e);
            return;
        }
        stream.flush().await.unwrap_or_else(|e| {
            tracing::error!(?e, "TCP stream write response failed")
        });
    }

    async fn handle_connection(mut stream: TcpStream, lb: Arc<Box<dyn LoadBalancer>>) {
        let req = Self::read_request(&mut stream).await;
        let resp = match req {
            Ok(req) => lb.handle(req).await,
            Err(e) => {
                Err(e.context("[Load Balancer] failed to read request"))
            }
        };

        if let Err(e) = &resp {
            tracing::error!(?e, "[Load Balancer] request handle failed");
        }
        let prompt = if resp.is_ok() { "success ✅" } else { "failed ❌" };
        let response = &resp.unwrap_or_else(|_| {
            let failed_resp = WordCountResponse::failed_resp();
            serde_json::to_string(&failed_resp).unwrap_or_default()
        });
        tracing::info!("[Load Balancer] request {}, response = {}", prompt, &response);
        Self::send_response(&mut stream, response).await;
    }
}