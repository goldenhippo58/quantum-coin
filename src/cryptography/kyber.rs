use pqcrypto_kyber::kyber512::{
    decapsulate, encapsulate, keypair, Ciphertext, PublicKey, SecretKey, SharedSecret,
};
use pqcrypto_traits::kem::{Ciphertext as _, PublicKey as _, SecretKey as _, SharedSecret as _};
use std::error::Error;

pub struct KyberEncryption;

impl KyberEncryption {
    pub fn generate_keypair() -> (PublicKey, SecretKey) {
        keypair()
    }

    pub fn encrypt(public_key: &PublicKey) -> Result<(SharedSecret, Ciphertext), Box<dyn Error>> {
        let (shared_secret, ciphertext) = encapsulate(public_key);
        Ok((shared_secret, ciphertext))
    }

    pub fn decrypt(
        ciphertext: &Ciphertext,
        secret_key: &SecretKey,
    ) -> Result<SharedSecret, Box<dyn Error>> {
        let shared_secret = decapsulate(ciphertext, secret_key);
        Ok(shared_secret)
    }

    // Helper methods for byte conversion
    pub fn ciphertext_from_bytes(bytes: &[u8]) -> Option<Ciphertext> {
        use pqcrypto_traits::kem::Ciphertext;
        match Ciphertext::from_bytes(bytes) {
            Ok(ct) => Some(ct),
            Err(_) => None,
        }
    }

    pub fn secret_key_from_bytes(bytes: &[u8]) -> Option<SecretKey> {
        use pqcrypto_traits::kem::SecretKey;
        match SecretKey::from_bytes(bytes) {
            Ok(sk) => Some(sk),
            Err(_) => None,
        }
    }
}
