use std::error::Error;
use std::path::Path;

use nucleo::{Config, Nucleo};
use rusqlite::Connection;

use super::Plugin;

// ──────────────────────────────────────────────────────────────────────────────
// ClipboardPlugin
// ──────────────────────────────────────────────────────────────────────────────

pub struct ClipboardPlugin {
    items: Vec<String>,
    nucleo: Option<Nucleo<String>>,
    db_path: String,
    max_items: usize,
}

impl ClipboardPlugin {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            items: Vec::new(),
            nucleo: None,
            db_path: format!("{}/.config/wayland-palette/clipboard.sqlite", home),
            max_items: 50,
        }
    }

    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items = max;
        self
    }

    // ── Initialization ───────────────────────────────────────────────────────

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.init_database()?;
        self.items = self.load_history()?;

        let mut nucleo = Nucleo::<String>::new(
            Config::DEFAULT,
            std::sync::Arc::new(|| {}),
            None,
            1,
        );
        let injector = nucleo.injector();
        for item in &self.items {
            let s = item.clone();
            injector.push(s.clone(), |c_item, col| { col[0] = c_item.clone().into(); });
        }
        while nucleo.tick(10).running {}

        self.nucleo = Some(nucleo);
        Ok(())
    }

    // ── Database ─────────────────────────────────────────────────────────────

    fn init_database(&self) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = Path::new(&self.db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id        INTEGER PRIMARY KEY,
                content   TEXT NOT NULL,
                timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                pinned    BOOLEAN DEFAULT 0
            )",
            [],
        )?;
        Ok(())
    }

    fn load_history(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT content FROM clipboard_history ORDER BY timestamp DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([self.max_items as i64], |row| row.get::<_, String>(0))?;
        let mut items = Vec::new();
        for row in rows.flatten() {
            items.push(row);
        }
        Ok(items)
    }

    pub fn add_item(&mut self, content: String) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open(&self.db_path)?;
        // Skip duplicates — delete old occurrence first so it resurfaces at top
        conn.execute("DELETE FROM clipboard_history WHERE content = ?1", [&content])?;
        conn.execute(
            "INSERT INTO clipboard_history (content, timestamp) VALUES (?1, strftime('%s', 'now'))",
            [&content],
        )?;
        // Trim to max_items
        conn.execute(
            "DELETE FROM clipboard_history WHERE id NOT IN (
                SELECT id FROM clipboard_history ORDER BY timestamp DESC LIMIT ?1
            )",
            [self.max_items as i64],
        )?;
        // Prepend to in-memory list and rebuild index
        self.items.retain(|i| i != &content);
        self.items.insert(0, content);
        self.rebuild_index();
        Ok(())
    }

    fn rebuild_index(&mut self) {
        let mut nucleo = Nucleo::<String>::new(
            Config::DEFAULT,
            std::sync::Arc::new(|| {}),
            None,
            1,
        );
        let injector = nucleo.injector();
        for item in &self.items {
            let s = item.clone();
            injector.push(s, |c_item, col| { col[0] = c_item.clone().into(); });
        }
        while nucleo.tick(10).running {}
        self.nucleo = Some(nucleo);
    }

    // ── Search ───────────────────────────────────────────────────────────────

    pub fn search(&mut self, query: &str, limit: usize) -> Vec<String> {
        let nucleo = match &mut self.nucleo {
            Some(n) => n,
            None => return Vec::new(),
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
        (0..count)
            .filter_map(|i| snapshot.get_matched_item(i).map(|item| item.data.clone()))
            .collect()
    }

    pub fn get_items(&self) -> &[String] {
        &self.items
    }

    // ── Clipboard ────────────────────────────────────────────────────────────

    /// Copy content to clipboard (Wayland-first, X11 fallback).
    pub fn copy_item(content: &str) {
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        if wayland {
            let _ = std::process::Command::new("wl-copy").arg(content).spawn();
        } else {
            let _ = std::process::Command::new("xclip")
                .args(["-selection", "clipboard"])
                .arg(content)
                .spawn();
        }
    }
}

impl Default for ClipboardPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// ── Plugin Trait ─────────────────────────────────────────────────────────────

impl Plugin for ClipboardPlugin {
    fn id(&self) -> &str { "clipboard_history" }
    fn name(&self) -> &str { "Clipboard History" }
    fn trigger(&self) -> &str { "@c" }
    fn aliases(&self) -> Vec<&str> { vec![":clip"] }
    fn init(&mut self) -> Result<(), Box<dyn Error>> { self.init() }
    fn ui_component(&self) -> &str { "ui/plugin/clipboard_history.slint" }
}
