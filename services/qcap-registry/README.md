# Q-Cap Registry (MVP)

A minimal registry that serves a health check, an index of `.qcap` files scanned from a seed directory, and the `.qcap` artifacts themselves.

## Endpoints

- `GET /health` — `{"status":"ok"}`
- `GET /index.json` — JSON array of capsules `{ name, path, size, content_type }`
- `GET /artifacts/<name>` — serves the `.qcap` file directly from the seed dir

## Seed Directory

By default, the registry scans `services/qcap-registry/seed`. You can override by setting `QCAP_REGISTRY_SEED`.

## Seed helper (optional)

Seed directory defaults to `services/qcap-registry/seed` or set `QCAP_REGISTRY_SEED`.

Quick start:

```sh
# Seed with demo capsules
scripts/seed-registry.sh

# Run registry
go run services/qcap-registry/main.go

# Smoke test endpoints (optional)
scripts/smoke-registry.sh
```
cargo run -p qcap-cli -- pack /tmp/qcap-seed-b --out services/qcap-registry/seed/beta.qcap --key /tmp/ed25519.seed.hex
