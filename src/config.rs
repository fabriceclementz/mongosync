use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub source: SourceConfig,
    pub destinations: Vec<DestinationConfig>,
}

#[derive(Debug, Deserialize)]
pub struct SourceConfig {
    pub connection_uri: String,
    pub database: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum DestinationConfig {
    #[serde(rename = "stdout")]
    Stdout(StdoutConfig),
    #[serde(rename = "mongodb")]
    MongoDB(MongoDBConfig),
}

#[derive(Debug, Deserialize)]
pub struct StdoutConfig {}

#[derive(Debug, Deserialize)]
pub struct MongoDBConfig {
    pub connection_uri: String,
}
