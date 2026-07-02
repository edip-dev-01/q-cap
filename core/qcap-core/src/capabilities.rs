use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CapabilityToken {
    pub cap_root: String,   // merkle_root
    pub allow: String,      // e.g., "read"
    pub expires: String,    // RFC3339 or unix-seconds:<ts>
    pub signature: String,  // hex
    pub public_key: String, // hex
    pub algorithm: String,  // "ed25519"
}

impl CapabilityToken {
    pub fn grant(cap_root: &str, allow: &str, expires: &str, keypair: &SigningKey) -> Self {
        let payload = format!("{}|{}|{}", cap_root, allow, expires);
        let sig: Signature = keypair.sign(payload.as_bytes());
        let pk: VerifyingKey = keypair.verifying_key();
        Self {
            cap_root: cap_root.to_string(),
            allow: allow.to_string(),
            expires: expires.to_string(),
            signature: hex::encode(sig.to_bytes()),
            public_key: hex::encode(pk.to_bytes()),
            algorithm: "ed25519".to_string(),
        }
    }

    pub fn verify(&self) -> Result<()> {
        if self.algorithm != "ed25519" {
            return Err("unsupported algorithm".into());
        }
        let pk_bytes = hex::decode(&self.public_key)?;
        let sig_bytes = hex::decode(&self.signature)?;
        let pk = VerifyingKey::from_bytes(pk_bytes.as_slice().try_into().map_err(|_| "bad pk")?)?;
        let sig = Signature::from_bytes(sig_bytes.as_slice().try_into().map_err(|_| "bad sig")?);
        let payload = format!("{}|{}|{}", self.cap_root, self.allow, self.expires);
        pk.verify(payload.as_bytes(), &sig)
            .map_err(|_| "cap signature verify failed".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    #[test]
    fn grant_and_verify_cap() {
        let sk = SigningKey::generate(&mut OsRng);
        let cap = CapabilityToken::grant("blake3:abc", "read", "unix-seconds:999999", &sk);
        cap.verify().expect("cap verify");
    }

    #[test]
    fn tamper_cap_fails() {
        let sk = SigningKey::generate(&mut OsRng);
        let mut cap = CapabilityToken::grant("blake3:abc", "read", "unix-seconds:999999", &sk);
        cap.allow = "write".into();
        assert!(cap.verify().is_err());
    }
}
