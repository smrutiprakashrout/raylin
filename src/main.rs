use freedesktop_desktop_entry::DesktopEntry;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};
use nucleo::{Config, Nucleo};
use rusqlite::{Connection, Result as SqlResult};
use serde::Deserialize;
use slint::{ComponentHandle, Model, SharedPixelBuffer, VecModel, Image};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use walkdir::WalkDir;

slint::include_modules!();

#[derive(Deserialize, Debug, Clone)]
pub struct EmojiRawData {
    pub name: String,
    pub group: String,
}

#[derive(Debug, Clone)]
pub struct EmojiData {
    pub character: String,
    pub name: String,
    pub group: String,
}

/// App data model
#[derive(Clone, Debug)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon_path: String,
    pub type_label: String,  // "Application" or "Command Line"
}

/// Freedesktop-compliant icon lookup.
///
/// Search strategy (first match wins):
///   1. Absolute path  — used directly if it exists
///   2. Priority dirs  — specific size/scalable dirs checked without walking (fast path)
///   3. Theme walk     — WalkDir over icon roots, filtered to `apps/` subdirs (catches
///                       custom themes, extra sizes, Flatpak per-user installs)
///   4. Snap           — /snap/<name>/current/meta/gui/icon.*
///
/// Extension priority per directory: svg → png → jpg → ico → xpm
/// (.xpm noted as unsupported by Slint; loader will skip it, but we still surface the path
///  so the caller can decide.)
///
/// Supports reverse-DNS Flatpak names like `org.localsend.localsend_app`.
pub fn find_icon_path(icon_name: &str) -> Option<std::path::PathBuf> {
    use std::path::PathBuf;

    if icon_name.is_empty() { return None; }

    // ── 1. Absolute path ────────────────────────────────────────────────────
    if icon_name.starts_with('/') {
        let p = PathBuf::from(icon_name);
        return if p.exists() { Some(p) } else { None };
    }

    let home = std::env::var("HOME").unwrap_or_default();

    // Extension search order — SVG first (vector, scales perfectly), xpm last (unsupported by Slint)
    let exts: &[&str] = &[".svg", ".png", ".jpg", ".ico", ".xpm", ""];

    // ── 2. Priority dirs (fast path — covers 95% of real-world installs) ────
    //    Listed from most preferred (largest / scalable) to least preferred.
    let priority_dirs: Vec<PathBuf> = vec![
        // User local theme
        PathBuf::from(format!("{}/.local/share/icons/hicolor/scalable/apps", home)),
        PathBuf::from(format!("{}/.local/share/icons/hicolor/256x256/apps",  home)),
        PathBuf::from(format!("{}/.local/share/icons/hicolor/128x128/apps",  home)),
        PathBuf::from(format!("{}/.local/share/icons/hicolor/64x64/apps",    home)),
        PathBuf::from(format!("{}/.local/share/icons/hicolor/48x48/apps",    home)),
        PathBuf::from(format!("{}/.local/share/pixmaps",                     home)),
        // System hicolor
        PathBuf::from("/usr/share/icons/hicolor/scalable/apps"),
        PathBuf::from("/usr/share/icons/hicolor/256x256/apps"),
        PathBuf::from("/usr/share/icons/hicolor/128x128/apps"),
        PathBuf::from("/usr/share/icons/hicolor/64x64/apps"),
        PathBuf::from("/usr/share/icons/hicolor/48x48/apps"),
        PathBuf::from("/usr/share/icons/hicolor/32x32/apps"),
        PathBuf::from("/usr/share/pixmaps"),
        // Flatpak system exports
        PathBuf::from("/var/lib/flatpak/exports/share/icons/hicolor/scalable/apps"),
        PathBuf::from("/var/lib/flatpak/exports/share/icons/hicolor/256x256/apps"),
        PathBuf::from("/var/lib/flatpak/exports/share/icons/hicolor/128x128/apps"),
        PathBuf::from("/var/lib/flatpak/exports/share/icons/hicolor/48x48/apps"),
        // Flatpak user exports
        PathBuf::from(format!("{}/.local/share/flatpak/exports/share/icons/hicolor/scalable/apps", home)),
        PathBuf::from(format!("{}/.local/share/flatpak/exports/share/icons/hicolor/256x256/apps", home)),
        PathBuf::from(format!("{}/.local/share/flatpak/exports/share/icons/hicolor/128x128/apps", home)),
    ];

    for dir in &priority_dirs {
        if !dir.exists() { continue; }
        for ext in exts {
            let candidate = dir.join(format!("{}{}", icon_name, ext));
            if candidate.exists() { return Some(candidate); }
        }
    }

    // ── 3. Theme walk — catches arbitrary themes and non-standard sizes ──────
    //    Only descend into directories whose path contains "/apps" segment.
    let walk_roots: Vec<PathBuf> = vec![
        PathBuf::from(format!("{}/.local/share/icons", home)),
        PathBuf::from("/usr/share/icons"),
        PathBuf::from(format!("{}/.local/share/flatpak/exports/share/icons", home)),
        PathBuf::from("/var/lib/flatpak/exports/share/icons"),
    ];

    for root in &walk_roots {
        if !root.exists() { continue; }
        for entry in WalkDir::new(root)
            .min_depth(3)   // skip theme root and size-root dirs themselves
            .max_depth(5)   // theme/size/category — cap depth for performance
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
            // Only look in `apps/` subdirectories (the Freedesktop spec location)
            .filter(|e| e.path().components().any(|c| c.as_os_str() == "apps"))
        {
            for ext in exts {
                let candidate = entry.path().join(format!("{}{}", icon_name, ext));
                if candidate.exists() { return Some(candidate); }
            }
        }
    }

    // ── 4. Snap ─────────────────────────────────────────────────────────────
    //    icon_name usually matches the snap package name
    for ext in [".png", ".svg"] {
        let p = PathBuf::from(format!("/snap/{}/current/meta/gui/icon{}", icon_name, ext));
        if p.exists() { return Some(p); }
    }

    // ── 5. Reverse-DNS alias (Flatpak apps like org.app.Name) ───────────────
    //    Try the last component as a shorter alias, e.g. "localsend_app" from
    //    "org.localsend.localsend_app"
    if icon_name.contains('.') {
        if let Some(short_name) = icon_name.split('.').last() {
            if short_name != icon_name {
                return find_icon_path(short_name);
            }
        }
    }

    None
}

