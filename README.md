# Mongosync

Sync your MongoDB collections in realtime using MongoDB changes streams.

```yaml
source:
  connection_uri: <string>
  database: <string>
sinks:
  - type: stdout
    pretty: true
```

## Development

```
docker compose up -d --remove-orphans
cargo run -- --config ./examples/mongosync.yaml
```
