use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String, // Task description (e.g., "Train an AI model").
    pub data: Vec<u8>,       // Input data for the task.
    pub reward: u64,         // Reward for completing the task.
    pub is_complete: bool,   // Task completion status.
}

#[derive(Debug, Clone)]
pub struct TaskQueue {
    pub tasks: HashMap<String, Task>, // Task ID -> Task details.
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, id: String, description: String, data: Vec<u8>, reward: u64) {
        let task = Task {
            id,
            description,
            data,
            reward,
            is_complete: false,
        };
        self.tasks.insert(task.id.clone(), task);
    }

    pub fn complete_task(&mut self, id: &str) -> Option<Task> {
        if let Some(task) = self.tasks.get_mut(id) {
            task.is_complete = true;
            return Some(task.clone());
        }
        None
    }

    pub fn get_pending_tasks(&self) -> Vec<&Task> {
        self.tasks.values().filter(|t| !t.is_complete).collect()
    }
}