/// Thin wrapper: returns an empty String when no icon is found (keeps existing call sites).
fn resolve_icon_path(icon_name: &str) -> String {
    find_icon_path(icon_name)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Returns true if the binary named by `exec` is findable on PATH or as an absolute path.
fn binary_exists(exec: &str) -> bool {
    if exec.is_empty() { return false; }
    // Absolute path
    if exec.starts_with('/') {
        return Path::new(exec).exists();
    }
    // Search PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            if Path::new(&format!("{}/{}", dir, exec)).exists() {
                return true;
            }
        }
    }
    false
}

/// Extract the bare binary name from an Exec= value.
/// Strips: env VAR=val prefixes, %f/%u/etc field codes, surrounding quotes.
fn parse_exec_binary(exec_raw: &str) -> &str {
    let mut s = exec_raw.trim();

    // Skip "env" command and VAR=value assignments (e.g. "env FOO=bar /usr/bin/app")
    loop {
        if s.starts_with("env ") { s = s[4..].trim(); continue; }
        if s.contains('=') && !s.contains(' ') { return ""; } // lone VAR=val, no binary
        let first = s.split_whitespace().next().unwrap_or("");
        if first.contains('=') { s = &s[first.len()..].trim(); continue; }
        break;
    }

    // Take just the first token (the binary name/path)
    let binary = s.split_whitespace().next().unwrap_or("");
    // Strip surrounding quotes
    let binary = binary.trim_matches('"').trim_matches('\'');
    binary
}

