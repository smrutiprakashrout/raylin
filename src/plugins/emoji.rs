use std::error::Error;

use nucleo::{Config, Nucleo};

use crate::models::{EmojiData, EmojiRawData};
use super::Plugin;

// ──────────────────────────────────────────────────────────────────────────────
// EmojiPlugin
// ──────────────────────────────────────────────────────────────────────────────

pub struct EmojiPlugin {
    /// Ordered map: category_name → emojis (preserves JSON insertion order)
    pub emoji_db: indexmap::IndexMap<String, Vec<EmojiData>>,
    nucleo: Option<Nucleo<EmojiData>>,
}

impl EmojiPlugin {
    pub fn new() -> Self {
        Self {
            emoji_db: indexmap::IndexMap::new(),
            nucleo: None,
        }
    }

    // ── Initialization ───────────────────────────────────────────────────────

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.emoji_db = Self::load_emojis();

        let mut nucleo = Nucleo::<EmojiData>::new(
            Config::DEFAULT,
            std::sync::Arc::new(|| {}),
            None,
            1,
        );
        let injector = nucleo.injector();
        // Inject in JSON order so blank-query results preserve category sequence
        for (_group, emojis) in &self.emoji_db {
            for e in emojis {
                injector.push(e.clone(), |e_item, c| { c[0] = e_item.name.clone().into(); });
            }
        }
        while nucleo.tick(10).running {}

        self.nucleo = Some(nucleo);
        Ok(())
    }

    // ── Data Loading ─────────────────────────────────────────────────────────

    /// Load emojis from the embedded JSON file.
    fn load_emojis() -> indexmap::IndexMap<String, Vec<EmojiData>> {
        let emoji_str = include_str!("../../emojis.json");
        let parsed: Vec<indexmap::IndexMap<String, EmojiRawData>> =
            serde_json::from_str(emoji_str).unwrap_or_else(|e| {
                eprintln!("[emoji] Error parsing emojis.json: {:?}", e);
                vec![]
            });

        let mut groups: indexmap::IndexMap<String, Vec<EmojiData>> = indexmap::IndexMap::new();
        for map in parsed {
            for (character, raw) in map {
                groups.entry(raw.group.clone()).or_insert_with(Vec::new).push(EmojiData {
                    character,
                    name: raw.name,
                    group: raw.group,
                });
            }
        }
        groups
    }

    // ── Search ───────────────────────────────────────────────────────────────

    /// Fuzzy-search emojis by name.
    /// Returns `(group_key_order, category_map)` with grid positions computed.
    /// `limit` caps total emoji count shown.
    pub fn search_for_ui(
        &mut self,
        query: &str,
        limit: usize,
    ) -> indexmap::IndexMap<String, Vec<EmojiResultData>> {
        let nucleo = match &mut self.nucleo {
            Some(n) => n,
            None => return indexmap::IndexMap::new(),
        };

        nucleo.pattern.reparse(
            0, query,
            nucleo::pattern::CaseMatching::Ignore,
            nucleo::pattern::Normalization::Smart,
            false,
        );
        while nucleo.tick(10).running {}

        let snapshot = nucleo.snapshot();
        let count = snapshot.matched_item_count().min(limit as u32);

        // Pre-populate with all category keys to preserve JSON order on blank queries
        let mut categories_map: indexmap::IndexMap<String, Vec<EmojiResultData>> =
            indexmap::IndexMap::new();
        if query.is_empty() {
            for key in self.emoji_db.keys() {
                categories_map.insert(key.clone(), Vec::new());
            }
        }

        let mut flat_index: i32 = 0;
        for i in 0..count {
            if let Some(item) = snapshot.get_matched_item(i) {
                let group_name = item.data.group.clone();
                let group_list = categories_map.entry(group_name).or_insert_with(Vec::new);
                group_list.push(EmojiResultData {
                    character: item.data.character.clone(),
                    name: item.data.name.clone(),
                    row: (group_list.len() / 7) as i32,
                    col: (group_list.len() % 7) as i32,
                    orig_index: flat_index,
                });
                flat_index += 1;
            }
        }

        // Remove empty categories (when searching)
        categories_map.into_iter().filter(|(_, v)| !v.is_empty()).collect()
    }

    pub fn get_all(&self) -> impl Iterator<Item = &EmojiData> {
        self.emoji_db.values().flat_map(|v| v.iter())
    }

    pub fn get_groups(&self) -> Vec<&String> {
        self.emoji_db.keys().collect()
    }

    // ── Clipboard ────────────────────────────────────────────────────────────

    /// Copy an emoji character to clipboard.
    /// Uses wl-copy for Wayland (GNOME/KDE), falls back to xclip for X11.
    pub fn copy_to_clipboard(character: &str) {
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        if wayland {
            let _ = std::process::Command::new("wl-copy").arg(character).spawn();
        } else {
            let _ = std::process::Command::new("xclip")
                .args(["-selection", "clipboard"])
                .arg(character)
                .spawn();
        }
    }
}

impl Default for EmojiPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// ── Intermediate result type used when building Slint model ─────────────────

#[derive(Clone, Debug)]
pub struct EmojiResultData {
    pub character: String,
    pub name: String,
    pub row: i32,
    pub col: i32,
    pub orig_index: i32,
}

// ── Plugin Trait ─────────────────────────────────────────────────────────────

impl Plugin for EmojiPlugin {
    fn id(&self) -> &str { "emoji" }
    fn name(&self) -> &str { "Emoji Picker" }
    fn trigger(&self) -> &str { "@e" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn init(&mut self) -> Result<(), Box<dyn Error>> { self.init() }
    fn ui_component(&self) -> &str { "ui/plugin/emoji.slint" }
}
