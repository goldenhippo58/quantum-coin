use pqcrypto_kyber::kyber512;

pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let (public, private) = kyber512::keypair();
    (public.to_vec(), private.to_vec())
}

pub fn encrypt(message: &[u8], public_key: &[u8]) -> Vec<u8> {
    kyber512::enc(message, public_key)
}

pub fn decrypt(ciphertext: &[u8], private_key: &[u8]) -> Vec<u8> {
    kyber512::dec(ciphertext, private_key)
}
