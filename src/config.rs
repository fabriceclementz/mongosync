use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub source: SourceConfig,
    pub sinks: Vec<SinksConfig>,
}

#[derive(Debug, Deserialize)]
pub struct SourceConfig {
    pub connection_uri: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum SinksConfig {
    #[serde(rename = "stdout")]
    Stdout(StdoutConfig),
    #[serde(rename = "file")]
    File(FileConfig),
    #[serde(rename = "mongodb")]
    MongoDB(MongoDBConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct StdoutConfig {
    pub pretty: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileConfig {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MongoDBConfig {
    pub connection_uri: String,
}
