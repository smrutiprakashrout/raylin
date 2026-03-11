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
}

fn resolve_icon_path(icon_name: &str) -> String {
    if icon_name.is_empty() {
        return "".to_string();
    }
    
    // If it's already an absolute path (e.g. flatpak overriding), return it
    if icon_name.starts_with('/') {
        if Path::new(icon_name).exists() {
            return icon_name.to_string();
        }
    }

    let search_paths = vec![
        "/usr/share/pixmaps",
        "/usr/share/icons/hicolor/48x48/apps",
        "/usr/share/icons/hicolor/64x64/apps",
        "/usr/share/icons/hicolor/128x128/apps",
        "/usr/share/icons/hicolor/256x256/apps",
        "/usr/share/icons/hicolor/scalable/apps",
        "/var/lib/flatpak/exports/share/icons/hicolor/48x48/apps",
        "/var/lib/flatpak/exports/share/icons/hicolor/64x64/apps",
        "/var/lib/flatpak/exports/share/icons/hicolor/scalable/apps",
    ];

    let extensions = [".png", ".svg", ".xpm"];

    for dir in search_paths {
        for ext in extensions {
            let path = format!("{}/{}{}", dir, icon_name, ext);
            if Path::new(&path).exists() {
                return path;
            }
        }
    }

    "".to_string()
}

