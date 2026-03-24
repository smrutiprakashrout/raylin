use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    InProgress,
    Paused,
    Completed,
    Archived,
}

impl Default for ProjectStatus {
    fn default() -> Self { Self::Active }
}

/// A project grouping multiple to-do items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub status: ProjectStatus,
    pub color: String,  // hex color, e.g. "#89b4fa"
    pub icon: String,   // emoji or icon name
}

impl Project {
    pub fn new(id: i64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: String::new(),
            status: ProjectStatus::default(),
            color: "#89b4fa".to_string(),
            icon: "📁".to_string(),
        }
    }
}
