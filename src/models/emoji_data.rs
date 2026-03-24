use serde::{Deserialize, Serialize};

/// Raw emoji entry as stored in emojis.json.
#[derive(Deserialize, Debug, Clone)]
pub struct EmojiRawData {
    pub name: String,
    pub group: String,
}

/// Processed emoji with character included.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiData {
    pub character: String,
    pub name: String,
    pub group: String,
}

/// A category of emojis (e.g. "Smileys & Emotion").
#[derive(Debug, Clone)]
pub struct EmojiCategory {
    pub name: String,
    pub emojis: Vec<EmojiData>,
}

/// Emoji with grid coordinates for the picker UI.
#[derive(Debug, Clone)]
pub struct EmojiWithPosition {
    pub emoji: EmojiData,
    pub row: i32,
    pub col: i32,
    pub orig_index: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emoji_data_clone() {
        let e = EmojiData {
            character: "😀".to_string(),
            name: "grinning face".to_string(),
            group: "Smileys & Emotion".to_string(),
        };
        let c = e.clone();
        assert_eq!(c.character, "😀");
    }

    #[test]
    fn emoji_category_new() {
        let cat = EmojiCategory {
            name: "Smileys & Emotion".to_string(),
            emojis: vec![],
        };
        assert_eq!(cat.name, "Smileys & Emotion");
        assert!(cat.emojis.is_empty());
    }
}