fn scan_desktop_files() -> Vec<AppEntry> {
    let mut apps: Vec<AppEntry> = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();

    let search_paths = vec![
        "/usr/share/applications".to_string(),
        format!("{}/.local/share/applications", home),
        // Flatpak
        "/var/lib/flatpak/exports/share/applications".to_string(),
        format!("{}/.local/share/flatpak/exports/share/applications", home),
        // Snap
        "/var/lib/snapd/desktop/applications".to_string(),
    ];

    for search_path in search_paths {
        if !Path::new(&search_path).exists() { continue; }

        for entry in WalkDir::new(&search_path).into_iter().filter_map(Result::ok) {
            if !entry.path().extension().map_or(false, |e| e == "desktop") { continue; }

            let Ok(content) = fs::read_to_string(entry.path()) else { continue };
            let path_buf = entry.path().to_path_buf();
            let no_locales: Option<&[String]> = None;
            let Ok(de) = DesktopEntry::from_str(path_buf, &content, no_locales) else { continue };

            // ── Visibility filters ──────────────────────────────────────────
            if de.no_display() || de.hidden() { continue; }

            // OnlyShowIn: skip if present and doesn't include common DEs
            // (freedesktop crate exposes this as a raw string)
            let only_show = content.lines()
                .find(|l| l.starts_with("OnlyShowIn="))
                .map(|l| l.trim_start_matches("OnlyShowIn="));
            if let Some(only) = only_show {
                // Accept if any of our preferred environments are listed
                let accepted = ["GNOME", "KDE", "sway", "Hyprland", "wlroots", "X-"];
                if !accepted.iter().any(|de_name| only.contains(de_name)) {
                    continue;
                }
            }

            // ── Name ────────────────────────────────────────────────────────
            let mut name = String::new();
            for line in content.lines() {
                if line.starts_with("Name=") {
                    name = line.trim_start_matches("Name=").to_string();
                    break;
                } else if line.starts_with("Name[en_US]=") && name.is_empty() {
                    name = line.trim_start_matches("Name[en_US]=").to_string();
                } else if line.starts_with("Name[en]=") && name.is_empty() {
                    name = line.trim_start_matches("Name[en]=").to_string();
                }
            }
            if name.is_empty() {
                name = de.name(&[] as &[String])
                    .map(|c| c.to_string())
                    .unwrap_or_default();
            }
            name = name.replace(".desktop", "");
            if name.is_empty() { continue; }

            // ── Exec validation ─────────────────────────────────────────────
            let exec_raw = de.exec().unwrap_or("").to_string();
            let exec_clean = exec_raw
                .replace("%f", "").replace("%F", "")
                .replace("%u", "").replace("%U", "")
                .replace("%d", "").replace("%D", "")
                .replace("%n", "").replace("%N", "")
                .replace("%i", "").replace("%c", "")
                .replace("%k", "").replace("%v", "").replace("%m", "")
                .trim().to_string();

            if exec_clean.is_empty() { continue; }

            let binary = parse_exec_binary(&exec_clean);
            if binary.is_empty() || !binary_exists(binary) { continue; }

            // ── Terminal / type classification ────────────────────────────
            let is_terminal = content.lines()
                .find(|l| l.starts_with("Terminal="))
                .map(|l| l.trim_start_matches("Terminal=").trim().eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let type_label = if is_terminal { "Command Line" } else { "Application" }.to_string();

            // ── Icon ─────────────────────────────────────────────────────────
            let icon_raw = de.icon().unwrap_or("application-x-executable").to_string();
            let icon_path = resolve_icon_path(&icon_raw);

            // ── Deduplication by name (first occurrence / more specific wins) ─
            if !apps.iter().any(|a| a.name.eq_ignore_ascii_case(&name)) {
                apps.push(AppEntry { name, exec: exec_clean, icon_path, type_label });
            }
        }
    }

    apps
}

fn setup_clipboard_db() -> SqlResult<Connection> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.config/wayland-palette/clipboard.sqlite", home);
    
    if let Some(parent) = Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(1), Some(e.to_string())))?;
    }

    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS clipboard_history (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(conn)
}

