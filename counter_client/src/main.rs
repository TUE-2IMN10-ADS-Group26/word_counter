use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use futures::future::join_all;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rand::seq::IndexedRandom;
use rand::thread_rng;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, OnceCell};
use tokio::time::Instant;
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
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Count occurrences of a word in a file
    Count {
        #[arg(short, long, help = "query word")]
        word: String,
        #[arg(short, long, help = "target file name")]
        file_name: String,
        #[arg(long, default_value_t = false, help = "if use load balancer")]
        with_lb: bool,
    },
    /// Count random words in a file
    Random {
        #[arg(short, long, help = "batch query num")]
        batch: usize,
        #[arg(short, long, help = "target file name")]
        file_name: String,
        #[arg(short, long, help = "interval(ms) between requests in each thread")]
        interval: u64,
        #[arg(long, default_value_t = false, help = "if use load balancer")]
        with_lb: bool,
    },
}

#[derive(Clone)]
struct ClientContext {
    params: CliParams,
    word_list: OnceCell<Vec<String>>,
    client: Arc<OnceCell<CounterClient<Channel>>>,
}

impl ClientContext {
    fn new(params: CliParams) -> Self {
        ClientContext {
            params,
            word_list: OnceCell::new(),
            client: Arc::new(OnceCell::new()),
        }
    }

    async fn init_client() -> CounterClient<Channel> {
        let channel = Self::init_channel().await
            .context("init RPC client failed")
            .unwrap_or_else(|e| { panic!("{:#?}", e); });
        CounterClient::new(channel)
    }

    async fn init_channel() -> Result<Channel> {
        let uri: Uri = Uri::from_str("http://server1:50051").context("parse server Uri failed")?;
        let inner_endpoint = Channel::builder(uri)
            .connect_timeout(Duration::from_secs(5))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .timeout(Duration::from_secs(10));
        let channel = inner_endpoint.connect().await.context("RPC channel connect failed")?;
        Ok(channel)
    }

    async fn get_client(&self) -> CounterClient<Channel> {
        self.client.deref().get_or_init(|| async { Self::init_client().await }).await.clone()
    }

    async fn get_random_word(&self) -> String {
        let words: Vec<String> = self.word_list.get_or_init(|| async {
            let file = File::open("src/orchard-street-medium.txt").await.context("word list file not found").unwrap();
            let reader = BufReader::new(file);
            let mut lines = reader.lines();
            let mut words = Vec::new();
            while let Ok(Some(line)) = lines.next_line().await {
                words.push(line);
            }
            words
        }).await.clone();
        if let Some(random_word) = words.choose(&mut thread_rng()) { return String::from(random_word); };
        panic!("empty word list");
    }

    fn try_get_query_word(&self) -> Option<String> {
        if let Commands::Count { word, .. } = &self.params.command {
            return Some(word.clone());
        }
        None
    }

    fn try_get_batch_num(&self) -> Option<usize> {
        if let Commands::Random { batch, .. } = &self.params.command {
            return Some(batch.clone());
        }
        None
    }

    fn try_get_interval(&self) -> Option<u64> {
        if let Commands::Random { interval, .. } = &self.params.command {
            return Some(interval.clone());
        }
        None
    }

    fn get_file_name(&self) -> String {
        match &self.params.command {
            Commands::Count { file_name, .. } => { file_name.clone() }
            Commands::Random { file_name, .. } => { file_name.clone() }
        }
    }

    fn with_lb(&self) -> bool {
        match &self.params.command {
            Commands::Count { with_lb, .. } => { with_lb.clone() }
            Commands::Random { with_lb, .. } => { with_lb.clone() }
        }
    }
}

#[tokio::main]
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
            exec_random_query(client_ctx).await
        }
    }
}

