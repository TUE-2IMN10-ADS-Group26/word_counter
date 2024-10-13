use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result};
use async_std::io::{ReadExt, WriteExt};
use async_std::net::{TcpListener, TcpStream};
use async_std::task::spawn;
use futures::lock::Mutex;
use futures::StreamExt;

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

    pub async fn start(&mut self, exit: Arc<AtomicBool>) {
        if self.config.fault_tolerance() {
            self.load_balancer.health_maintain();
        }
        self.listener.incoming().for_each_concurrent(
            None, |tcp_stream| async {
                // graceful exit while process is killed
                if Arc::clone(&exit).load(Ordering::SeqCst) {
                    self.close();
                    return;
                }
                // handle client tcp request
                let load_balancer = Arc::clone(&self.load_balancer);
                match tcp_stream {
                    Err(e) => {
                        tracing::error!(?e, "connection failed");
                    }
                    Ok(stream) => {
                        spawn(Self::handle_connection(Arc::new(Mutex::new(stream)), load_balancer));
                    }
                }
            },
        ).await;
    }

    pub fn close(&self) {
        if self.config.fault_tolerance() {
            self.load_balancer.stop_health_maintain();
        }
    }
    async fn read_request(stream: &mut TcpStream) -> Result<String> {
        let mut req = String::new();
        stream.read_to_string(&mut req).await.context("read tcp request failed")?;
        Ok(req)
    }

    async fn send_response(stream: &mut TcpStream, resp: String) {
        stream.write_all(resp.as_bytes()).await.unwrap_or_else(|e| {
            tracing::error!(?e, "send response failed")
        });
        stream.flush().await.unwrap();
    }

    async fn handle_connection(stream: Arc<Mutex<TcpStream>>, lb: Arc<Box<dyn LoadBalancer>>) {
        let mut stream = stream.lock().await;
        let req = Self::read_request(&mut *stream).await;
        match req {
            Ok(req) => {
                let resp = lb.handle(req).await
                    .unwrap_or_else(|e| {
                        tracing::error!(?e, "Load Balancer handle failed");
                        let failed_resp = WordCountResponse::failed_resp();
                        serde_json::to_string(&failed_resp).unwrap_or_default()
                    });
                Self::send_response(&mut *stream, resp).await;
            }
            Err(e) => {
                tracing::error!(?e, "failed to read request");
                let failed_resp = WordCountResponse::failed_resp();
                Self::send_response(&mut *stream, serde_json::to_string(&failed_resp).unwrap_or_default()).await;
            }
        }
    }
}