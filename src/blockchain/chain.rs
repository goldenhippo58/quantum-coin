use crate::blockchain::block::Block;
use crate::blockchain::state::StakingState;
use crate::consensus::pouw::{Task, TaskQueue}; // Import Task and TaskQueue
use rocksdb::{IteratorMode, DB};
use serde_json;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,          // Blockchain blocks
    pub db: DB,                     // RocksDB database for persistence
    pub staking_state: StakingState, // Staking state for rewards
    pub task_queue: TaskQueue,      // Queue for PoUW tasks
}

impl Blockchain {
    // Initialize a new Blockchain
    pub fn new(path: &str) -> Self {
        let db = DB::open_default(path).expect("Failed to open RocksDB");
        let mut blockchain = Blockchain {
            chain: vec![],
            db,
            staking_state: StakingState::new(),
            task_queue: TaskQueue::new(), // Initialize task queue
        };
        blockchain.load_chain();
        blockchain
    }

    // Load the blockchain from the database
    pub fn load_chain(&mut self) {
        let iter = self.db.iterator(IteratorMode::Start);
        for item in iter {
            if let Ok((_, value)) = item {
                let block: Block = serde_json::from_slice(&value).unwrap();
                self.chain.push(block);
            }
        }
        if self.chain.is_empty() {
            self.chain.push(Block::genesis());
        }
    }

    // Add a block to the blockchain
    pub fn add_block(&mut self, block: Block) {
        self.db
            .put(
                block.index.to_string(),
                serde_json::to_string(&block).unwrap(),
            )
            .expect("Failed to save block to RocksDB");
        self.chain.push(block);
    }

    // Validate a block (simple PoW example)
    pub fn validate_block(&self, block: &Block) -> bool {
        block.hash.starts_with("0")
    }

    // Staking functions
    pub fn stake(&mut self, wallet: String, amount: u64) {
        self.staking_state.stake(wallet, amount);
    }

    pub fn unstake(&mut self, wallet: String, amount: u64) -> bool {
        self.staking_state.unstake(wallet, amount)
    }

    pub fn get_rewards(&self, wallet: &String) -> u64 {
        self.staking_state.get_rewards(wallet)
    }

    // PoUW Task Management Functions
    pub fn add_task(&mut self, id: String, description: String, data: Vec<u8>, reward: u64) {
        self.task_queue.add_task(id, description, data, reward);
    }

    pub fn complete_task(&mut self, id: &str) -> Option<Task> {
        self.task_queue.complete_task(id)
    }

    pub fn get_pending_tasks(&self) -> Vec<&Task> {
        self.task_queue.get_pending_tasks()
    }
}
