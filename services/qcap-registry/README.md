# Q-Cap Registry (MVP)

A minimal registry that stores `.qcap` artifacts on disk, persists an `index.json`, and can require bearer-token auth for publishing.

## Endpoints

- `GET /health` - JSON status, including whether publish auth is enabled
- `GET /index.json` - JSON array of capsules
- `GET /index` - HTML index listing
- `POST /artifacts` - publish a `.qcap`
- `GET /artifacts/<name>` - download a `.qcap`

## Configuration

- `QCAP_REGISTRY_STORE` - artifact directory
- `QCAP_REGISTRY_SEED` - backward-compatible alias for `QCAP_REGISTRY_STORE`
- `QCAP_REGISTRY_INDEX` - index file path, defaults to `<store>/index.json`
- `QCAP_REGISTRY_TOKEN` - when set, `POST /artifacts` requires `Authorization: Bearer <token>`

## Quick Start

```sh
# Seed with demo capsules
scripts/seed-registry.sh

# Run registry with publish auth
QCAP_REGISTRY_TOKEN=demo-token go run services/qcap-registry/main.go

# Publish with the CLI
cargo run -p qcap-cli -- publish target/qcap-demo/demo.qcap \
  --registry http://127.0.0.1:8080 \
  --token demo-token

# Smoke test endpoints
scripts/smoke-registry.sh
```
