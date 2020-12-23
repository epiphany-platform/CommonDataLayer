use anyhow::Context;
use serde_json::Value;
use std::path::PathBuf;

pub fn read_json(file: Option<PathBuf>) -> anyhow::Result<Value> {
    if let Some(file_name) = file {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(file_name)
            .context("Couldn't open file")?;
        serde_json::from_reader(file).context("File did not contain valid JSON")
    } else {
        serde_json::from_reader(std::io::stdin()).context("File did not contain valid JSON")
    }
}
