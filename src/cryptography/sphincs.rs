use pqcrypto_sphincsplus::sphincssha2128fsimple::{
    detached_sign, keypair, verify_detached_signature, DetachedSignature, PublicKey, SecretKey,
};
use pqcrypto_traits::sign::DetachedSignature as _;
use std::error::Error;

pub struct Sphincs;

impl Sphincs {
    pub fn generate_keypair() -> Result<(PublicKey, SecretKey), Box<dyn Error>> {
        let (public_key, secret_key) = keypair();
        Ok((public_key, secret_key))
    }

    pub fn sign(message: &[u8], secret_key: &SecretKey) -> Result<Vec<u8>, Box<dyn Error>> {
        let signature = detached_sign(message, secret_key);
        Ok(DetachedSignature::as_bytes(&signature).to_vec())
    }

    pub fn verify(message: &[u8], signature_bytes: &[u8], public_key: &PublicKey) -> bool {
        if let Ok(ds) = DetachedSignature::from_bytes(signature_bytes) {
            verify_detached_signature(&ds, message, public_key).is_ok()
        } else {
            false
        }
    }
}
