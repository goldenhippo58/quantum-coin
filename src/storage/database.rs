use rocksdb::{DB, Options};
use crate::blockchain::block::Block;

pub struct BlockchainDB {
    db: DB,
}

impl BlockchainDB {
    pub fn new(path: &str) -> Self {
        let db = DB::open_default(path).expect("Failed to open RocksDB");
        BlockchainDB { db }
    }

    pub fn save_block(&self, block: &Block) {
        let serialized = serde_json::to_string(block).expect("Failed to serialize block");
        self.db.put(block.index.to_string(), serialized).expect("Failed to save block");
    }

    pub fn load_blocks(&self) -> Vec<Block> {
        let mut blocks = Vec::new();
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for (_, value) in iter {
            let block: Block = serde_json::from_slice(&value).expect("Failed to deserialize block");
            blocks.push(block);
        }
        blocks
    }
}