fn scan_desktop_files() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();

    let search_paths = vec![
        "/usr/share/applications".to_string(),
        format!("{}/.local/share/applications", home),
        "/var/lib/flatpak/exports/share/applications".to_string(),
        format!("{}/.local/share/flatpak/exports/share/applications", home),
    ];

    for path in search_paths {
        if !Path::new(&path).exists() {
            continue;
        }
        for entry in WalkDir::new(&path).into_iter().filter_map(Result::ok) {
            if entry.path().extension().map_or(false, |ext| ext == "desktop") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let path_buf = entry.path().to_path_buf();
                    // Provide a strongly typed empty locales filter Option<&[String]>
                    let no_locales: Option<&[String]> = None;
                    if let Ok(desktop_entry) = DesktopEntry::from_str(path_buf, &content, no_locales) {
                        // Skip if NoDisplay or Hidden
                        if desktop_entry.no_display() || desktop_entry.hidden() {
                            continue;
                        }

                        let empty_locales: &[String] = &[];
                        
                        // Strict strict Name parsing: search the file content directly for the actual `Name=` or `Name[en_US]=` key.
                        // We do this because the `freedesktop-desktop-entry` crate sometimes falls back to the file name which breaks the UI requirement.
                        let mut exact_name = String::new();
                        for line in content.lines() {
                            if line.starts_with("Name=") {
                                exact_name = line.trim_start_matches("Name=").to_string();
                                break;
                            } else if line.starts_with("Name[en_US]=") {
                                exact_name = line.trim_start_matches("Name[en_US]=").to_string();
                            } else if exact_name.is_empty() && line.starts_with("Name[en]=") {
                                exact_name = line.trim_start_matches("Name[en]=").to_string();
                            }
                        }

                        // Fallback purely to crate string ONLY if absolutely necessary and strip extensions
                        let mut pretty_name = if !exact_name.is_empty() {
                            exact_name
                        } else {
                            desktop_entry.name(empty_locales).map(|c| c.to_string()).unwrap_or_else(|| "Unknown".to_string())
                        };
                        
                        // Sanitize weird edge cases if freedesktop-crate returns the .desktop extension
                        if pretty_name.ends_with(".desktop") {
                            pretty_name = pretty_name.replace(".desktop", "");
                        }

                        let mut exec = desktop_entry.exec().unwrap_or("").to_string();
                        let icon_raw = desktop_entry.icon().unwrap_or("application-x-executable").to_string();
                        let icon_path = resolve_icon_path(&icon_raw);

                        // Clean up exec string from %f, %U, etc.
                        exec = exec.replace("%f", "").replace("%F", "").replace("%u", "").replace("%U", "").trim().to_string();
                        
                        if !pretty_name.is_empty() && !exec.is_empty() {
                            apps.push(AppEntry {
                                name: pretty_name,
                                exec,
                                icon_path,
                            });
                        }
                    }
                }
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
                    let slint_image = if !item.data.icon_path.is_empty() {
                        if let Ok(image) = slint::Image::load_from_path(std::path::Path::new(&item.data.icon_path)) { image } else { slint::Image::default() }
                    } else { slint::Image::default() };
                    results.push(SearchResult { name: item.data.name.clone().into(), exec: item.data.exec.clone().into(), icon: slint_image });
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
            // Build a flat list from the LIVE filtered model in Slint.
            // Each entry: (orig_index, cat_idx, local_row, local_col)
            // We use flat-list POSITION (0..N) for navigation, not orig_index.
            let mut flat: Vec<(i32, usize, i32, i32)> = Vec::new();
            let cats = ui.get_emoji_categories();
            for i in 0..cats.row_count() {
                if let Some(cat) = cats.row_data(i) {
                    for j in 0..cat.emojis.row_count() {
                        if let Some(e) = cat.emojis.row_data(j) {
                            flat.push((e.orig_index, i, e.row, e.col));
                        }
                    }
                }
            }

            if flat.is_empty() { return; }

            let current_orig = ui.get_selected_index() as i32;

            // Find the flat-list POSITION of the currently selected emoji.
            // If it's not found (selection is off-screen after a filter), snap to 0.
            let current_pos = flat.iter().position(|(oi, _, _, _)| *oi == current_orig)
                .unwrap_or(0);

            let (_, cur_cat, cur_row, cur_col) = flat[current_pos];

            let next_pos = match dir {
                // Left/Right: simple flat-list step, clamped to [0, N-1]
                MoveDir::Left => current_pos.saturating_sub(1),
                MoveDir::Right => (current_pos + 1).min(flat.len() - 1),

                MoveDir::Up | MoveDir::Down => {
                    let going_up = matches!(dir, MoveDir::Up);
                    let target_local_row = if going_up { cur_row - 1 } else { cur_row + 1 };

                    // Try to find a neighbor at target_local_row within the SAME category
                    let same_cat_target: Option<usize> = flat
                        .iter()
                        .enumerate()
                        .filter(|(_, (_, ci, r, _))| *ci == cur_cat && *r == target_local_row)
                        .min_by_key(|(_, (_, _, _, c))| (c - cur_col).abs())
                        .map(|(pos, _)| pos);

                    if let Some(pos) = same_cat_target {
                        pos
                    } else if going_up {
                        // Jump to the LAST row of the previous category, same column preference
                        let prev_cat = (0..cur_cat).rev()
                            .find(|&c| flat.iter().any(|(_, ci, _, _)| *ci == c));
                        if let Some(pc) = prev_cat {
                            let max_row = flat.iter()
                                .filter(|(_, ci, _, _)| *ci == pc)
                                .map(|(_, _, r, _)| *r)
                                .max()
                                .unwrap_or(0);
                            flat.iter().enumerate()
                                .filter(|(_, (_, ci, r, _))| *ci == pc && *r == max_row)
                                .min_by_key(|(_, (_, _, _, c))| (c - cur_col).abs())
                                .map(|(pos, _)| pos)
                                .unwrap_or(current_pos)
                        } else {
                            current_pos // already at the very first row
                        }
                    } else {
                        // Jump to the FIRST row of the next category, same column preference
                        let next_cat = (cur_cat + 1..cats.row_count())
                            .find(|&c| flat.iter().any(|(_, ci, _, _)| *ci == c));
                        if let Some(nc) = next_cat {
                            flat.iter().enumerate()
                                .filter(|(_, (_, ci, r, _))| *ci == nc && *r == 0)
                                .min_by_key(|(_, (_, _, _, c))| (c - cur_col).abs())
                                .map(|(pos, _)| pos)
                                .unwrap_or(current_pos)
                        } else {
                            current_pos // already at the very last row
                        }
                    }
                }
            };

            let (next_orig, next_cat, next_local_row, _) = flat[next_pos];

            // Update the Slint selection
            ui.set_selected_index(next_orig);

            // Scroll-to-visible: calculate absolute y of the target tile
            let mut y_offset: i32 = 0;
            for i in 0..cats.row_count() {
                if i == next_cat { break; }
                if let Some(cat) = cats.row_data(i) {
                    if cat.emojis.row_count() > 0 {
                        let rows = (cat.emojis.row_count() as f32 / 7.0).ceil() as i32;
                        y_offset += 30 + (rows * 85) + 16; // header + rows + gap
                    }
                }
            }
            y_offset += 30;                  // own category header
            y_offset += next_local_row * 85; // row within own category
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
