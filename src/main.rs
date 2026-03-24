mod core;
mod plugins;
mod utils;
mod models;

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};
use slint::{ComponentHandle, Model, VecModel};

use plugins::app_launcher::AppLauncherPlugin;
use plugins::emoji::EmojiPlugin;
use plugins::clipboard_history::ClipboardPlugin;

// All triggers/aliases live in core::shortcuts — edit there to change them.
use core::shortcuts::{TRIGGER_EMOJI, TRIGGER_CLIPBOARD, TRIGGER_SYSTEM, matches_trigger};

slint::include_modules!();

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    // ── 0. Config ─────────────────────────────────────────────────────────────
    let _config = core::config::AppConfig::load();

    // ── 1. Initialize plugins ─────────────────────────────────────────────────
    let mut app_plugin = AppLauncherPlugin::new();
    app_plugin.init()?;
    let app_plugin = Rc::new(RefCell::new(app_plugin));

    let mut emoji_plugin = EmojiPlugin::new();
    emoji_plugin.init()?;
    let emoji_plugin = Rc::new(RefCell::new(emoji_plugin));

    let mut clip_plugin = ClipboardPlugin::new();
    clip_plugin.init()?;
    let clip_plugin = Rc::new(RefCell::new(clip_plugin));

    // ── 2. Slint UI ───────────────────────────────────────────────────────────
    let ui = AppWindow::new()?;

    // ── 3. Global hotkeys ─────────────────────────────────────────────────────
    let manager = GlobalHotKeyManager::new().unwrap();
    let hotkey_main  = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space);
    let hotkey_emoji = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyE);
    let hotkey_clip  = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);
    manager.register(hotkey_main).unwrap();
    manager.register(hotkey_emoji).unwrap();
    manager.register(hotkey_clip).unwrap();

    // ── 4. Pop mode (back button / Escape / Alt+Left) ─────────────────────────
    let ui_handle_pop = ui.as_weak();
    ui.on_pop_mode(move || {
        let ui = ui_handle_pop.unwrap();
        ui.set_active_mode(AppMode::Root);
        ui.set_search_text("".into());
        ui.invoke_text_changed("".into());
        ui.invoke_update_scroll(0.0);
        
        let focus_ui = ui.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(u) = focus_ui.upgrade() {
                u.invoke_focus_search();
            }
        });
    });

    // ── 5. Text changed — dispatch using centralized trigger table ────────────
    let ui_handle   = ui.as_weak();
    let app_clone   = app_plugin.clone();
    let emoji_clone = emoji_plugin.clone();
    let clip_clone  = clip_plugin.clone();

    ui.on_text_changed(move |text| {
        let ui = ui_handle.unwrap();
        let mut query = text.to_string();

        // Prefix dispatch in root mode — triggers defined in core::shortcuts
        if ui.get_active_mode() == AppMode::Root {
            if matches_trigger(&query, TRIGGER_EMOJI) {
                ui.set_active_mode(AppMode::Emoji);
                ui.set_search_text("".into());
                query = String::new();
            } else if matches_trigger(&query, TRIGGER_CLIPBOARD) {
                ui.set_active_mode(AppMode::Clipboard);
                ui.set_search_text("".into());
                query = String::new();
            } else if matches_trigger(&query, TRIGGER_SYSTEM) {
                ui.set_active_mode(AppMode::SystemActions);
                ui.set_search_text("".into());
                let focus_ui = ui.as_weak();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(u) = focus_ui.upgrade() {
                        u.invoke_focus_system();
                    }
                });
                return; // System mode has no search — nothing to query
            }
        }

        let mode = ui.get_active_mode();

        match mode {
            AppMode::Root => {
                let mut plugin = app_clone.borrow_mut();
                let matches = plugin.search(&query, 4);
                let results: Vec<SearchResult> = matches.iter().map(|app| {
                    let slint_image = {
                        let path = &app.icon_path;
                        if !path.is_empty() && !path.ends_with(".xpm") {
                            match slint::Image::load_from_path(std::path::Path::new(path)) {
                                Ok(img) => img,
                                Err(_)  => slint::Image::default(),
                            }
                        } else {
                            slint::Image::default()
                        }
                    };
                    SearchResult {
                        name:       app.name.clone().into(),
                        exec:       app.exec.clone().into(),
                        icon:       slint_image,
                        type_label: app.type_label.clone().into(),
                    }
                }).collect();
                ui.set_results(Rc::new(VecModel::from(results)).into());
                ui.set_selected_index(0);
            }

            AppMode::Emoji => {
                let mut plugin = emoji_clone.borrow_mut();
                let categories_map = plugin.search_for_ui(&query, 3000);
                let categories: Vec<EmojiCategory> = categories_map
                    .into_iter()
                    .map(|(name, emojis)| {
                        let slint_emojis: Vec<EmojiResult> = emojis.into_iter().map(|e| {
                            EmojiResult {
                                character:  e.character.into(),
                                name:       e.name.into(),
                                row:        e.row,
                                col:        e.col,
                                orig_index: e.orig_index,
                            }
                        }).collect();
                        EmojiCategory {
                            name:   name.into(),
                            count:  slint_emojis.len() as i32,
                            emojis: Rc::new(VecModel::from(slint_emojis)).into(),
                        }
                    })
                    .collect();
                ui.set_emoji_categories(Rc::new(VecModel::from(categories)).into());
                if query.is_empty() {
                    ui.set_selected_index(0);
                    ui.invoke_update_scroll(0.0);
                }
            }

            AppMode::Clipboard => {
                let mut plugin = clip_clone.borrow_mut();
                let items = plugin.search(&query, 12);
                let results: Vec<ClipboardResult> = items.iter()
                    .map(|s| ClipboardResult { content: s.clone().into() })
                    .collect();
                ui.set_clipboard_results(Rc::new(VecModel::from(results)).into());
                ui.set_selected_index(0);
            }

            AppMode::SystemActions => {} // No dynamic search; grid is static in Slint
        }
    });

    // ── 6. Emoji 2-D grid navigation ─────────────────────────────────────────
    enum MoveDir { Up, Down, Left, Right }
    let move_focus = |ui_weak: &slint::Weak<AppWindow>, dir: MoveDir| {
        if let Some(ui) = ui_weak.upgrade() {
            let cats = ui.get_emoji_categories();
            if cats.row_count() == 0 { return; }
            let group_idx = ui.get_current_group_index() as usize;
            let active_cat = match cats.row_data(group_idx) { Some(c) => c, None => return };

            let mut flat: Vec<(i32, i32, i32)> = Vec::new();
            for j in 0..active_cat.emojis.row_count() {
                if let Some(e) = active_cat.emojis.row_data(j) {
                    flat.push((e.orig_index, e.row, e.col));
                }
            }
            if flat.is_empty() { return; }

            let current_orig = ui.get_selected_index() as i32;
            let current_pos = flat.iter().position(|(oi, _, _)| *oi == current_orig).unwrap_or(0);
            let (_, cur_row, cur_col) = flat[current_pos];

            let next_pos = match dir {
                MoveDir::Left  => current_pos.saturating_sub(1),
                MoveDir::Right => (current_pos + 1).min(flat.len() - 1),
                MoveDir::Up | MoveDir::Down => {
                    let going_up = matches!(dir, MoveDir::Up);
                    let target_row = if going_up { cur_row - 1 } else { cur_row + 1 };
                    flat.iter().enumerate()
                        .filter(|(_, (_, r, _))| *r == target_row)
                        .min_by_key(|(_, (_, _, c))| (c - cur_col).abs())
                        .map(|(pos, _)| pos)
                        .unwrap_or(current_pos)
                }
            };
            let (next_orig, next_local_row, _) = flat[next_pos];
            ui.set_selected_index(next_orig);
            ui.invoke_update_scroll((50 + next_local_row * 85) as f32);
        }
    };

    let ui_nav_up    = ui.as_weak();
    ui.on_nav_up(move || move_focus(&ui_nav_up, MoveDir::Up));
    let ui_nav_down  = ui.as_weak();
    ui.on_nav_down(move || move_focus(&ui_nav_down, MoveDir::Down));
    let ui_nav_left  = ui.as_weak();
    ui.on_nav_left(move || move_focus(&ui_nav_left, MoveDir::Left));
    let ui_nav_right = ui.as_weak();
    ui.on_nav_right(move || move_focus(&ui_nav_right, MoveDir::Right));

    let ui_cycle = ui.as_weak();
    ui.on_cycle_group(move || {
        let ui = ui_cycle.unwrap();
        let total = ui.get_emoji_categories().row_count() as i32;
        if total == 0 { return; }
        let next = (ui.get_current_group_index() + 1) % total;
        ui.set_current_group_index(next);
        ui.set_selected_index(0);
        ui.invoke_update_scroll(0.0);
    });

    // ── 7. List navigation ────────────────────────────────────────────────────
    let ui_next = ui.as_weak();
    ui.on_next_item(move || {
        let ui = ui_next.unwrap();
        let current = ui.get_selected_index();
        let max = match ui.get_active_mode() {
            AppMode::Root          => ui.get_results().row_count().saturating_sub(1) as i32,
            AppMode::Clipboard     => ui.get_clipboard_results().row_count().saturating_sub(1) as i32,
            AppMode::SystemActions => 4, // 5 buttons (0-4)
            AppMode::Emoji => {
                let cats = ui.get_emoji_categories();
                let total: usize = (0..cats.row_count())
                    .filter_map(|i| cats.row_data(i))
                    .map(|c| c.emojis.row_count())
                    .sum();
                total.saturating_sub(1) as i32
            }
        };
        if current < max { ui.set_selected_index(current + 1); }
    });

    let ui_prev = ui.as_weak();
    ui.on_prev_item(move || {
        let ui = ui_prev.unwrap();
        let current = ui.get_selected_index();
        if current > 0 { ui.set_selected_index(current - 1); }
    });

    // ── 8. Execute selected ───────────────────────────────────────────────────
    let ui_exec = ui.as_weak();
    ui.on_execute_selected(move || {
        let ui  = ui_exec.unwrap();
        let mode = ui.get_active_mode();
        let idx  = ui.get_selected_index() as usize;

        match mode {
            AppMode::Root => {
                let results = ui.get_results();
                if let Some(item) = results.row_data(idx) {
                    println!("[app] Launching: {}", item.exec);
                    AppLauncherPlugin::launch(&item.exec.to_string());
                    std::process::exit(0);
                }
            }
            AppMode::Emoji => {
                let cats = ui.get_emoji_categories();
                for i in 0..cats.row_count() {
                    if let Some(cat) = cats.row_data(i) {
                        for j in 0..cat.emojis.row_count() {
                            if let Some(e) = cat.emojis.row_data(j) {
                                if e.orig_index == idx as i32 {
                                    println!("[emoji] Copying: {}", e.character);
                                    EmojiPlugin::copy_to_clipboard(&e.character.to_string());
                                    std::process::exit(0);
                                }
                            }
                        }
                    }
                }
            }
            AppMode::Clipboard => {
                let results = ui.get_clipboard_results();
                if let Some(item) = results.row_data(idx) {
                    println!("[clipboard] Copying item");
                    ClipboardPlugin::copy_item(&item.content.to_string());
                    std::process::exit(0);
                }
            }
            AppMode::SystemActions => {} // handled by on_execute_system_action
        }
    });

    // ── 9. System action execution ────────────────────────────────────────────
    let ui_sys = ui.as_weak();
    let sys_plugin = std::rc::Rc::new(plugins::systemactions::SystemActionsPlugin::new());
    ui.on_execute_system_action(move |action| {
        if let Some(ui) = ui_sys.upgrade() {
            ui.hide().ok();
        }
        sys_plugin.execute_action(action.as_str());
    });

    // ── 10. Global hotkey listener thread ────────────────────────────────────
    let ui_hotkeys = ui.as_weak();
    let main_id    = hotkey_main.id();
    let emoji_id   = hotkey_emoji.id();
    let clip_id    = hotkey_clip.id();

    let global_hotkey_channel = GlobalHotKeyEvent::receiver();
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = global_hotkey_channel.try_recv() {
                let handle = ui_hotkeys.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = handle.upgrade() {
                        let (mode, text) = if event.id == main_id {
                            (AppMode::Root, "")
                        } else if event.id == emoji_id {
                            (AppMode::Emoji, "")
                        } else if event.id == clip_id {
                            (AppMode::Clipboard, "")
                        } else {
                            return;
                        };
                        ui.set_active_mode(mode);
                        ui.set_search_text(text.into());
                        ui.invoke_text_changed(text.into());
                    }
                }).unwrap();
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    // ── 11. Slint run loop ────────────────────────────────────────────────────
    println!("Starting Raycast AppWindow...");
    let ui_init = ui.as_weak();
    let init_timer = slint::Timer::default();
    init_timer.start(slint::TimerMode::SingleShot, std::time::Duration::from_millis(50), move || {
        if let Some(ui) = ui_init.upgrade() {
            ui.invoke_text_changed("".into());
        }
    });

    ui.run().map_err(|e| Box::new(e) as Box<dyn Error>)?;
    Ok(())
}