fn load_emojis() -> indexmap::IndexMap<String, Vec<EmojiData>> {
    let emoji_str = include_str!("../emojis.json");
    // Parse to preserve order of outer array, even though it's technically a list of objects with one key
    let parsed: Vec<indexmap::IndexMap<String, EmojiRawData>> = serde_json::from_str(emoji_str).unwrap_or_else(|_e| {
        println!("Error parsing emojis: {:?}", _e);
        vec![]
    });
    
    // We use an IndexMap to preserve the exact category order found in the JSON
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

fn get_clipboard_history(conn: &Connection) -> Vec<String> {
    let mut stmt = conn.prepare("SELECT content FROM clipboard_history ORDER BY timestamp DESC LIMIT 50").unwrap_or_else(|_| panic!("Failed to prepare statement"));
    let mut rows = stmt.query([]).unwrap();
    let mut history = Vec::new();
    while let Ok(Some(row)) = rows.next() {
        let content: String = row.get(0).unwrap();
        history.push(content);
    }
    history
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    // 1. Initialize Slint UI
    let ui = AppWindow::new()?;

    // 2. Initialize background processes
    // Global hotkeys
    let manager = GlobalHotKeyManager::new().unwrap();
    let hotkey_main = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space);
    let hotkey_emoji = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyE);
    let hotkey_clip = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);
    
    manager.register(hotkey_main).unwrap();
    manager.register(hotkey_emoji).unwrap();
    manager.register(hotkey_clip).unwrap();

    // Clipboard DB & Emojis
    let conn = setup_clipboard_db().map_err(|e| Box::new(e) as Box<dyn Error>)?;
    let clip_history = get_clipboard_history(&conn);
    let emoji_db = load_emojis();

    // 3. Nucleo Indexers
    println!("Scanning applications...");
    let apps = scan_desktop_files();
    
    let mut nucleo_apps = Nucleo::<AppEntry>::new(Config::DEFAULT, std::sync::Arc::new(|| {}), None, 1);
    let mut nucleo_emojis = Nucleo::<EmojiData>::new(Config::DEFAULT, std::sync::Arc::new(|| {}), None, 1);
    let mut nucleo_clip = Nucleo::<String>::new(Config::DEFAULT, std::sync::Arc::new(|| {}), None, 1);

    // Inject data
    let inj_apps = nucleo_apps.injector();
    for app in &apps {
        inj_apps.push(app.clone(), |a, c| { c[0] = a.name.clone().into(); });
    }
    
    let inj_emojis = nucleo_emojis.injector();
    let mut flat_index = 0;
    // We inject emojis in the exact order parsed to maintain original group sequence during blank searches
    for (group_name, emojis_in_group) in &emoji_db {
        for e in emojis_in_group {
            inj_emojis.push(e.clone(), |e_item, c| { c[0] = e_item.name.clone().into(); });
            flat_index += 1;
        }
    }

    let inj_clip = nucleo_clip.injector();
    for c in &clip_history {
        inj_clip.push(c.clone(), |c_item, col| { col[0] = c_item.clone().into(); });
    }
    
    // Wait for initial injestion
    while nucleo_apps.tick(10).running { }
    while nucleo_emojis.tick(10).running { }
    while nucleo_clip.tick(10).running { }

    // Navigation Context Handling
    let ui_handle_pop = ui.as_weak();
    ui.on_pop_mode(move || {
        let ui = ui_handle_pop.unwrap();
        ui.set_active_mode(AppMode::Root);
        ui.set_search_text("".into());
        ui.invoke_text_changed("".into());
        ui.invoke_update_scroll(0.0);
    });

    let ui_handle = ui.as_weak();
    
    // 4. Input Text Changed Logic
    ui.on_text_changed(move |text| {
        let ui = ui_handle.unwrap();
        let mut query = text.as_str();

        // Check Hard Prefixes if in root AppSearch mode
        if ui.get_active_mode() == AppMode::Root {
            if query == ":e" || query.starts_with(":e ") {
                ui.set_active_mode(AppMode::Emoji);
                ui.set_search_text("".into()); // Strip prefix
                query = "";
            } else if query == ":c" || query.starts_with(":c ") {
                ui.set_active_mode(AppMode::Clipboard);
                ui.set_search_text("".into()); // Strip prefix
                query = "";
            }
        }

        let mode = ui.get_active_mode();

        if mode == AppMode::Root {
            nucleo_apps.pattern.reparse(0, query, nucleo::pattern::CaseMatching::Ignore, nucleo::pattern::Normalization::Smart, false);
            while nucleo_apps.tick(10).running { }
            let snapshot = nucleo_apps.snapshot();
            let mut results = Vec::new();
            
            let count = snapshot.matched_item_count().min(8);
            for i in 0..count {
                if let Some(item) = snapshot.get_matched_item(i) {
                    // Safely load the icon, skipping .xpm (Slint backends don't support it).
                    // Any other load failure falls back to a blank default icon.
                    let slint_image = {
                        let path = &item.data.icon_path;
                        let is_xpm = path.ends_with(".xpm");
                        if !path.is_empty() && !is_xpm {
                            match slint::Image::load_from_path(std::path::Path::new(path)) {
                                Ok(img) => img,
                                Err(_) => slint::Image::default(), // corrupted / unsupported format
                            }
                        } else {
                            slint::Image::default() // .xpm or no icon
                        }
                    };
                    results.push(SearchResult {
                        name: item.data.name.clone().into(),
                        exec: item.data.exec.clone().into(),
                        icon: slint_image,
                        type_label: item.data.type_label.clone().into(),
                    });
                }
            }
            ui.set_results(Rc::new(VecModel::from(results)).into());
            
        } else if mode == AppMode::Emoji {
            nucleo_emojis.pattern.reparse(0, query, nucleo::pattern::CaseMatching::Ignore, nucleo::pattern::Normalization::Smart, false);
            while nucleo_emojis.tick(10).running { }
            let snapshot = nucleo_emojis.snapshot();
            let count = snapshot.matched_item_count().min(3000); // show full emoji dataset
            
            // Preserve insertion order of categories matching the JSON original order
            let mut categories_map: indexmap::IndexMap<String, Vec<EmojiResult>> = indexmap::IndexMap::new();
            
            // If the query is empty, we must pre-populate the map with ALL original category keys
            // so that the exact JSON order is strictly maintained.
            if query.is_empty() {
                for key in emoji_db.keys() {
                    categories_map.insert(key.clone(), Vec::new());
                }
            }

            let mut flat_index = 0;
            for i in 0..count {
                if let Some(item) = snapshot.get_matched_item(i) {
                    let group_name = item.data.group.clone();
                    
                    // If not pre-populated, this will insert it in the order found by Nucleo.
                    // For blank queries, it perfectly drops into the pre-populated ordered keys.
                    let group_list = categories_map.entry(group_name).or_insert(Vec::new());
                    
                    group_list.push(EmojiResult { 
                        character: item.data.character.clone().into(), 
                        name: item.data.name.clone().into(),
                        row: (group_list.len() / 7) as i32,
                        col: (group_list.len() % 7) as i32,
                        orig_index: flat_index,
                    });
                    flat_index += 1;
                }
            }
            
            // Filter out empty categories (important when searching)
            let mut categories_map: indexmap::IndexMap<String, Vec<EmojiResult>> = categories_map
                .into_iter()
                .filter(|(_, items)| !items.is_empty())
                .collect();
            
            let mut categories = Vec::new();
            for (name, emojis) in categories_map {
                let name_str: String = name;
                categories.push(EmojiCategory {
                    name: name_str.into(),
                    count: emojis.len() as i32,
                    emojis: Rc::new(VecModel::from(emojis)).into(),
                });
            }
            ui.set_emoji_categories(Rc::new(VecModel::from(categories)).into());
            if query.is_empty() {
                ui.set_selected_index(0); // Only reset selection if this is an explicit root search, not general typing
                ui.invoke_update_scroll(0.0); // Snap back to top
            }

        } else if mode == AppMode::Clipboard {
            nucleo_clip.pattern.reparse(0, query, nucleo::pattern::CaseMatching::Ignore, nucleo::pattern::Normalization::Smart, false);
            while nucleo_clip.tick(10).running { }
            let snapshot = nucleo_clip.snapshot();
            let mut results = Vec::new();
            
            let count = snapshot.matched_item_count().min(12);
            for i in 0..count {
                if let Some(item) = snapshot.get_matched_item(i) {
                    results.push(ClipboardResult { content: item.data.clone().into() });
                }
            }
            ui.set_clipboard_results(Rc::new(VecModel::from(results)).into());
        }

        if mode == AppMode::Root || mode == AppMode::Clipboard {
            ui.set_selected_index(0); // Reset selection
        }
    });
    enum MoveDir { Up, Down, Left, Right }
    let move_focus = |ui_weak: &slint::Weak<AppWindow>, dir: MoveDir| {
        if let Some(ui) = ui_weak.upgrade() {
            let cats = ui.get_emoji_categories();
            if cats.row_count() == 0 { return; }

            // Only the ACTIVE GROUP is visible — navigate within it.
            let group_idx = ui.get_current_group_index() as usize;
            let active_cat = match cats.row_data(group_idx) {
                Some(c) => c,
                None => return,
            };

            // Build flat list: (orig_index, local_row, local_col)
            let mut flat: Vec<(i32, i32, i32)> = Vec::new();
            for j in 0..active_cat.emojis.row_count() {
                if let Some(e) = active_cat.emojis.row_data(j) {
                    flat.push((e.orig_index, e.row, e.col));
                }
            }
            if flat.is_empty() { return; }

            let current_orig = ui.get_selected_index() as i32;
            let current_pos = flat.iter().position(|(oi, _, _)| *oi == current_orig)
                .unwrap_or(0);

            let (_, cur_row, cur_col) = flat[current_pos];

            let next_pos = match dir {
                MoveDir::Left  => current_pos.saturating_sub(1),
                MoveDir::Right => (current_pos + 1).min(flat.len() - 1),
                MoveDir::Up | MoveDir::Down => {
                    let going_up = matches!(dir, MoveDir::Up);
                    let target_row = if going_up { cur_row - 1 } else { cur_row + 1 };

                    let row_match: Option<usize> = flat
                        .iter()
                        .enumerate()
                        .filter(|(_, (_, r, _))| *r == target_row)
                        .min_by_key(|(_, (_, _, c))| (c - cur_col).abs())
                        .map(|(pos, _)| pos);

                    row_match.unwrap_or(current_pos) // clamp at first/last row
                }
            };

            let (next_orig, next_local_row, _) = flat[next_pos];
            ui.set_selected_index(next_orig);

            // Scroll: within the single-category view only the row offset matters.
            // Badge header height ~38px + top padding 12px = 50px.
            let y_offset = 50 + next_local_row * 85;
            ui.invoke_update_scroll(y_offset as f32);
        }
    };

    let ui_handle_nav_up = ui.as_weak();
    ui.on_nav_up(move || move_focus(&ui_handle_nav_up, MoveDir::Up));

    let ui_handle_nav_down = ui.as_weak();
    ui.on_nav_down(move || move_focus(&ui_handle_nav_down, MoveDir::Down));

    let ui_handle_nav_left = ui.as_weak();
    ui.on_nav_left(move || move_focus(&ui_handle_nav_left, MoveDir::Left));

    let ui_handle_nav_right = ui.as_weak();
    ui.on_nav_right(move || move_focus(&ui_handle_nav_right, MoveDir::Right));

    // Ctrl+T: cycle to the next emoji category group with wraparound
    let ui_handle_cycle = ui.as_weak();
    ui.on_cycle_group(move || {
        let ui = ui_handle_cycle.unwrap();
        let total_groups = ui.get_emoji_categories().row_count() as i32;
        if total_groups == 0 { return; }
        let next_group = (ui.get_current_group_index() + 1) % total_groups;
        ui.set_current_group_index(next_group);
        ui.set_selected_index(0); // reset focus to first emoji in new group
        ui.invoke_update_scroll(0.0); // snap scroll to top
    });

    let ui_handle_nav = ui.as_weak();
    ui.on_next_item(move || {
        let ui = ui_handle_nav.unwrap();
        let current = ui.get_selected_index();
        let max = match ui.get_active_mode() {
            AppMode::Root => ui.get_results().row_count().saturating_sub(1) as i32,
            AppMode::Emoji => {
                let mut c = 0;
                let cats = ui.get_emoji_categories();
                for i in 0..cats.row_count() {
                    if let Some(cat) = cats.row_data(i) {
                        c += cat.emojis.row_count();
                    }
                }
                c.saturating_sub(1) as i32
            },
            AppMode::Clipboard => ui.get_clipboard_results().row_count().saturating_sub(1) as i32,
        };
        if current < max {
            ui.set_selected_index(current + 1);
        }
    });

    let ui_handle_prev = ui.as_weak();
    ui.on_prev_item(move || {
        let ui = ui_handle_prev.unwrap();
        let current = ui.get_selected_index();
        if current > 0 {
            ui.set_selected_index(current - 1);
        }
    });

    let ui_handle_exec = ui.as_weak();
    ui.on_execute_selected(move || {
        let ui = ui_handle_exec.unwrap();
        let mode = ui.get_active_mode();
        let idx = ui.get_selected_index() as usize;
        
        if mode == AppMode::Root {
            let results = ui.get_results();
            if let Some(item) = results.row_data(idx) {
                println!("Executing: {}", item.exec);
                let mut clean_exec = item.exec.to_string();
                let codes = ["%f", "%F", "%u", "%U", "%c", "%k", "%i", "%m"];
                for code in codes { clean_exec = clean_exec.replace(code, ""); }
                clean_exec = clean_exec.trim().to_string();

                if let Err(e) = std::process::Command::new("sh").arg("-c").arg(&clean_exec).spawn() {
                    eprintln!("Failed to spawn {}: {}", clean_exec, e);
                }
                std::process::exit(0);
            }
        } else if mode == AppMode::Emoji {
            let mut char_to_copy = None;
            let cats = ui.get_emoji_categories();
            for i in 0..cats.row_count() {
                if let Some(cat) = cats.row_data(i) {
                    for j in 0..cat.emojis.row_count() {
                        if let Some(e) = cat.emojis.row_data(j) {
                            if e.orig_index == idx as i32 {
                                char_to_copy = Some(e.character.to_string());
                            }
                        }
                    }
                }
            }
            if let Some(c) = char_to_copy {
                println!("Copying emoji: {}", c);
                let _ = std::process::Command::new("wl-copy").arg(c).spawn();
                std::process::exit(0);
            }
        } else if mode == AppMode::Clipboard {
            let results = ui.get_clipboard_results();
            if let Some(item) = results.row_data(idx) {
                println!("Copying clipboard item");
                let _ = std::process::Command::new("wl-copy").arg(&item.content.to_string()).spawn();
                std::process::exit(0);
            }
        }
    });
    
    // Spawn hotkey listener
    let ui_handle_hotkeys = ui.as_weak();
    let main_id = hotkey_main.id();
    let emoji_id = hotkey_emoji.id();
    let clip_id = hotkey_clip.id();

    let global_hotkey_channel = GlobalHotKeyEvent::receiver();
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = global_hotkey_channel.try_recv() {
                let ui_handle = ui_handle_hotkeys.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        if event.id == main_id {
                            ui.set_active_mode(AppMode::Root);
                            ui.set_search_text("".into());
                            ui.invoke_text_changed("".into());
                        } else if event.id == emoji_id {
                            ui.set_active_mode(AppMode::Emoji);
                            ui.set_search_text("".into());
                            ui.invoke_text_changed("".into());
                        } else if event.id == clip_id {
                            ui.set_active_mode(AppMode::Clipboard);
                            ui.set_search_text("".into());
                            ui.invoke_text_changed("".into());
                        }
                    }
                }).unwrap();
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    // Start Slint run loop
    println!("Starting Raycast AppWindow...");
    
    // Trigger initial search for empty string to populate default list
    let ui_handle_init = ui.as_weak();
    let init_timer = slint::Timer::default();
    init_timer.start(slint::TimerMode::SingleShot, std::time::Duration::from_millis(50), move || {
        if let Some(ui) = ui_handle_init.upgrade() {
            ui.invoke_text_changed("".into());
        }
    });

    ui.run().map_err(|e| Box::new(e) as Box<dyn Error>)?;
    Ok(())
}
