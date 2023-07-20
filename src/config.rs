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

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SinksConfig {
    #[serde(rename = "stdout")]
    Stdout(StdoutConfig),
    #[serde(rename = "mongodb")]
    MongoDB(MongoDBConfig),
}

#[derive(Debug, Deserialize)]
pub struct StdoutConfig {
    pub pretty: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MongoDBConfig {
    pub connection_uri: String,
}
