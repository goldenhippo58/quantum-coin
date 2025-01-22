use crate::blockchain::block::{Block, Transaction};
use crate::blockchain::chain::Blockchain;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use pqcrypto_kyber::kyber512;
use pqcrypto_traits::kem::{PublicKey, SecretKey};
use rocksdb::DB;
use std::sync::{Arc, Mutex};

pub fn create_wallet() -> (String, String) {
    let (public_key, private_key) = kyber512::keypair();
    let public_key_b64 = STANDARD.encode(public_key.as_bytes());
    let private_key_b64 = STANDARD.encode(private_key.as_bytes());

    // Save wallet to RocksDB
    let db = DB::open_default("./wallets").expect("Failed to open wallet database");
    db.put(&public_key_b64, &private_key_b64)
        .expect("Failed to save wallet");

    (public_key_b64, private_key_b64)
}

pub fn send_transaction(blockchain: Arc<Mutex<Blockchain>>, sender: &str, to: &str, amount: u64) {
    println!(
        "Creating a transaction to send {} QuantumCoins from {} to {}",
        amount, sender, to
    );

    // Retrieve sender's private key from RocksDB
    let db = DB::open_default("./wallets").expect("Failed to open wallet database");
    let _private_key = match db.get(sender) {
        Ok(Some(key)) => String::from_utf8(key.to_vec()).expect("Failed to parse private key"),
        Ok(None) => {
            println!("Wallet not found for sender: {}", sender);
            return;
        }
        Err(_) => {
            println!("Failed to retrieve wallet from database.");
            return;
        }
    };

    // Create a cryptographic signature (placeholder logic)
    let signature = format!("SIGNATURE({}:{}:{})", sender, to, amount);

    let transaction = Transaction {
        sender: sender.to_string(),
        receiver: to.to_string(),
        amount,
        signature: signature.into_bytes(),
    };

    let mut blockchain = blockchain.lock().unwrap();
    let previous_hash = blockchain
        .chain
        .last()
        .unwrap_or(&Block::genesis())
        .hash
        .clone();
    let new_block = Block::new(
        blockchain.chain.len() as u64,
        previous_hash,
        vec![transaction],
    );

    blockchain.add_block(new_block);
    println!("Transaction added to the blockchain!");
}
