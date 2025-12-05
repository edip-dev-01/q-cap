#![forbid(unsafe_code)]
use blake3::Hasher;
use thiserror::Error;

pub mod manifest;
pub mod archive;
pub mod payload_merkle;
pub mod signatures;
pub mod capabilities;

#[derive(Debug, Error)]
pub enum QcapError {
    #[error("generic: {0}")]
    Generic(String),
}

pub fn merkle_root_demo(bytes: &[u8]) -> String {
    let mut h = Hasher::new();
    h.update(bytes);
    format!("blake3:{}", h.finalize().to_hex())
}
