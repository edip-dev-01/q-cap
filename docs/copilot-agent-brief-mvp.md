# Q-Cap MVP Build Brief (for GitHub Copilot GPT 5 Agent)

You are operating in this repository:

> https://github.com/jvanulde/q-cap

Q-Cap is "a packaging format for openly publishing data, where access is cryptographically controlled by capability tokens." The repo already has a Rust core library, a Rust CLI, a minimal Go registry, and a TS SDK stub. The README describes a **preview format** for `.qcap` files.

Your job as Copilot Agent is to help implement a **real, minimal MVP** of the Q-Cap format and CLI using the existing architecture.

---

## 1. Context & Existing Structure

From the README (authoritative):

- **Repo structure**: :contentReference[oaicite:2]{index=2}
  - `core/qcap-core/` — Rust library (crypto & format building blocks)
  - `core/qcap-cli/` — Rust CLI (currently only a demo: `hash`)
  - `services/qcap-registry/` — Go minimal registry (health check)
  - `sdks/ts/` — TS SDK stub
  - `api/proto/` — Protobuf IDL stub
  - `docs/` — docs stubs

- **Preview Q-Cap format**: :contentReference[oaicite:3]{index=3}  
  A `.qcap` is currently described as a **ZIP or tar+gz** containing:
  - `manifest.json` — schema version, Merkle root, issuer, policies, metadata
  - `payload/` — arbitrary files (optionally encrypted per file)
  - `meta/` — readme, license, schemas, STAC/OGC tags
  - `signatures/` — detached signatures (ed25519) over the manifest & Merkle root

The core library already mentions:

- BLAKE3 Merkle tree
- XChaCha20-Poly1305 AEAD
- ed25519
- Argon2id
- Macaroons-style capabilities

These primitives should be **reused**, not re-invented.

---

## 2. MVP Goal (What "Done" Looks Like)

Implement a minimal Q-Cap **MVP** that can:

1. **Pack**:
   - Take an input directory (or single file).
   - Produce a `.qcap` archive (tar+gz or zip) structured as:
     - `manifest.json`
     - `payload/…`
     - `meta/…` (optional)
     - `signatures/…`

2. **Sign & Verify**:
   - Compute a BLAKE3 Merkle root over the `payload/` contents.
   - Store that root in `manifest.json`.
   - Sign the root (and maybe manifest) with ed25519.
   - Store signature + public key in `signatures/`.
   - Verify everything in `qcap verify`.

3. **Inspect / Describe**:
   - `qcap inspect` (or `qcap describe`) prints a human-friendly summary:
     - manifest info
     - Merkle root
     - signer fingerprint
     - list of payload files

4. (Optional MVP) **Simple capability token**:
   - A small macaroons-style capability that:
     - is bound to the Merkle root
     - controls read/export
   - `qcap open` that checks a capability before exporting payload content.

We **do not** need full registry integration, DP ledger, vector indexes, or WASM lenses for this MVP.

---

## 3. Constraints & Design Choices

- Use **Rust 2021**.
- Reuse existing crates and modules within `core/qcap-core` where possible (especially crypto).
- Preserve the **preview structure** from README:
  - Archive-based `.qcap` (zip or tar+gz).
  - `manifest.json` as the central description.
  - Merkle root + signatures as in the README security model. :contentReference[oaicite:4]{index=4}
- Keep the CLI UX simple and Unix-like.

Do **NOT**:
- Introduce custom TLV-based binary containers in this MVP (that can be a later evolution).
- Change the high-level security model already described in the README without explicit comments.

---

## 4. Incremental Tasks for Copilot Agent

Implement these in **small, testable steps**.

### Task 1 — Understand existing core & CLI

1. Inspect:
   - `core/qcap-core/src/lib.rs` and submodules.
   - `core/qcap-cli/src/main.rs` (or equivalent).
2. Identify:
   - Existing BLAKE3 hashing helpers.
   - ed25519 key/sign/verify helpers.
   - Any Merkle-related code.
   - Any capability/macaroons-related code.

**Output**: A short `docs/mvp-notes-core.md` summarizing what’s already there and what can be reused.

---

### Task 2 — Define Manifest Struct

