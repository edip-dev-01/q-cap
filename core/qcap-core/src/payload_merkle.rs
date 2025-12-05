use blake3::Hasher;
use std::fs::File;
use std::io::{Read};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Compute a deterministic BLAKE3 Merkle root over all files under `input_dir`.
///
/// Leaf hash: blake3(path_bytes || 0x00 || file_contents)
/// Tree: pairwise concatenation of child digests (as bytes) hashed again; odd leaf promoted.
/// Final root encoded as `blake3:<hex>`.
pub fn compute_payload_merkle_root(input_dir: &Path) -> Result<String> {
    let files = list_files(input_dir)?;
    let mut leaves: Vec<[u8; 32]> = Vec::new();
    for abs in files {
        let rel = abs.strip_prefix(input_dir).unwrap();
        let path_bytes = rel.as_os_str().to_string_lossy().replace('\\', "/").into_bytes();
    let mut h = Hasher::new();
    h.update(&path_bytes);
    h.update(&[0]);
        let mut f = File::open(&abs)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        h.update(&buf);
        leaves.push(*h.finalize().as_bytes());
    }

    if leaves.is_empty() {
        // empty directory -> hash of empty string
        let mut h = Hasher::new();
        let root = h.finalize();
        return Ok(format!("blake3:{}", root.to_hex()));
    }

    // Build tree
    let mut level = leaves;
    while level.len() > 1 {
        let mut next: Vec<[u8; 32]> = Vec::new();
        let mut i = 0;
        while i < level.len() {
            if i + 1 < level.len() {
                let mut h = Hasher::new();
                h.update(&level[i]);
                h.update(&level[i + 1]);
                next.push(*h.finalize().as_bytes());
                i += 2;
            } else {
                // odd promotion
                next.push(level[i]);
                i += 1;
            }
        }
        level = next;
    }
    Ok(format!("blake3:{}", blake3::Hash::from_bytes(level[0]).to_hex()))
}

fn list_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() {
            out.push(p.to_path_buf());
        }
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn merkle_root_is_stable_for_same_content() {
        let dir1 = tempdir().unwrap();
        let root1 = dir1.path();
        std::fs::create_dir(root1.join("a")).unwrap();
        std::fs::write(root1.join("a/one.txt"), b"hello").unwrap();
        std::fs::write(root1.join("a/two.txt"), b"world").unwrap();

        let dir2 = tempdir().unwrap();
        let root2 = dir2.path();
        std::fs::create_dir(root2.join("b")).unwrap();
        std::fs::write(root2.join("b/one.txt"), b"hello").unwrap();
        std::fs::write(root2.join("b/two.txt"), b"world").unwrap();

        let r1 = compute_payload_merkle_root(&root1.join("a")).unwrap();
        let r2 = compute_payload_merkle_root(&root2.join("b")).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn merkle_root_changes_when_content_changes() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir(root.join("d")).unwrap();
        std::fs::write(root.join("d/x.txt"), b"alpha").unwrap();
        std::fs::write(root.join("d/y.txt"), b"beta").unwrap();
        let r1 = compute_payload_merkle_root(&root.join("d")).unwrap();
        std::fs::write(root.join("d/y.txt"), b"beta!").unwrap();
        let r2 = compute_payload_merkle_root(&root.join("d")).unwrap();
        assert_ne!(r1, r2);
    }
}
