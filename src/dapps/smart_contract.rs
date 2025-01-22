#![cfg_attr(not(feature = "std"), no_std)]

use serde::{Deserialize, Serialize};
use crate::blockchain::chain::Blockchain; // Access the blockchain
use crate::consensus::pouw::{Task, TaskQueue}; // Import Task and TaskQueue
use ink_lang as ink;

#[ink::contract]
mod task_contract {
    use super::*;

    /// Represents a task stored in the blockchain.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct TaskMetadata {
        id: u32,
        description: String,
        encrypted_data: Vec<u8>,
        reward: u64,
        is_complete: bool,
    }

    /// Main smart contract for managing tasks.
    #[ink(storage)]
    pub struct TaskContract {
        tasks: ink_storage::collections::HashMap<u32, TaskMetadata>, // Store tasks
        next_task_id: u32,
        blockchain: Blockchain, // Reference to the blockchain
    }

    impl TaskContract {
        /// Initializes the contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                tasks: Default::default(),
                next_task_id: 0,
                blockchain: Blockchain::new("./blockchain_data"),
            }
        }

        /// Adds a new task to the blockchain.
        #[ink(message)]
        pub fn add_task(&mut self, description: String, encrypted_data: Vec<u8>, reward: u64) -> u32 {
            let task_id = self.next_task_id;
            self.next_task_id += 1;

            let task_metadata = TaskMetadata {
                id: task_id,
                description,
                encrypted_data,
                reward,
                is_complete: false,
            };

            // Add to task queue in blockchain
            self.blockchain.add_task(
                task_id.to_string(),
                task_metadata.description.clone(),
                task_metadata.encrypted_data.clone(),
                task_metadata.reward,
            );

            // Add task metadata to contract storage
            self.tasks.insert(task_id, task_metadata);
            task_id
        }

        /// Completes a task and updates the blockchain.
        #[ink(message)]
        pub fn complete_task(&mut self, task_id: u32, result: Vec<u8>) -> bool {
            let task = self.tasks.get_mut(&task_id).expect("Task not found");

            // Mark the task as complete and update the blockchain
            task.is_complete = true;
            self.blockchain.complete_task(&task_id.to_string());
            true
        }

        /// Retrieves a task by its ID.
        #[ink(message)]
        pub fn get_task(&self, task_id: u32) -> Option<TaskMetadata> {
            self.tasks.get(&task_id).cloned()
        }

        /// Retrieves all pending tasks.
        #[ink(message)]
        pub fn get_pending_tasks(&self) -> Vec<TaskMetadata> {
            self.tasks.values().filter(|task| !task.is_complete).cloned().collect()
        }

        /// Retrieves all completed tasks.
        #[ink(message)]
        pub fn get_completed_tasks(&self) -> Vec<TaskMetadata> {
            self.tasks.values().filter(|task| task.is_complete).cloned().collect()
        }
    }
}
