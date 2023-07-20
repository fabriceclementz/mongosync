use anyhow::{Context, Result};
use log::debug;
use log::info;
use mongodb::options::ChangeStreamOptions;
use mongodb::options::FullDocumentType;
use mongodb::{
    bson::Document,
    change_stream::event::OperationType::{Delete, Insert, Update},
    options::ClientOptions,
    Client,
};
use serde::Serialize;

use crate::config::SinksConfig::MongoDB;
use crate::config::SinksConfig::Stdout;
use crate::config::{Config, SourceConfig};

pub async fn run_replication(config: &Config) -> Result<()> {
    info!("Starting replication...");
    let client = get_source_client(&config.source)
        .await
        .context("Failed to get a handle to the source database")?;

    let db = client.database(&config.source.database);
    let options = ChangeStreamOptions::builder()
        .full_document(Some(FullDocumentType::UpdateLookup))
        .build();
    let mut change_stream = db
        .watch(None, options)
        .await
        .context("Failed to start change stream")?;

    let mut _resume_token = None;
    while change_stream.is_alive() {
        if let Some(event) = change_stream.next_if_any().await? {
            debug!("changestream event: {:?}", event);

            let msg = match event.operation_type {
                Insert => {
                    let ns = event.ns.unwrap();
                    let db = ns.db;
                    let coll = ns.coll.unwrap();
                    Some(Message {
                        event: Event::Insert,
                        db,
                        coll,
                        id: event.document_key,
                        data: event.full_document,
                    })
                }
                Update => {
                    let ns = event.ns.unwrap();
                    let db = ns.db;
                    let coll = ns.coll.unwrap();
                    Some(Message {
                        event: Event::Update,
                        db,
                        coll,
                        id: event.document_key,
                        data: event.full_document,
                    })
                }
                Delete => {
                    let ns = event.ns.unwrap();
                    let db = ns.db;
                    let coll = ns.coll.unwrap();
                    Some(Message {
                        event: Event::Delete,
                        db,
                        coll,
                        id: event.document_key,
                        data: event.full_document,
                    })
                }
                _ => None,
            };

            match msg {
                Some(msg) => {
                    for sink in &config.sinks {
                        match sink {
                            Stdout(options) => {
                                let json = match options.pretty {
                                    Some(pretty) if pretty == true => {
                                        serde_json::to_string_pretty(&msg)
                                            .context("Cannot pretty serialize message to JSON")?
                                    }
                                    _ => serde_json::to_string(&msg)
                                        .context("Cannot serialize message to JSON")?,
                                };
                                println!("{}", json);
                            }
                            MongoDB(_) => {}
                        }
                    }
                }
                None => {}
            }
        }

        _resume_token = change_stream.resume_token();
    }

    Ok(())
}

async fn get_source_client(source: &SourceConfig) -> Result<Client> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(&source.connection_uri).await?;

    // Manually set an option.
    client_options.app_name = Some(env!("CARGO_CRATE_NAME").to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    Ok(client)
}

#[derive(Debug, Serialize)]
pub struct Message<T>
where
    T: Serialize,
{
    pub event: Event,
    pub db: String,
    pub coll: String,
    pub id: Option<Document>,
    pub data: Option<T>,
}

#[derive(Debug, Serialize)]
pub enum Event {
    Insert,
    Update,
    Delete,
}