1. In `core/qcap-core`, create a manifest module, e.g. `src/manifest.rs`, with a serializable type:

   Example (adjust as needed):

   ```rust
   #[derive(Serialize, Deserialize, Debug, Clone)]
   pub struct QcapManifest {
       pub schema_version: String,        // e.g. "0.1.0"
       pub merkle_root: String,           // "blake3:<hex>"
       pub issuer: Option<String>,        // key fingerprint or issuer id
       pub created_at: String,            // RFC3339
       pub metadata: serde_json::Value,   // free-form (title, description, tags, etc.)
   }
````

2. Expose helper functions:

   * `QcapManifest::new(...)`
   * `to_json_bytes()`
   * `from_json_bytes(...)`

3. Add unit tests for manifest (round-trip JSON).

---

### Task 3 — Archive Writer (Packing)

In `core/qcap-core`:

1. Add a module `src/archive.rs` that:

   * Accepts:

     * an input directory path.
     * a manifest instance.
   * Produces:

     * a `.qcap` archive (choose **tar+gz** or **zip**, whichever is easier with existing dependencies).

2. Layout:

   * `manifest.json` at root of archive.
   * Everything under the input directory mapped to `payload/…` inside the archive.
   * Create empty `meta/` and `signatures/` directories for now if needed.

3. Provide a function like:

   ```rust
   pub fn create_qcap_archive(
       input_dir: &Path,
       manifest: &QcapManifest,
       output_file: &Path,
   ) -> Result<()> { ... }
   ```

4. Add tests using a temp directory with a couple of small files.

---

### Task 4 — Merkle Root Over Payload

In `core/qcap-core`:

1. Implement a function to:

   * Walk files that will be placed under `payload/`.
   * Compute a deterministic BLAKE3 Merkle root over their contents and relative paths.

2. The simplest approach is:

   * Sort payload file paths.
   * For each, compute `leaf = blake3(path_bytes || 0x00 || file_contents)`.
   * Build a binary Merkle tree over leaves (existing helper if present).
   * Encode root as `"blake3:<hex>"`.

3. Expose as:

   ```rust
   pub fn compute_payload_merkle_root(input_dir: &Path) -> Result<String> { ... }
   ```

4. Add unit tests that:

   * Use known fixtures.
   * Check the root is stable for the same content ordering.

---

### Task 5 — Signing & Signatures Directory

In `core/qcap-core`:

1. Define a signatures struct:

   ```rust
   #[derive(Serialize, Deserialize, Debug, Clone)]
   pub struct QcapSignatureBundle {
       pub merkle_root: String,
       pub signature: String,      // base64 or hex
       pub public_key: String,     // base64 or hex
       pub algorithm: String,      // e.g. "ed25519"
   }
   ```

2. Implement:

   * `sign_merkle_root(root: &str, key: &Ed25519Keypair) -> QcapSignatureBundle`
   * `verify_signature(bundle: &QcapSignatureBundle) -> Result<()>`

3. During packing:

   * After computing `merkle_root`, create manifest.
   * Sign the root, create a `signatures/manifest.sig.json` file inside the archive.

4. Add tests for signature creation & verification.

---

### Task 6 — CLI: `qcap pack`

In `core/qcap-cli`:

1. Extend the CLI (using `clap` or whatever is already there) to add:

   ```bash
   qcap pack <INPUT_DIR> --out <OUTPUT.qcap> --key <ED25519_PRIVKEY_PATH>
   ```

2. Behavior:

   * Load private key.
   * Compute payload Merkle root.
   * Build manifest (with timestamp).
   * Create `.qcap` archive:

     * write `manifest.json`
     * write `payload/` content
     * write `signatures/manifest.sig.json`
   * Print a short success message including:

     * output path
     * merkle root
     * signer fingerprint (if easy)

3. Add an integration test in `qcap-cli` that:

   * Creates a temp dir with some files.
   * Runs `qcap pack`.
   * Asserts the `.qcap` file exists and can be opened as a tar/zip.

---

### Task 7 — CLI: `qcap verify`

In `core/qcap-cli`:

1. Implement:

   ```bash
   qcap verify <FILE.qcap>
   ```

2. Behavior:

   * Open archive.
   * Read `manifest.json` into `QcapManifest`.
   * Read `signatures/manifest.sig.json` into `QcapSignatureBundle`.
   * Extract `payload/` into a temp directory (or stream verify if convenient).
   * Recompute Merkle root from payload.
   * Verify:

     * recomputed root == manifest.merkle_root == bundle.merkle_root
     * signature is valid.

3. Print a human-readable summary:

   * "Verification: OK/FAILED"
   * Merkle root
   * Signer pubkey (short fingerprint)
   * Number of payload files

4. Add an integration test:

   * Pack a capsule → verify it → assert success.
   * Modify one payload file → verify should fail.

---

### Task 8 — CLI: `qcap inspect` (Describe)

Add:

```bash
qcap inspect <FILE.qcap>
```

Behavior:

* Read manifest and signatures only (no full payload extraction if possible).
* List:

  * schema_version
  * merkle_root
  * issuer (if present)
  * created_at
  * count of payload files (if cheap to compute)
* Print as a neat table or structured text.

---

### Task 9 (Optional) — Minimal Capability Token

If time/complexity permits:

1. In `qcap-core`, define a very simple macaroons-based capability structure bound to the `merkle_root`:

   * `cap_root` (same as merkle root)
   * `allow` (`"read"` for MVP)
   * `expires` (RFC3339)
   * signature / macaroons caveats reusing existing capability code.

2. In CLI:

   * `qcap grant <FILE.qcap> --key <ED25519_PRIVKEY> --allow read --out cap.json`
   * `qcap open <FILE.qcap> --cap cap.json --out <OUTPUT_DIR>`

3. Enforce:

   * `open` MUST verify the capsule and then verify the capability token.

This can be basic; the main point is to show **self-governing artifact** behavior.

---

## 5. Out of Scope for This MVP

Do **NOT** implement the following yet (can be later milestones):

* Privacy ledger / differential privacy
* WASM Lenses or on-file compute
* Vector indexes (IVF/HNSW)
* TEE attestation
* Full registry integration (beyond existing health check)
* TLV-style monolithic binary container (stick to archive-based format for now)

---

## 6. General Expectations

* Keep changes incremental and well-factored.
* Add unit tests and integration tests where feasible.
* Do not break the existing `hash` demo command; extend it with new subcommands.
* Keep public APIs in `qcap-core` reasonably stable and documented in Rustdoc.

Once these tasks are done, the project will have a **working `.qcap` MVP** that:

* Can be produced from real data,
* Verified offline,
* Inspected easily,
* And extended later toward the richer TLV + on-file-policy design.
