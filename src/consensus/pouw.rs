use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub data: Vec<u8>,
    pub reward: u64,
    pub is_complete: bool,
}

#[derive(Debug, Clone)]
pub struct TaskQueue {
    pub tasks: HashMap<String, Task>,
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
        self.tasks.values().filter(|task| !task.is_complete).collect()
    }
}
