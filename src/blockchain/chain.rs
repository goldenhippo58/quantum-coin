use crate::blockchain::block::Block;
use crate::blockchain::state::StakingState;
use crate::consensus::pouw::{Task, TaskQueue};
use rocksdb::{IteratorMode, DB};
use serde_json;

#[derive(Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub db: DB,
    pub staking_state: StakingState,
    pub task_queue: TaskQueue,
}

impl Blockchain {
    pub fn new(path: &str) -> Self {
        let db = DB::open_default(path).expect("Failed to open RocksDB");
        let mut blockchain = Blockchain {
            chain: vec![],
            db,
            staking_state: StakingState::new(),
            task_queue: TaskQueue::new(),
        };
        blockchain.load_chain();
        blockchain
    }

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

    pub fn add_block(&mut self, block: Block) {
        self.db
            .put(
                block.index.to_string(),
                serde_json::to_string(&block).unwrap(),
            )
            .expect("Failed to save block to RocksDB");
        self.chain.push(block);
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        block.hash.starts_with("0")
    }

    pub fn latest_block(&self) -> &Block {
        self.chain.last().expect("Blockchain is empty")
    }

    pub fn stake(&mut self, wallet: String, amount: u64) {
        self.staking_state.stake(wallet, amount);
    }

    pub fn unstake(&mut self, wallet: String, amount: u64) -> bool {
        self.staking_state.unstake(wallet, amount)
    }

    pub fn get_rewards(&self, wallet: &String) -> u64 {
        self.staking_state.get_rewards(wallet)
    }

    pub fn add_task(
        &mut self,
        id: String,
        description: String,
        encrypted_data: Vec<u8>,
        reward: u64,
        sphincs_public_key: Vec<u8>,
        signature: Vec<u8>,
    ) {
        let task = Task {
            id: id.clone(),
            description,
            data: encrypted_data,
            reward,
            is_complete: false,
            sphincs_public_key,
            signature,
        };
        self.task_queue.tasks.insert(id, task);
    }

    pub fn complete_task(&mut self, id: &str) -> Option<Task> {
        self.task_queue.complete_task(id)
    }

    pub fn get_pending_tasks(&self) -> Vec<&Task> {
        self.task_queue.get_pending_tasks()
    }

    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.task_queue.get_task(id)
    }
}