async fn exec_random_query(client_ctx: &mut ClientContext) {
    let mut handles = Vec::new();
    let total = client_ctx.try_get_batch_num().unwrap();
    let interval_ms = client_ctx.try_get_interval().unwrap();
    let bar = build_progress_bar(total);
    let batch_start = Instant::now();
    let total_process_latency_ms: Arc<AtomicU64> = Arc::new(AtomicU64::default());
    let idx = Arc::new(Mutex::new(0));

    for _ in 0..total {
        let mut client_ctx = client_ctx.clone();
        let bar = bar.clone();
        let total_process_latency_ms = Arc::clone(&total_process_latency_ms);
        let idx = Arc::clone(&idx);
        let handle = tokio::spawn(async move {
            let req = build_random_request(&client_ctx).await;
            let start = Instant::now();
            let resp = call_count(&mut client_ctx, req.clone()).await;
            let latency = start.elapsed();
            total_process_latency_ms.fetch_add(latency.as_millis() as u64, Ordering::SeqCst);
            let mut idx = idx.lock().await;
            *idx += 1;
            bar.set_prefix(format!("[{}/{}]", idx.to_string().blue(), total));
            bar.inc(1);
            bar.set_message(format!("{}", state_message(req, resp, latency)));
        });
        handles.push(handle);
        thread::sleep(Duration::from_millis(interval_ms));
    }
    join_all(handles).await;

    let batch_duration = batch_start.elapsed();
    let average = Duration::from_millis(total_process_latency_ms.load(Ordering::SeqCst) / total as u64);

    bar.set_prefix(format!("[{}/{}]", total.to_string().blue(), total));
    bar.finish_with_message(format!("🎉 All jobs done!\n\n⏳ Total time: {}\n⏳ Average time per query: {}",
                                    HumanDuration(batch_duration).to_string().blue(),
                                    fmt_latency(average).blue()));
}

fn build_progress_bar(total: usize) -> ProgressBar {
    let progress = MultiProgress::new();
    let style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.green} {wide_msg}").unwrap();
    let bar = progress.add(ProgressBar::new(total as u64));
    bar.set_style(style);
    bar
}

async fn exec_query(client_ctx: &mut ClientContext) {
    let req = build_request(client_ctx);
    let start = Instant::now();
    let resp = call_count(client_ctx, req.clone()).await;
    println!("{}", state_message(req, resp, start.elapsed()));
}

fn state_message(req: WordCountRequest, resp: Result<WordCountResponse>, latency: Duration) -> String {
    let latency = fmt_latency(latency);
    match resp {
        Ok(r) => {
            format!("✅ {} in {}: word '{}' occur {} times in {}.", "succeed".green(), latency.green(), req.word, r.count, req.file_name)
        }
        Err(e) => {
            format!("❌ {}, word: {}, file: {}, err={:?}", "failed".red(), req.word, req.file_name, e)
        }
    }
}

fn fmt_latency(latency: Duration) -> String {
    let micros = latency.as_micros();
    format!("{}.{} ms", micros / 1000, micros % 1000)
}

async fn call_count(client_ctx: &mut ClientContext, req: WordCountRequest) -> Result<WordCountResponse> {
    if client_ctx.with_lb() {
        count_with_lb(req).await
    } else {
        count_without_lb(client_ctx, req).await
    }
}

fn build_request(client_ctx: &ClientContext) -> WordCountRequest {
    WordCountRequest {
        word: client_ctx.try_get_query_word().unwrap(), // should not panic
        file_name: client_ctx.get_file_name().clone(),
    }
}

async fn build_random_request(client_ctx: &ClientContext) -> WordCountRequest {
    WordCountRequest {
        word: client_ctx.get_random_word().await,
        file_name: client_ctx.get_file_name().clone(),
    }
}

// RPC
async fn count_without_lb(client_ctx: &mut ClientContext, req: WordCountRequest) -> Result<WordCountResponse> {
    let resp = client_ctx.get_client().await.count(Request::new(req)).await.context("call RPC method: count failed")?;
    Ok(resp.into_inner())
}

// TCP
async fn count_with_lb(req: WordCountRequest) -> Result<WordCountResponse> {
    let mut stream = TcpStream::connect("load_balancer:8080").await.context("init TCP stream failed")?;

    let message = serde_json::to_string(&req).context("TCP request serialize failed")?;
    stream.write_u32(message.len() as u32).await.context("TCP stream fail to write message length")?;
    stream.write_all(message.as_bytes()).await.context("TCP stream write message failed")?;

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await.context("failed to read response length")?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buffer = vec![0; len];
    stream.read_exact(&mut buffer).await.context("failed to read response body")?;
    let response = String::from_utf8(buffer).context("invalid response")?;

    let response: WordCountResponse = serde_json::from_str(&response).with_context(|| {
        format!("TCP response deserialize failed, resp={response}")
    })?;
    Ok(response)
}

