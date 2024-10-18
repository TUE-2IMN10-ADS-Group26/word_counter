use std::io::{Read, Write};
use std::net::TcpStream;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use async_std::task;
use clap::{Parser, Subcommand};
use fake::Fake;
use fake::faker::lorem::en::Word;
use indicatif::ProgressIterator;
use once_cell::sync::Lazy;
use rayon::ThreadPoolBuilder;
use tonic::Request;
use tonic::transport::{Channel, Uri};

use word_counter::counter_client::CounterClient;

use crate::word_counter::{WordCountRequest, WordCountResponse};

pub mod word_counter {
    include!("proto_gen/word_counter.rs");
}

#[derive(Parser, Clone)]
#[command(about = "count word in some text")]
struct CliParams {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    file_name: String,
    #[arg(long)]
    with_lb: bool,
}

#[derive(Subcommand, Clone)]
enum Commands {
    Count { word: String },
    Random { batch: usize },
}

#[derive(Clone)]
struct ClientContext {
    params: CliParams,
    client: Arc<Lazy<CounterClient<Channel>>>,
}

impl ClientContext {
    fn new(params: CliParams) -> Self {
        ClientContext {
            params,
            client: Arc::new(Lazy::new(|| {
                task::block_on(Self::init_client())
            })),
        }
    }

    async fn init_client() -> CounterClient<Channel> {
        let channel = Self::init_channel().await
            .context("init RPC client failed")
            .unwrap_or_else(|e| { panic!("{:#?}", e); });
        CounterClient::new(channel)
    }

    async fn init_channel() -> Result<Channel> {
        let uri: Uri = Uri::from_str("server1:50051").context("parse server Uri failed")?;
        let inner_endpoint = Channel::builder(uri)
            .connect_timeout(Duration::from_secs(5))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .timeout(Duration::from_secs(2));
        let channel = inner_endpoint.connect().await.context("RPC channel connect failed")?;
        Ok(channel)
    }

    fn get_client(&self) -> CounterClient<Channel> {
        self.client.deref().deref().clone()
    }

    fn try_get_query_word(&self) -> Option<String> {
        if let Commands::Count { word } = &self.params.command {
            return Some(word.clone());
        }
        None
    }

    fn try_get_batch_num(&self) -> Option<usize> {
        if let Commands::Random { batch } = &self.params.command {
            return Some(batch.clone());
        }
        None
    }
}

#[async_std::main]
async fn main() {
    let params = CliParams::parse();
    let mut client_ctx = ClientContext::new(params);
    exec(&mut client_ctx).await
}

async fn exec(client_ctx: &mut ClientContext) {
    match &client_ctx.params.command {
        Commands::Count { .. } => {
            exec_query(client_ctx).await
        }
        Commands::Random { .. } => {
            exec_random_query(client_ctx)
        }
    }
}

fn exec_random_query(client_ctx: &mut ClientContext) {
    let pool = ThreadPoolBuilder::new().build().expect("thread pool initialize failed");
    let exec_task = || {
        let mut client_ctx = client_ctx.clone();
        pool.spawn(|| {
            task::block_on(async move {
                let req = build_random_request(&client_ctx);
                let resp = call_count(&mut client_ctx, req.clone()).await;
                display(req, resp);
            });
            thread::sleep(Duration::from_millis(500));
        });
    };

    for _ in (0..client_ctx.try_get_batch_num().unwrap()).progress() {
        exec_task();
    }
}

async fn exec_query(client_ctx: &mut ClientContext) {
    let req = build_request(client_ctx);
    let resp = call_count(client_ctx, req.clone()).await;
    display(req, resp)
}

fn display(req: WordCountRequest, resp: Result<WordCountResponse>) {
    match resp {
        Ok(r) => {
            println!("✅ call [Count] success, word: {} occur {} times in {}", req.word, r.count, req.file_name);
        }
        Err(e) => {
            println!("❌ call [Count] failed, request={:?}, err={:?}", req, e);
        }
    }
}

async fn call_count(client_ctx: &mut ClientContext, req: WordCountRequest) -> Result<WordCountResponse> {
    if client_ctx.params.with_lb {
        count_with_lb(req)
    } else {
        count_without_lb(client_ctx, req).await
    }
}

fn build_request(client_ctx: &ClientContext) -> WordCountRequest {
    WordCountRequest {
        word: client_ctx.try_get_query_word().unwrap(), // should not panic
        file_name: client_ctx.params.file_name.clone(),
    }
}

fn build_random_request(client_ctx: &ClientContext) -> WordCountRequest {
    WordCountRequest {
        word: Word().fake(),
        file_name: client_ctx.params.file_name.clone(),
    }
}

// RPC
async fn count_without_lb(client_ctx: &mut ClientContext, req: WordCountRequest) -> Result<WordCountResponse> {
    let resp = client_ctx.get_client().count(Request::new(req)).await.context("call RPC method: count failed")?;
    Ok(resp.into_inner())
}

// TCP
fn count_with_lb(req: WordCountRequest) -> Result<WordCountResponse> {
    let mut stream = TcpStream::connect("load_balancer:8080").context("init TCP stream failed")?;
    stream.set_write_timeout(Some(Duration::from_millis(1000))).context("set TCP stream write timeout failed")?;
    stream.set_read_timeout(Some(Duration::from_millis(1000))).context("set TCP stream read timeout failed")?;
    let message = serde_json::to_vec(&req).context("TCP request serialize failed")?;
    stream.write_all(&message).context("send TCP message failed")?;
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer).context("read TCP stream failed")?;
    let response: WordCountResponse = serde_json::from_str(&buffer).context("TCP response deserialize failed")?;
    Ok(response)
}

