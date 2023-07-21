# Mongosync

Listens to changes in a MongoDB database and broadcasts them over many destinations.

## Sinks

- [ ] Stdout
- [ ] File
- [ ] MongoDB
- [ ] Websocket
- [ ] Elasticsearch

## Configuration file

```yaml
source:
  connection_uri: <string>
  database: <string>
sinks:
  - type: stdout
    pretty: true
  - type: file
    path: ./changes.log
  - type: mongodb
    connection_uri: <>
```

## Development

### Prerequisites

- Install [Docker](https://www.docker.com/get-started/)
- Install [Rust](https://www.rust-lang.org/tools/install)

### Run mongosync

```sh
# Start MongoDB
docker compose up -d --remove-orphans
# Start mongosync
cargo run -- --config ./examples/mongosync.yaml
```
