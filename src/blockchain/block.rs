use sha3::{Digest, Sha3_256};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub previous_hash: String,
    pub hash: String,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        let block_data = format!("{:?}{:?}{:?}", index, &transactions, timestamp);
        let hash = Sha3_256::digest(block_data.as_bytes());

        Block {
            index,
            timestamp,
            previous_hash,
            hash: format!("{:x}", hash),
            transactions,
        }
    }
}
