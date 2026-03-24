use serde::{Deserialize, Serialize};

/// A user-created note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub pinned: bool,
    pub tags: Vec<String>,
}

impl Note {
    pub fn new(id: i64, title: impl Into<String>, content: impl Into<String>) -> Self {
        let now = String::from(""); // timestamp handled externally
        Self {
            id,
            title: title.into(),
            content: content.into(),
            created_at: now.clone(),
            updated_at: now,
            pinned: false,
            tags: vec![],
        }
    }

    pub fn preview(&self, max_len: usize) -> &str {
        let s = self.content.as_str();
        if s.len() <= max_len { s } else { &s[..max_len] }
    }
}
