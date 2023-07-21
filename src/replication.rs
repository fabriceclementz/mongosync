use anyhow::{Context, Result};
use log::{debug, info};
use mongodb::{
    bson::Document,
    change_stream::event::OperationType::{Delete, Insert, Update},
    options::{ChangeStreamOptions, ClientOptions, FullDocumentType},
    Client,
};
use serde::Serialize;
use tokio::{fs::OpenOptions, io::AsyncWriteExt, sync::mpsc};

use crate::config::SinksConfig::{File, MongoDB};
use crate::config::StdoutConfig;
use crate::config::{Config, SourceConfig};
use crate::config::{FileConfig, SinksConfig::Stdout};

pub async fn run(config: &Config) -> Result<()> {
    info!("Starting replication...");

    for sink in &config.sinks {
        match sink {
            Stdout(_) => info!("stdout sink enabled"),
            MongoDB(_) => info!("mongodb sink enabled"),
            File(_) => info!("file sink enabled"),
        };
    }

    let (tx, mut rx) = mpsc::channel(1000);

    tokio::spawn(async move {
        info!("starting receiver");
        loop {
            while let Some(msg) = rx.recv().await {
                debug!("received msg {:?}", msg);
            }
        }
    });

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
            debug!("changestream event detected: {:?}", event);

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

            if let Some(msg) = msg {
                tx.send(msg.clone()).await.expect("cannot send message");
                for sink in &config.sinks {
                    let sink = sink.clone();
                    match sink {
                        Stdout(options) => {
                            let msg = msg.clone();
                            tokio::spawn(async move {
                                handle_stdout_sink(&options, &msg);
                            });
                        }
                        MongoDB(_) => {
                            let msg = msg.clone();
                            tokio::spawn(async move {
                                handle_mongo_sink(&msg);
                            });
                        }
                        File(options) => {
                            let msg = msg.clone();
                            tokio::spawn(async move {
                                handle_file_sink(&options, &msg).await;
                            });
                        }
                    };
                }
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub enum Event {
    Insert,
    Update,
    Delete,
}

pub fn handle_stdout_sink(options: &StdoutConfig, msg: &Message<Document>) {
    let json = match options.pretty {
        Some(pretty) if pretty => {
            serde_json::to_string_pretty(&msg).expect("Cannot pretty serialize message to JSON")
        }
        _ => serde_json::to_string(&msg).expect("Cannot serialize message to JSON"),
    };
    println!("{}", json);
}

pub fn handle_mongo_sink(msg: &Message<Document>) {
    info!("sync in mongo destination: {:?}", msg);
}

pub async fn handle_file_sink(options: &FileConfig, msg: &Message<Document>) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&options.path)
        .await
        .expect("Cannot create sink file");

    let json = serde_json::to_vec(&msg).expect("Cannot serialize message to JSON");
    file.write_all(&json)
        .await
        .expect("cannot write message in file");

    info!("sync in file destination: {:?}", msg);
}
