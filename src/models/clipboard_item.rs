use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClipboardContentType {
    Text,
    Image,
    File,
}

impl Default for ClipboardContentType {
    fn default() -> Self { Self::Text }
}

/// A single entry in clipboard history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: String,
    pub content_type: ClipboardContentType,
    pub timestamp: String,
    pub pinned: bool,
}

impl ClipboardItem {
    pub fn new_text(id: i64, content: impl Into<String>, timestamp: impl Into<String>) -> Self {
        Self {
            id,
            content: content.into(),
            content_type: ClipboardContentType::Text,
            timestamp: timestamp.into(),
            pinned: false,
        }
    }

    pub fn preview(&self, max_len: usize) -> &str {
        let s = self.content.as_str();
        if s.len() <= max_len { s } else { &s[..max_len] }
    }

    pub fn is_text(&self) -> bool {
        self.content_type == ClipboardContentType::Text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_item_preview() {
        let item = ClipboardItem::new_text(1, "Hello, world!", "2024-01-01");
        assert_eq!(item.preview(5), "Hello");
        assert_eq!(item.preview(100), "Hello, world!");
    }

    #[test]
    fn clipboard_item_is_text() {
        let item = ClipboardItem::new_text(1, "test", "now");
        assert!(item.is_text());
    }
}
