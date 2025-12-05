use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use ed25519_dalek::SigningKey;
use qcap_core::manifest::QcapManifest;
use qcap_core::payload_merkle::compute_payload_merkle_root;
use qcap_core::signatures::{sign_merkle_root, QcapSignatureBundle, verify_signature};
use qcap_core::capabilities::CapabilityToken;
use qcap_core::archive::create_qcap_archive_with_signature;
use zip::read::ZipArchive;
use std::io::Read;
use tempfile::tempdir;

#[derive(Parser)]
#[command(name = "qcap", version, about = "Q-Cap CLI (alpha)")]
struct Cli { #[command(subcommand)] command: Commands }

#[derive(Subcommand)]
enum Commands {
    /// Demo: hash input bytes
    Hash { input: String },
    /// Pack a directory into a .qcap archive
    Pack {
        /// Input directory to include under payload/
        input_dir: PathBuf,
        /// Output .qcap file path
        #[arg(short = 'o', long = "out")]
        out: PathBuf,
        /// Ed25519 private key file (32-byte seed hex or pkcs8 DER not supported yet)
        #[arg(short = 'k', long = "key")]
        key: PathBuf,
    },
    /// Verify a .qcap archive
    Verify {
        /// Input .qcap file
        file: PathBuf,
    },
    /// Inspect a .qcap archive and print summary
    Inspect {
        /// Input .qcap file
        file: PathBuf,
    },
    /// Grant a capability token bound to a .qcap's merkle root
    Grant {
        /// Input .qcap file
        file: PathBuf,
        /// Allow operation (e.g., read)
        #[arg(long = "allow", default_value = "read")]
        allow: String,
        /// Expiry timestamp (unix-seconds:<ts> for MVP)
        #[arg(long = "expires", default_value = "unix-seconds:9999999999")]
        expires: String,
        /// Ed25519 private key seed hex
        #[arg(short = 'k', long = "key")]
        key: PathBuf,
        /// Output cap token path (JSON)
        #[arg(short = 'o', long = "out")]
        out: PathBuf,
    },
    /// Open a .qcap after verifying a capability token, exporting payload
    Open {
        /// Input .qcap file
        file: PathBuf,
        /// Capability token JSON path
        #[arg(long = "cap")]
        cap: PathBuf,
        /// Output directory to export payload
        #[arg(short = 'o', long = "out")]
        out: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Hash { input } => {
            let root = qcap_core::merkle_root_demo(input.as_bytes());
            println!("{}", root);
    }
        Commands::Pack { input_dir, out, key } => {
            // Compute merkle root over payload
            let root = compute_payload_merkle_root(&input_dir).expect("merkle root");

            // Build manifest
            let manifest = QcapManifest::new_with_now(
                "0.1.0",
                &root,
                None,
                serde_json::json!({}),
            );

            // Load private key (hex seed of 32 bytes)
            let key_hex = fs::read_to_string(&key).expect("read key file");
            let key_hex = key_hex.trim();
            let seed = hex::decode(key_hex).expect("decode hex seed");
            assert_eq!(seed.len(), 32, "expected 32-byte seed hex");
            let mut seed_arr = [0u8; 32];
            seed_arr.copy_from_slice(&seed);
            let signing_key = SigningKey::from_bytes(&seed_arr);

            // Sign merkle root
            let sig_bundle = sign_merkle_root(&root, &signing_key);

            // Create archive with signature
            create_qcap_archive_with_signature(&input_dir, &manifest, &sig_bundle, &out)
                .expect("create archive");
            println!("Packed {}\n- merkle root: {}\n- output: {}", input_dir.display(), root, out.display());
        }
        Commands::Verify { file } => {
            // Open zip
            let f = fs::File::open(&file).expect("open qcap");
            let mut zip = ZipArchive::new(f).expect("zip");

            // Read manifest.json
            let manifest: QcapManifest = {
                let mut mf = zip.by_name("manifest.json").expect("manifest.json");
                let mut mstr = String::new();
                mf.read_to_string(&mut mstr).expect("manifest read");
                serde_json::from_str(&mstr).expect("manifest parse")
            };

            // Read signature bundle
            let sig: QcapSignatureBundle = {
                let mut sf = zip.by_name("signatures/manifest.sig.json").expect("sig bundle");
                let mut sstr = String::new();
                sf.read_to_string(&mut sstr).expect("sig read");
                serde_json::from_str(&sstr).expect("sig parse")
            };

            // Extract payload into temp dir
            let tmp = tempdir().expect("tempdir");
            let out_dir = tmp.path();
            let mut payload_files = 0usize;
            for i in 0..zip.len() {
                let mut entry = zip.by_index(i).expect("entry");
                let name = entry.name().to_string();
                if name.starts_with("payload/") {
                    // strip payload/
                    let rel = &name[8..];
                    let dest = out_dir.join(rel);
                    if entry.is_dir() {
                        fs::create_dir_all(&dest).ok();
                    } else {
                        if let Some(parent) = dest.parent() { fs::create_dir_all(parent).ok(); }
                        let mut buf = Vec::new();
                        entry.read_to_end(&mut buf).expect("read payload entry");
                        fs::write(&dest, &buf).expect("write payload entry");
                        payload_files += 1;
                    }
                }
            }

            // Recompute merkle root
            let recomputed = compute_payload_merkle_root(out_dir).expect("recompute merkle");

            // Checks
            let roots_match = recomputed == manifest.merkle_root && recomputed == sig.merkle_root;
            let sig_ok = verify_signature(&sig).is_ok();

            if roots_match && sig_ok {
                println!("Verification: OK\n- merkle root: {}\n- signer: {}\n- payload files: {}", recomputed, &sig.public_key[0..16], payload_files);
            } else {
                println!("Verification: FAILED\n- recomputed: {}\n- manifest: {}\n- bundle: {}\n- signature_ok: {}", recomputed, manifest.merkle_root, sig.merkle_root, sig_ok);
                std::process::exit(1);
            }
        }
        Commands::Inspect { file } => {
            // Open zip
            let f = fs::File::open(&file).expect("open qcap");
            let mut zip = ZipArchive::new(f).expect("zip");

            // Read manifest.json
            let manifest: QcapManifest = {
                let mut mf = zip.by_name("manifest.json").expect("manifest.json");
                let mut mstr = String::new();
                mf.read_to_string(&mut mstr).expect("manifest read");
                serde_json::from_str(&mstr).expect("manifest parse")
            };

            // Read signature bundle (optional; if missing, treat as unknown)
            let sig_summary = if let Ok(mut sf) = zip.by_name("signatures/manifest.sig.json") {
                let mut sstr = String::new();
                sf.read_to_string(&mut sstr).ok();
                if let Ok(sig) = serde_json::from_str::<QcapSignatureBundle>(&sstr) {
                    format!("{}", &sig.public_key.chars().take(16).collect::<String>())
                } else {
                    "(unreadable)".to_string()
                }
            } else {
                "(none)".to_string()
            };

            // Count payload files without extraction
            let mut payload_files = 0usize;
            for i in 0..zip.len() {
                let entry = zip.by_index(i).expect("entry");
                let name = entry.name().to_string();
                if name.starts_with("payload/") && !entry.is_dir() {
                    payload_files += 1;
                }
            }

            println!(
                "Q-Cap Inspect\n- schema_version: {}\n- merkle_root: {}\n- issuer: {}\n- created_at: {}\n- signer: {}\n- payload files: {}",
                manifest.schema_version,
                manifest.merkle_root,
                manifest.issuer.as_deref().unwrap_or("(none)"),
                manifest.created_at,
                sig_summary,
                payload_files
            );
        }
        Commands::Grant { file, allow, expires, key, out } => {
            // Open zip and read manifest
            let f = fs::File::open(&file).expect("open qcap");
            let mut zip = ZipArchive::new(f).expect("zip");
            let manifest: QcapManifest = {
                let mut mf = zip.by_name("manifest.json").expect("manifest.json");
                let mut mstr = String::new();
                mf.read_to_string(&mut mstr).expect("manifest read");
                serde_json::from_str(&mstr).expect("manifest parse")
            };
            // Load private key
            let key_hex = fs::read_to_string(&key).expect("read key file");
            let seed = hex::decode(key_hex.trim()).expect("hex");
            assert_eq!(seed.len(), 32, "expected 32-byte seed");
            let mut seed_arr = [0u8; 32];
            seed_arr.copy_from_slice(&seed);
            let signing_key = SigningKey::from_bytes(&seed_arr);
            // Grant
            let cap = CapabilityToken::grant(&manifest.merkle_root, &allow, &expires, &signing_key);
            let json = serde_json::to_vec_pretty(&cap).expect("cap json");
            fs::write(&out, &json).expect("write cap");
            println!("Granted capability\n- cap_root: {}\n- allow: {}\n- expires: {}\n- out: {}", manifest.merkle_root, allow, expires, out.display());
        }
        Commands::Open { file, cap, out } => {
            // Read capability token and verify
            let cap_bytes = fs::read(&cap).expect("read cap");
            let cap: CapabilityToken = serde_json::from_slice(&cap_bytes).expect("parse cap");
            cap.verify().expect("cap verify");
            // Open archive, read manifest
            let f = fs::File::open(&file).expect("open qcap");
            let mut zip = ZipArchive::new(f).expect("zip");
            let manifest: QcapManifest = {
                let mut mf = zip.by_name("manifest.json").expect("manifest.json");
                let mut mstr = String::new();
                mf.read_to_string(&mut mstr).expect("manifest read");
                serde_json::from_str(&mstr).expect("manifest parse")
            };
            // Ensure cap root matches manifest root and allow includes read
            if cap.cap_root != manifest.merkle_root || cap.allow != "read" {
                println!("Open: FAILED (cap mismatch or disallowed)");
                std::process::exit(1);
            }
            // Export payload to out
            fs::create_dir_all(&out).expect("create out");
            for i in 0..zip.len() {
                let mut entry = zip.by_index(i).expect("entry");
                let name = entry.name().to_string();
                if name.starts_with("payload/") {
                    let rel = &name[8..];
                    let dest = out.join(rel);
                    if entry.is_dir() {
                        fs::create_dir_all(&dest).ok();
                    } else {
                        if let Some(parent) = dest.parent() { fs::create_dir_all(parent).ok(); }
                        let mut buf = Vec::new();
                        entry.read_to_end(&mut buf).expect("read payload entry");
                        fs::write(&dest, &buf).expect("write payload entry");
                    }
                }
            }
            println!("Open: OK\n- exported to: {}", out.display());
        }
    }
}
