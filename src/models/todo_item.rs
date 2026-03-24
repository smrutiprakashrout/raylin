use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

impl Default for Priority {
    fn default() -> Self { Self::Medium }
}

/// A single to-do item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub priority: Priority,
    pub due_date: Option<String>,
    pub project_id: Option<i64>,
    pub tags: Vec<String>,
}

impl TodoItem {
    pub fn new(id: i64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            completed: false,
            priority: Priority::default(),
            due_date: None,
            project_id: None,
            tags: vec![],
        }
    }

    pub fn toggle(&mut self) {
        self.completed = !self.completed;
    }
}
