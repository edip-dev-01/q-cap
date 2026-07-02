use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct QcapSignatureBundle {
    pub merkle_root: String,
    pub signature: String,  // hex
    pub public_key: String, // hex (32 bytes)
    pub algorithm: String,  // "ed25519"
}

pub fn sign_merkle_root(root: &str, keypair: &SigningKey) -> QcapSignatureBundle {
    let sig: Signature = keypair.sign(root.as_bytes());
    let pk: VerifyingKey = keypair.verifying_key();
    QcapSignatureBundle {
        merkle_root: root.to_string(),
        signature: hex::encode(sig.to_bytes()),
        public_key: hex::encode(pk.to_bytes()),
        algorithm: "ed25519".to_string(),
    }
}

pub fn verify_signature(bundle: &QcapSignatureBundle) -> Result<()> {
    if bundle.algorithm != "ed25519" {
        return Err("unsupported algorithm".into());
    }
    let pk_bytes = hex::decode(&bundle.public_key)?;
    let sig_bytes = hex::decode(&bundle.signature)?;
    let pk = VerifyingKey::from_bytes(pk_bytes.as_slice().try_into().map_err(|_| "bad pk")?)?;
    let sig = Signature::from_bytes(sig_bytes.as_slice().try_into().map_err(|_| "bad sig")?);
    pk.verify(bundle.merkle_root.as_bytes(), &sig)
        .map_err(|_| "signature verify failed".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    #[test]
    fn sign_and_verify_root() {
        let keypair = SigningKey::generate(&mut OsRng);
        let root = "blake3:abcdef";
        let bundle = sign_merkle_root(root, &keypair);
        assert_eq!(bundle.algorithm, "ed25519");
        verify_signature(&bundle).expect("verify");
    }

    #[test]
    fn verify_fails_on_tamper() {
        let keypair = SigningKey::generate(&mut OsRng);
        let root = "blake3:abcdef";
        let mut bundle = sign_merkle_root(root, &keypair);
        bundle.merkle_root = "blake3:deadbeef".to_string();
        assert!(verify_signature(&bundle).is_err());
    }
}
