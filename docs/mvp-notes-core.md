# MVP Notes: Existing Core & CLI (Task 1)

This notes document summarizes what exists in the repository that we can reuse for the Q-Cap MVP.

## Observed Structure

- `core/qcap-core/` — Rust library with initial crypto helper(s)
- `core/qcap-cli/` — Rust CLI using `clap` with a demo `hash` subcommand
- `services/qcap-registry/` — Go health-check service (out of scope for MVP packing/verify)
- `sdks/ts/` — TypeScript SDK stub (not used for MVP CLI)

## qcap-core (Rust library)

File: `core/qcap-core/src/lib.rs`

- Crates used:
  - `blake3` — hashing
  - `thiserror` — error type derivation
- Public API currently available:
  - `pub enum QcapError` — basic error wrapper
  - `pub fn merkle_root_demo(bytes: &[u8]) -> String` — computes a BLAKE3 hash over input bytes and returns "blake3:<hex>".
- Notes:
  - There are no modules yet for manifest, archive, merkle tree construction over files, signatures, or capabilities.
  - The demo function shows canonical digest formatting we can reuse: prefix `blake3:` and hex encoding.

## qcap-cli (Rust CLI)

File: `core/qcap-cli/src/main.rs`

- Uses `clap` for argument parsing.
- Subcommands:
  - `hash <input>` — computes BLAKE3 using `qcap_core::merkle_root_demo` over the provided string.
- Notes:
  - CLI wiring is in place. We can extend with additional subcommands: `pack`, `verify`, and `inspect`.

## Reuse Plan for MVP

- Hashing:
  - Reuse `blake3` crate and output format (`blake3:<hex>`); augment with file-based Merkle construction.
- Error handling:
  - Reuse `QcapError` and extend with variants as needed.
- CLI parsing:
  - Reuse current `clap` setup; add subcommands with minimal boilerplate impact.

## Gaps to Implement (Upcoming Tasks)

- Manifest struct and JSON helpers (`manifest.rs`).
- Archive creation and reading (`archive.rs`) using tar+gz or zip.
- Deterministic payload Merkle root over directory contents.
- Signature bundle and ed25519 sign/verify.
- CLI commands: `pack`, `verify`, `inspect` (+ optional capability token later).

## Quick Conclusions

The codebase is intentionally minimal. We have the hashing crate and CLI skeleton we need to build upon. Next step (Task 2) is to define the manifest struct and helpers in `qcap-core` with unit tests.
