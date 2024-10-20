use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context};
use anyhow::Result;
use deadpool_redis::{Connection, Pool};
use moka::future::Cache;
use redis::{cmd, RedisResult};
use tokio::time::Instant;
use tonic::{async_trait, Code, Request, Response, Status};

use word_counter::{WordCountRequest, WordCountResponse};

use crate::counter_server::word_counter::counter_server::Counter;
use crate::read_counter::ReadCounter;

pub mod word_counter {
    include!("proto_gen/word_counter.rs");
}

const FAILED: i64 = -1;
pub struct CounterService {
    redis_conn_pool: Pool,
    cache: Cache<String, i64>,
}

impl CounterService {
    pub fn new(pool: Pool) -> Self {
        CounterService {
            redis_conn_pool: pool,
            cache: Cache::builder().max_capacity(32 * 1024 * 1024).build(), // with maximum 32mb
        }
    }

    async fn count_from_file(&self, word: &str, file_path: &Path) -> i64 {
        ReadCounter::count(word, file_path).await.unwrap_or_else(
            |e| {
                tracing::error!("ReadCounter count failed, err={:?}", e);
                FAILED
            }
        )
    }

    async fn get_from_cache(&self, key: &str) -> i64 {
        let lc_value = self.get_from_local_cache(key).await;
        if lc_value >= 0 {
            tracing::info!("from local cache: [key:{}. value:{}]", key, lc_value);
        }
        lc_value
    }

    async fn get_from_local_cache(&self, key: &str) -> i64 {
        self.cache.get_with(key.to_string(), async {
            let redis_value = self.get_from_redis(key).await;
            if redis_value > 0 {
                tracing::info!("from redis: [key:{}. value:{}]", key, redis_value);
            }
            redis_value
        }).await
    }

    async fn get_from_redis(&self, key: &str) -> i64 {
        let conn = self.get_redis_conn().await;
        if conn.is_none() { return FAILED; }
        let value: RedisResult<Option<i64>> = cmd("GET").arg(&[key]).query_async(&mut conn.unwrap()).await;
        match value {
            Ok(Some(v)) => {
                v
            }
            Ok(None) => {
                tracing::info!("key: {} not exist in redis", key);
                FAILED
            }
            Err(e) => {
                tracing::error!("get from redis failed, err={:?}", e);
                FAILED
            }
        }
    }

    async fn set_cache(&self, key: &str, value: i64) {
        self.set_local_cache(key, value).await;
        self.set_redis(key, value).await;
    }

    async fn set_local_cache(&self, key: &str, value: i64) {
        self.cache.insert(String::from(key), value).await;
    }
    async fn set_redis(&self, key: &str, value: i64) {
        let expiration_secs = if value == 0 { 30 } else { 300 };
        if let Some(mut conn) = self.get_redis_conn().await {
            cmd("SET")
                .arg(key)
                .arg(value)
                .arg("EX")
                .arg(expiration_secs)
                .query_async::<()>(&mut conn)
                .await
                .unwrap_or_else(
                    |e| {
                        tracing::error!("set redis failed, err={:?}", e);
                    }
                );
            return;
        }
        tracing::error!("set redis failed: get redis conn failed.");
    }

    async fn get_redis_conn(&self) -> Option<Connection> {
        let conn = self.redis_conn_pool.get().await;
        if conn.is_err() {
            tracing::error!("get redis connection from pool failed, err={:?}", conn.err());
            return None;
        }
        Some(conn.unwrap())
    }

    fn key(file_name: &str, word: &str) -> String {
        let file_name = Path::new(file_name).file_stem().unwrap().to_str().unwrap();
        format!("{}:{}", file_name, word)
    }

    fn fmt_latency(latency: Duration) -> String {
        let micros = latency.as_micros();
        format!("{}.{} ms", micros / 1000, micros % 1000)
    }
}

#[async_trait]
impl Counter for CounterService {
    async fn count(&self, request: Request<WordCountRequest>) -> std::result::Result<Response<WordCountResponse>, Status> {
        let start = Instant::now();
        let req = request.into_inner();
        tracing::info!("request received: {:#?}", req);
        if let Err(e) = req.check_params().context("request failed with invalid params") {
            return Err(Status::new(Code::FailedPrecondition, format!("{:?}", e)));
        }
        let key = Self::key(&req.file_name, &req.word);
        let mut value = self.get_from_cache(&key).await;
        if value == FAILED {
            tracing::info!("cache missed, key: {}", key);
            value = self.count_from_file(&req.word, &req.get_file_path()).await;
            tracing::info!("count from file, [key: {}, value: {}]", key, value);
            self.set_cache(&key, value).await
        };
        let end = start.elapsed();
        tracing::info!("handle latency: {} for key: {}", Self::fmt_latency(end), key);
        Ok(Response::new(WordCountResponse {
            count: value,
            status_code: 0,
            status_message: "ok".to_string(),
            log_id: "".to_string(),
        }))
    }
}

impl WordCountRequest {
    pub fn get_file_path(&self) -> PathBuf {
        PathBuf::from(&format!("{}/{}", env::var("TEXT_PATH").unwrap_or("../texts".to_string()), self.file_name))
    }

    pub fn get_file_name(&self) -> Option<&OsStr> {
        Path::new(&self.file_name).file_stem()
    }

    pub fn check_params(&self) -> Result<()> {
        if String::from(&self.word).is_empty() {
            return Err(anyhow!("invalid request: empty query word"));
        }
        if String::from(&self.file_name).is_empty() {
            return Err(anyhow!("invalid request: empty file name"));
        }
        if self.get_file_name().is_none() || self.get_file_name().unwrap().to_str().is_none() {
            return Err(anyhow!("invalid request: invalid file name: {}", self.file_name));
        }
        if !self.get_file_path().exists() {
            return Err(anyhow!("invalid request: file not exist: {}", self.file_name));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::counter_server::CounterService;

    #[test]
    fn test_key() {
        assert_eq!("Titanic:rose", CounterService::key("Titanic.txt", "rose"));
        assert_eq!("Titanic:rose", CounterService::key("Titanic", "rose"));
    }
}