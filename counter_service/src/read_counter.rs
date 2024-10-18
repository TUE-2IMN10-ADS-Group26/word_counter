use std::path::Path;

use anyhow::{Context, Result};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Default)]
pub struct ReadCounter {}

impl ReadCounter {
    pub(crate) async fn count(word: &str, file_path: &Path) -> Result<i64> {
        let file = File::open(file_path).await.context(format!("fail to open file: {:?}", file_path))?;
        let reader = BufReader::new(file);

        let mut count: i64 = 0;
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await.context("some error occur while reading file.")? {
            count += line.matches(word).count() as i64;
        }

        Ok(count)
    }
}