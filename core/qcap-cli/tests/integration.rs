use std::fs;
use std::path::PathBuf;
use assert_cmd::Command;
use tempfile::tempdir;
use zip::ZipArchive;

fn write_seed_hex(path: &PathBuf) {
    // Deterministic 32-byte seed for reproducible tests
    fs::write(path, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").unwrap();
}

#[test]
fn pack_then_verify_ok() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let payload = root.join("payload");
    fs::create_dir_all(&payload).unwrap();
    fs::write(payload.join("a.txt"), b"hello").unwrap();
    fs::write(payload.join("b.bin"), [1u8, 2, 3, 4]).unwrap();

    let out = root.join("demo.qcap");
    let seed = root.join("seed.hex");
    write_seed_hex(&seed);

    // Pack
    Command::new(assert_cmd::cargo::cargo_bin!("qcap-cli"))
        .args(["pack", payload.to_str().unwrap(), "--out", out.to_str().unwrap(), "--key", seed.to_str().unwrap()])
        .assert()
        .success();

    assert!(out.exists(), "qcap created");

    // Verify
    Command::new(assert_cmd::cargo::cargo_bin!("qcap-cli"))
        .args(["verify", out.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn verify_fails_on_tamper() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let payload = root.join("payload");
    fs::create_dir_all(&payload).unwrap();
    fs::write(payload.join("a.txt"), b"hello").unwrap();
    fs::write(payload.join("b.bin"), [1u8, 2, 3, 4]).unwrap();

    let out = root.join("demo.qcap");
    let seed = root.join("seed.hex");
    write_seed_hex(&seed);

    // Pack
    Command::new(assert_cmd::cargo::cargo_bin!("qcap-cli"))
        .args(["pack", payload.to_str().unwrap(), "--out", out.to_str().unwrap(), "--key", seed.to_str().unwrap()])
        .assert()
        .success();

    // Tamper: rewrite payload/a.txt within the zip
    {
        let f = fs::OpenOptions::new().read(true).write(true).open(&out).unwrap();
        let mut zip = ZipArchive::new(f).unwrap();
        {
            let mut file = zip.by_name("payload/a.txt").unwrap();
            // Read original to satisfy borrow; then write new content via replace operation
            let mut _buf = Vec::new();
            use std::io::Read;
            file.read_to_end(&mut _buf).unwrap();
        }
        // ZipArchive doesn't support in-place modification easily; instead, extract and rebuild quick tamper:
    }
    // Simpler tamper: recreate archive with changed payload content
    let tampered = root.join("demo_tampered.qcap");
    {
        // Extract original and rewrite file
        let f = fs::File::open(&out).unwrap();
        let mut zip = ZipArchive::new(f).unwrap();
        let extract_dir = root.join("extract");
        fs::create_dir_all(&extract_dir).unwrap();
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).unwrap();
            let name = entry.name().to_string();
            let dest = extract_dir.join(&name);
            if entry.is_dir() {
                fs::create_dir_all(&dest).ok();
            } else {
                if let Some(parent) = dest.parent() { fs::create_dir_all(parent).ok(); }
                let mut buf = Vec::new();
                use std::io::Read;
                entry.read_to_end(&mut buf).unwrap();
                fs::write(&dest, &buf).unwrap();
            }
        }
        // Overwrite payload file
        fs::write(extract_dir.join("payload/a.txt"), b"tampered").unwrap();
        // Repack minimal: rebuild zip from extracted tree
        let file = fs::File::create(&tampered).unwrap();
        let mut zw = zip::ZipWriter::new(file);
        let opts = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for entry in walkdir::WalkDir::new(&extract_dir).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            let rel = p.strip_prefix(&extract_dir).unwrap();
            let name = rel.to_str().unwrap().replace('\\', "/");
            if entry.file_type().is_dir() {
                zw.add_directory(name.clone(), opts).ok();
            } else {
                zw.start_file(name.clone(), opts).unwrap();
                let bytes = fs::read(p).unwrap();
                use std::io::Write;
                zw.write_all(&bytes).unwrap();
            }
        }
        zw.finish().unwrap();
    }

    Command::new(assert_cmd::cargo::cargo_bin!("qcap-cli"))
        .args(["verify", tampered.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn grant_and_open_flow() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let payload = root.join("payload");
    fs::create_dir_all(&payload).unwrap();
    fs::write(payload.join("a.txt"), b"hello").unwrap();
    fs::write(payload.join("b.bin"), [1u8, 2, 3, 4]).unwrap();

    let out_qcap = root.join("demo.qcap");
    let seed = root.join("seed.hex");
    let cap = root.join("cap.json");
    let export_dir = root.join("exported");
    write_seed_hex(&seed);

    // Pack
    Command::new(assert_cmd::cargo::cargo_bin!("qcap-cli"))
        .args(["pack", payload.to_str().unwrap(), "--out", out_qcap.to_str().unwrap(), "--key", seed.to_str().unwrap()])
        .assert()
        .success();

    // Grant
    Command::new(assert_cmd::cargo::cargo_bin!("qcap-cli"))
        .args(["grant", out_qcap.to_str().unwrap(), "--allow", "read", "--expires", "unix-seconds:9999999999", "--key", seed.to_str().unwrap(), "--out", cap.to_str().unwrap()])
        .assert()
        .success();
    assert!(cap.exists(), "cap token created");

    // Open
    Command::new(assert_cmd::cargo::cargo_bin!("qcap-cli"))
        .args(["open", out_qcap.to_str().unwrap(), "--cap", cap.to_str().unwrap(), "--out", export_dir.to_str().unwrap()])
        .assert()
        .success();
    assert!(export_dir.join("a.txt").exists());
    assert!(export_dir.join("b.bin").exists());
}
