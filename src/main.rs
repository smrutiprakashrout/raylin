use freedesktop_desktop_entry::DesktopEntry;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};
use nucleo::{Config, Nucleo};
use rusqlite::{Connection, Result as SqlResult};
use slint::{ComponentHandle, Model, SharedPixelBuffer, VecModel, Image};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use walkdir::WalkDir;

slint::include_modules!();

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

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    // 1. Initialize Slint UI
    let ui = AppWindow::new()?;

    // 2. Initialize background processes
    // Global hotkey
    let manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space);
    manager.register(hotkey).unwrap();

    // Clipboard DB
    let _conn = setup_clipboard_db().map_err(|e| Box::new(e) as Box<dyn Error>)?;

    // 3. App Indexer using Nucleo
    println!("Scanning applications...");
    let apps = scan_desktop_files();
    
    let mut nucleo = Nucleo::<AppEntry>::new(
        Config::DEFAULT,
        std::sync::Arc::new(|| {}),
        None, // num_threads
        1,    // columns (0: name)
    );

    // Inject items into nucleo matcher
    let injector = nucleo.injector();
    for app in &apps {
        injector.push(app.clone(), |app, columns| {
            // Index the name
            columns[0] = app.name.clone().into();
        });
    }
    
    // Wait for initial injestion
    while nucleo.tick(10).running { }

    let ui_handle = ui.as_weak();
    
    // 4. Input Text Changed Logic
    ui.on_text_changed(move |text| {
        let ui = ui_handle.unwrap();
        let query = text.as_str();

        // Perform search
        nucleo.pattern.reparse(
            0,
            query,
            nucleo::pattern::CaseMatching::Ignore,
            nucleo::pattern::Normalization::Smart,
            false
        );

        // Tick to process the search
        while nucleo.tick(10).running { }

        let snapshot = nucleo.snapshot();
        let mut results = Vec::new();
        
        let count = snapshot.matched_item_count().min(8);
        for i in 0..count {
            if let Some(item) = snapshot.get_matched_item(i) {
                // If the app has an absolute path we found, load it into an image
                let slint_image = if !item.data.icon_path.is_empty() {
                    let path = std::path::Path::new(&item.data.icon_path);
                    if let Ok(image) = slint::Image::load_from_path(path) {
                        image
                    } else {
                        slint::Image::default()
                    }
                } else {
                    slint::Image::default()
                };

                results.push(SearchResult {
                    name: item.data.name.clone().into(),
                    exec: item.data.exec.clone().into(),
                    icon: slint_image,
                });
            }
        }

        let model = Rc::new(VecModel::from(results));
        ui.set_results(model.into());
        ui.set_selected_index(0); // Reset selection
    });
    
    let ui_handle_nav = ui.as_weak();
    ui.on_next_item(move || {
        let ui = ui_handle_nav.unwrap();
        let results = ui.get_results();
        let max = results.row_count().saturating_sub(1) as i32;
        let current = ui.get_selected_index();
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
        let results = ui.get_results();
        let idx = ui.get_selected_index() as usize;
        
        if let Some(item) = results.row_data(idx) {
            println!("Executing: {}", item.exec);
            
            let mut clean_exec = item.exec.to_string();
            // Clean up exec string from field codes like %u, %f before passing it to the OS
            let codes = ["%f", "%F", "%u", "%U", "%c", "%k", "%i", "%m"];
            for code in codes {
                clean_exec = clean_exec.replace(code, "");
            }
            clean_exec = clean_exec.trim().to_string();

            // Spawn the application process
            if let Err(e) = std::process::Command::new("sh")
                .arg("-c")
                .arg(&clean_exec)
                .spawn()
            {
                eprintln!("Failed to spawn {}: {}", clean_exec, e);
            }
            
            // Exit the launcher after executing
            std::process::exit(0);
        }
    });
    
    // Spawn hotkey listener
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();
    tokio::spawn(async move {
        loop {
            if let Ok(event) = global_hotkey_channel.try_recv() {
                if event.id == hotkey.id() {
                    println!("Hotkey pressed!");
                    // Toggle visibility logic here
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
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
