# Product Requirements Document (PRD)
## Linux Productivity Launcher

**Version:** 1.0  
**Date:** March 13, 2026  
**Status:** In Development  

---

## 1. Executive Summary

### 1.1 Product Overview
A Raycast-inspired productivity launcher for Linux that provides quick access to applications, clipboard management, productivity tools, and system utilities through a unified, keyboard-driven interface.

### 1.2 Problem Statement
Linux users lack a unified, efficient launcher application similar to Raycast (macOS) or PowerToys Run (Windows) that combines:
- Fast application launching
- Clipboard history management
- Quick access to productivity tools
- System controls and utilities
- Extensible functionality through plugins

### 1.3 Target Users
- Linux power users seeking productivity improvements
- Developers and technical professionals on Linux
- Users transitioning from macOS (Raycast users) to Linux
- Productivity-focused Linux desktop users

---

## 2. Product Vision & Goals

### 2.1 Vision Statement
To create the most powerful and extensible productivity launcher for Linux, enabling users to perform common tasks 10x faster through keyboard-first interactions.

### 2.2 Success Metrics
- User adoption: 10,000+ active users within 6 months
- Average daily usage: 50+ launches per user
- Task completion time: 50% reduction vs traditional methods
- User retention: 70% monthly active users after 3 months
- Extension ecosystem: 20+ community extensions within 1 year

---

## 3. Current Development Status

### 3.1 Completed Features ✅

#### Core Infrastructure
- ✅ **Rust + Slint Setup**: Project configured with Slint UI framework and Rust backend
- ✅ **Build System**: Proper Cargo.toml with all necessary dependencies
  - `slint` with femtovg and skia renderers
  - `freedesktop-desktop-entry` for .desktop file parsing
  - `global-hotkey` for keyboard shortcuts
  - `nucleo` for fuzzy searching
  - `rusqlite` for database
  - `tokio` async runtime
  - `walkdir` for file system traversal

#### App Launcher (✅ Fully Functional)
- ✅ **Desktop Entry Parsing**: Reads .desktop files from multiple locations
  - `/usr/share/applications`
  - `~/.local/share/applications`
  - `/var/lib/flatpak/exports/share/applications`
  - `~/.local/share/flatpak/exports/share/applications`
  - `/var/lib/snapd/desktop/applications`
- ✅ **Package Type Detection**: Identifies and labels apps by package manager
  - Flatpak packages
  - Snap packages
  - Native packages (APT/DNF/etc)
- ✅ **Icon Resolution**: Comprehensive icon lookup system
  - Freedesktop icon theme spec compliant
  - Searches hicolor theme (scalable, 256x256, 128x128, etc.)
  - Flatpak and Snap icon support
  - Reverse-DNS name handling (e.g., `org.app.Name`)
  - Extension priority: SVG → PNG → JPG → ICO
- ✅ **Fuzzy Search**: Real-time fuzzy matching using Nucleo library
- ✅ **Binary Validation**: Checks if executables exist before showing
- ✅ **Exec Field Parsing**: Properly handles field codes (%f, %u, etc.)
- ✅ **App Execution**: Launches applications via shell command

#### Emoji Picker (✅ Fully Functional)
- ✅ **Emoji Database**: SQLite database with emoji data
- ✅ **Category Organization**: Emojis grouped by categories
  - Smileys & Emotion
  - People & Body
  - Animals & Nature
  - Food & Drink
  - Travel & Places
  - Activities
  - Objects
  - Symbols
  - Flags
- ✅ **Grid Layout**: 7-column emoji grid with visual selection
- ✅ **Keyboard Navigation**: 
  - Arrow keys (Up/Down/Left/Right) for grid navigation
  - Ctrl+T to cycle between emoji categories
  - Enter to copy selected emoji
- ✅ **Search**: Real-time emoji name search
- ✅ **Copy to Clipboard**: Uses `wl-copy` for Wayland clipboard
- ✅ **Category Badge**: Shows current category name and emoji count
- ✅ **Focus Management**: Smart scrolling to keep selected emoji visible

#### Clipboard History (✅ Basic Implementation)
- ✅ **Clipboard Mode**: Dedicated mode for clipboard history (AppMode::Clipboard)
- ✅ **SQLite Storage**: Database table for clipboard entries
- ✅ **List View**: Shows clipboard items in scrollable list
- ✅ **Copy Action**: Re-copy items from history
- ⚠️ **Background Monitoring**: Needs clipboard monitoring daemon

#### UI/UX (✅ Modern & Polished)
- ✅ **Glassmorphism Design**: Semi-transparent dark theme (Catppuccin Mocha)
- ✅ **Modal System**: Three modes (Root/App, Emoji, Clipboard)
- ✅ **Back Button**: Contextual back button in non-root modes
- ✅ **Dynamic Height**: Window height adjusts based on results
- ✅ **Visual Feedback**: Hover states, selection highlighting
- ✅ **Rounded Corners**: 12px border radius
- ✅ **Footer Hints**: Contextual instructions in footer

#### Keyboard & Shortcuts (✅ Implemented)
- ✅ **Global Hotkeys**: Three registered shortcuts
  - Main launcher (default: Super+Space or configured)
  - Emoji picker
  - Clipboard history
- ✅ **Hotkey Listener**: Background thread for global hotkey events
- ✅ **Focus Handling**: Auto-focus on search input
- ✅ **Escape to Exit/Back**: Context-aware escape behavior
- ✅ **Enter to Execute**: Launch app or copy emoji/clipboard
- ✅ **Arrow Navigation**: Up/Down for lists, 4-directional for emoji grid

### 3.2 Known Technical Issues & Required Fixes

#### P0 - Critical (Blocking MVP)
1. **Background Process**
   - **Current**: App exits after action execution
   - **Required**: Persistent background daemon
   - **Implementation**: 
     - Keep process running after window hide
     - Window shows/hides on hotkey
     - Only `std::process::exit(0)` removed from execution callbacks

2. **Clipboard Monitoring**
   - **Current**: Clipboard history table exists but no monitoring
   - **Required**: Background service to watch clipboard changes
   - **Implementation**:
     - Use `arboard` or `clipboard-master` crate
     - Monitor clipboard in background thread
     - Insert new entries into SQLite on change
     - Limit history size (100-500 items)

#### P1 - Important (Should Have for v1.0)
1. **Settings Window**: No configuration UI yet
2. **Keyboard Shortcut Customization**: Hardcoded shortcuts
3. **Extension System**: No `@` command trigger system
4. **File Search**: Not implemented
5. **Web Search Integration**: No Google/YouTube search

### 3.3 Architecture Notes

**Current File Structure:**
```
wayland-palette/
├── Cargo.toml
├── build.rs (Slint compilation)
├── palette.slint (UI definition)
├── main.rs (Core logic)
└── emojis.db (SQLite database)
```

**Active Window System:**
- Window is frameless (`no-frame: true`)
- Background is transparent
- Uses Catppuccin Mocha color scheme
- Three distinct modes managed by `AppMode` enum

**Data Flow:**
1. Global hotkey pressed → Event received in background thread
2. Slint event loop invoked → UI shows, mode set
3. User types → Fuzzy search triggers
4. Results update → UI re-renders
5. Selection executed → Action performed, app exits (needs fix)

### 3.4 Next Immediate Steps

**Phase 1: Fix Critical Issues (Week 1-2)**
1. Implement window overlay mode (X11 + Wayland)
2. Convert to background daemon (remove exit calls)
3. Add clipboard monitoring service
4. Test stability on Ubuntu/Fedora

**Phase 2: Extension System (Week 3-4)**
1. Implement `@` trigger detection
2. Add extension trait and registry
3. Port clipboard to `@clipboard` extension
4. Add `@note` and `@todo` extensions

**Phase 3: Settings & Polish (Week 5-6)**
1. Create settings window (Slint component)
2. Add shortcut customization
3. File search implementation
4. Google/YouTube integration

---

## 4. Core Features & Requirements

### 4.1 Window & System Integration

#### 4.1.1 Window Behavior (P0 - Critical)

**Window Type Architecture:**
The application uses two distinct window types for different purposes:

**1. Overlay/Utility Windows** (Hidden from Dock/Taskbar)
There are two categories of overlay windows:

**A) Main Launcher Window** (`palette.slint`)
- Primary search interface
- Mode switching (Root/Plugin views)
- Search bar and results
- Plugin view container

**B) Utility Overlay Screens** (Plugin-specific overlays)
- Full-screen or large overlay interfaces for specific plugins
- Examples:
  - **App Launcher View:** List of applications with icons
  - **Emoji Picker:** Grid of emojis with arrow navigation
  - **Clipboard History:** List of clipboard items
  - **System Actions:** Grid of system controls (Lock, Sleep, etc.)
  - **Color Picker:** Color selection interface
  - **File Search:** File results with preview
  - **Music Control:** Now playing with controls

**All Overlay Windows Share These Characteristics:**
- Window Type: Overlay/Utility class
- Taskbar Visibility: Hidden
- Alt+Tab: Does not appear
- Activation: Global shortcut or launcher trigger
- Icon: Not visible in dock
- Positioning: Center screen on activation
- Dismissal: Auto-hide on focus loss or Esc
- Navigation: Arrow keys for grid/list navigation
- Execution: Enter key to execute selected action

**2. Normal Windows** (Visible in Dock/Taskbar)
- Onboarding window (first-time setup)
- Main settings window
- General settings pages
- **Characteristics**:
  - Window Type: Normal application window
  - Taskbar Visibility: Visible
  - Alt+Tab: Appears normally
  - Icon: Visible in dock/taskbar
  - Close button: Standard window controls
  - Persistent: Stays open until user closes

**Utility Overlay Screen Pattern:**
Many plugins use a dedicated overlay screen with these common features:
- **Grid or List Layout:** Visual organization of options
- **Arrow Key Navigation:** ←/→/↑/↓ to move between items
- **Visual Selection:** Highlighted card/item shows current selection
- **Enter to Execute:** Confirm selection
- **Esc to Cancel:** Return to launcher or close
- **Status Indicators:** Show current state (WiFi on/off, brightness %, etc.)
- **Large Icons/Emojis:** Easy visual identification

**Examples of Utility Overlay Screens:**
1. **System Actions:** 3×3 grid of system controls with arrow navigation
2. **Emoji Picker:** 7-column grid with category tabs
3. **Color Picker:** Color wheel or palette grid
4. **File Search:** File list with preview pane
5. **Screenshot Viewer:** Grid of recent screenshots

This pattern provides consistent, keyboard-driven navigation across all plugins.

#### 4.1.2 Keyboard Shortcuts System

The application implements a **two-tier shortcut system**:

**Tier 1: Global Shortcuts** (System-wide, work anywhere)
- Trigger launcher/plugins from any application
- Registered at OS level
- Configurable per user
- Examples:
  - `Super+Space` → Main launcher
  - `Super+E` → Emoji picker
  - `Super+V` → Clipboard history
  - `Super+N` → Quick note
  - `Super+T` → Todo list
  - `Super+;` → Settings window

**Tier 2: In-App Shortcuts** (Only when launcher is active)
- Navigation and actions within the launcher
- Plugin-specific commands
- Mode switching
- Examples:
  - `↑/↓` → Navigate results
  - `Enter` → Execute selected
  - `Esc` → Close/go back
  - `Ctrl+K` → Command palette
  - `Ctrl+,` → Open settings
  - `@` → Extension trigger
  - `!` → Web search prefix
  - `/` → File search
  - `>` → System actions

#### 4.1.3 Raycast-Style Per-Feature Configuration System

**Philosophy:** Each plugin has complete, independent configuration including global shortcuts, in-app shortcuts, aliases, and plugin-specific settings - exactly like Raycast.

**Configuration Structure:**
```toml
# ~/.config/wayland-palette/config.toml

[general]
theme = "catppuccin-mocha"
launch_at_startup = false
desktop_environment = "auto"  # auto-detected: GNOME/KDE

# ══════════════════════════════════════════════════════════
# APPLICATION LAUNCHER PLUGIN
# ══════════════════════════════════════════════════════════
[plugins.app_launcher]
enabled = true
global_hotkey = "Super+Space"      # System-wide trigger
in_app_hotkey = ""                 # Root mode - always active
aliases = []                        # No aliases - root mode

[plugins.app_launcher.settings]
show_package_types = true           # Show Flatpak/Snap/System labels
index_flatpak = true
index_snap = true
fuzzy_search_threshold = 0.8

# ══════════════════════════════════════════════════════════
# EMOJI PICKER PLUGIN
# ══════════════════════════════════════════════════════════
[plugins.emoji]
enabled = true
global_hotkey = "Super+E"          # System-wide trigger
in_app_hotkey = "Ctrl+E"           # Quick switch when launcher open
aliases = ["@emoji", "@e", "emoji"] # Multiple text triggers

[plugins.emoji.settings]
recent_count = 20                   # Number of recent emojis
default_category = "smileys"        # Default emoji category
show_names = true                   # Show emoji names
show_categories = true              # Show category badge

# ══════════════════════════════════════════════════════════
# CLIPBOARD HISTORY PLUGIN
# ══════════════════════════════════════════════════════════
[plugins.clipboard]
enabled = true
global_hotkey = "Super+V"          # System-wide trigger
in_app_hotkey = "Ctrl+V"           # Quick switch when launcher open
aliases = ["@clipboard", "@clip", "@c", "clip"]

[plugins.clipboard.settings]
max_items = 100                     # Maximum history items
clear_on_exit = false
ignore_password_managers = true     # Exclude sensitive apps
auto_paste = false                  # Wayland clipboard behavior

# ══════════════════════════════════════════════════════════
# QUICK NOTES PLUGIN (Disabled by default)
# ══════════════════════════════════════════════════════════
[plugins.note]
enabled = false
global_hotkey = "Super+N"
in_app_hotkey = "Ctrl+N"
aliases = ["@note", "@n", "note"]

[plugins.note.settings]
default_folder = "~/Documents/Notes"
auto_save = true
markdown_support = true

# ══════════════════════════════════════════════════════════
# TODO LIST PLUGIN (Disabled by default)
# ══════════════════════════════════════════════════════════
[plugins.todo]
enabled = false
global_hotkey = "Super+T"
in_app_hotkey = "Ctrl+T"
aliases = ["@todo", "@t", "task"]

[plugins.todo.settings]
show_completed = true
sort_by = "priority"  # priority, due_date, created_at
```

**Per-Plugin Configuration Features:**
- ✅ **Enable/Disable Toggle** - Turn any plugin on/off
- ✅ **Global Hotkey** - System-wide shortcut (e.g., Super+E)
- ✅ **In-App Hotkey** - Quick switch when launcher is open (e.g., Ctrl+E)
- ✅ **Multiple Aliases** - Text triggers (e.g., @emoji, @e, emoji)
- ✅ **Plugin-Specific Settings** - Custom configuration per plugin
- ✅ **Conflict Detection** - Warns if shortcuts conflict
- ✅ **User Customizable** - Change any setting through UI

**Alias System Features:**
- Multiple aliases per plugin (user can add/remove)
- Fuzzy matching on aliases
- No duplicate aliases across plugins
- Add custom aliases through settings UI
- Remove aliases with one click

---

### 4.2 Core Launcher Features

#### 4.2.1 Application Launcher (P0)
- **Search**: Fuzzy search across installed applications
- **Package Type Detection**: Display package source
  - Snap packages
  - Flatpak packages
  - AppImage
  - APT/DNF/Pacman (native packages)
  - Manual installations
- **Launch Speed**: Sub-100ms response time
- **Recent Apps**: Show frequently/recently used applications
- **App Actions**: Pin, unpin, show in file manager

#### 4.2.2 File Search (P1)
- Index user home directory
- Support for custom indexed paths
- Real-time search results
- File type filtering (documents, images, videos, etc.)
- Quick actions: Open, reveal in folder, copy path

---

### 4.3 Extension System (@-Commands)

Type `@` to trigger extension suggestions. Each extension provides specialized functionality.

#### 4.3.1 @emoji (P0 - Completed)
- Search and insert emojis
- Recent emoji history
- Category browsing
- Skin tone variations

#### 4.3.2 @clipboard (P0)
- **Clipboard History**: Last 100+ items
- **Search**: Full-text search across clipboard history
- **Pin Items**: Pin frequently used clipboard entries
- **Preview**: Image preview for clipboard images
- **Privacy**: Option to exclude sensitive apps (password managers)
- **Actions**: Paste, copy again, delete from history
- **Data Types**: Text, images, files, rich text

#### 4.3.3 @note (P1)
- **Quick Notes**: Create temporary notes
- **Persistent Storage**: Save notes locally
- **Search**: Full-text search across all notes
- **Markdown Support**: Basic markdown rendering
- **Tags**: Organize notes with tags
- **Quick Actions**: Create, edit, delete, share

#### 4.3.4 @project (P1)
- **Project Management**: Track multiple projects
- **Task Lists**: Per-project task management
- **Links**: Quick access to project folders, URLs, docs
- **Status Tracking**: Active, archived, completed projects
- **Search**: Search across projects and tasks

#### 4.3.5 @todo (P1)
- **Task Management**: In-app to-do list
- **Due Dates**: Set reminders and due dates
- **Priority Levels**: High, medium, low
- **Completion Tracking**: Mark complete, recurring tasks
- **Integration**: Optional calendar integration

#### 4.3.6 @color (P2)
- **Color Picker**: System-wide color picker
- **Format Support**: HEX, RGB, HSL, RGBA
- **Copy Formats**: One-click copy in any format
- **Color History**: Recent picked colors
- **Palette Generator**: Generate color schemes

#### 4.3.7 @system (P1) - System Actions Utility Screen

**System Actions** provides quick access to system power and connectivity controls through a dedicated overlay screen with visual grid navigation.

**Access Methods:**
- Type `@system`, `@sys`, or `action` in launcher
- Press `Ctrl+S` (in-app shortcut - when launcher is open)
- Press `Super+S` (global shortcut - system-wide)

**UI Layout - Grid-Based Selection:**
```
┌────────────────────────────────────────────────────┐
│  System Actions                                    │
├────────────────────────────────────────────────────┤
│                                                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  🔒      │  │  💤      │  │  🔄      │        │
│  │  Lock    │  │  Sleep   │  │  Restart │        │
│  │ Screen   │  │          │  │          │        │
│  └──────────┘  └──────────┘  └──────────┘        │
│                                                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  ⚡      │  │  👋      │  │  🌙      │        │
│  │ Shutdown │  │  Log Out │  │  Night   │        │
│  │          │  │          │  │   Mode   │        │
│  └──────────┘  └──────────┘  └──────────┘        │
│                                                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  📶      │  │  🔵      │  │  🔕      │        │
│  │   WiFi   │  │Bluetooth │  │    DND   │        │
│  │  Toggle  │  │  Toggle  │  │   Mode   │        │
│  └──────────┘  └──────────┘  └──────────┘        │
│                                                    │
└────────────────────────────────────────────────────┘
    Use ← → ↑ ↓ to select • Enter to execute
```

**Navigation:**
- **Arrow Keys (←/→/↑/↓):** Navigate between system actions in grid
- **Enter:** Execute selected action
- **Esc:** Close system actions screen
- **Visual Selection:** Highlighted card shows current selection
- **Grid Layout:** 3 columns × 3 rows (9 actions)

**Available System Actions:**

**Power Actions:**

1. **Lock Screen** 🔒
   - Immediately locks the screen
   - GNOME: `loginctl lock-session`
   - KDE: `qdbus org.freedesktop.ScreenSaver /ScreenSaver Lock`

2. **Sleep/Suspend** 💤
   - Suspends system to RAM
   - Command: `systemctl suspend`

3. **Restart** 🔄
   - Restarts the system
   - Command: `systemctl reboot`
   - Shows confirmation dialog

4. **Shutdown** ⚡
   - Powers off the system
   - Command: `systemctl poweroff`
   - Shows confirmation dialog

5. **Log Out** 👋
   - Logs out current user
   - GNOME: `gnome-session-quit --logout`
   - KDE: `qdbus org.kde.Shutdown /Shutdown logout`

**Display & Notifications:**

6. **Night Mode** 🌙
   - Toggles night light/blue light filter
   - GNOME: `gsettings set org.gnome.settings-daemon.plugins.color night-light-enabled`
   - KDE: Toggle Redshift/Night Color

7. **Do Not Disturb** 🔕
   - Toggles DND mode
   - Silences notifications
   - GNOME: `gsettings set org.gnome.desktop.notifications show-banners`

**Connectivity Actions:**

8. **WiFi Toggle** 📶
   - Enables/disables WiFi
   - Shows current status (connected/disconnected)
   - GNOME: `nmcli radio wifi on/off`

9. **Bluetooth Toggle** 🔵
    - Enables/disables Bluetooth
    - Shows current status (on/off)
    - Command: `bluetoothctl power on/off`

**Note:** Brightness and volume controls are available in the main Settings page under System/Display and Sound sections respectively, where they can be adjusted with more precision using sliders.

**UI Component:** `ui/plugin/system_actions.slint`
- **Window Type:** OVERLAY (hidden from dock/overview)
- **Layout:** Grid with 3 columns × 3 rows
- **Card Size:** 100px × 100px
- **Selection:** Highlighted border + background color
- **Icons:** Large emoji or system icons
- **Labels:** Centered below icon

**Backend:** `src/plugins/system_actions.rs`
- Detects desktop environment (GNOME/KDE/Other)
- Uses appropriate commands per desktop
- Handles confirmations for destructive actions (shutdown/restart/logout)
- Provides status feedback for toggles (WiFi on/off, Bluetooth on/off, Night Mode on/off)

**Configuration:** Per-plugin settings in `config.toml`
```toml
[plugins.system_actions]
enabled = true
global_hotkey = "Super+S"
in_app_hotkey = "Ctrl+S"
aliases = ["@system", "@sys", "action"]

[plugins.system_actions.settings]
show_confirmations = true          # Confirm shutdown/restart/logout
grid_columns = 3                   # Actions per row
show_status = true                 # Show WiFi/BT/Night Mode status
```

**Desktop Integration:**
- **GNOME:** Uses `gsettings`, `loginctl`, `nmcli`, `gnome-session-quit`
- **KDE:** Uses `qdbus`, KDE system commands, `qdbus org.kde.Shutdown`
- **Generic Linux:** Falls back to `systemctl`, universal commands

**Special Features:**
- **Status Indicators:** Shows WiFi connected/disconnected, Bluetooth on/off, Night Mode enabled/disabled
- **Confirmations:** Shutdown/restart/logout show "Are you sure?" dialog
- **Quick Toggle:** Actions like WiFi/Bluetooth/Night Mode/DND toggle immediately
- **Visual Feedback:** Selected action highlights, shows loading state during execution

**Implementation Priority:** P1 (Should Have for v1.0)

#### 4.3.8 @download (P2)
- **YouTube Downloader**: Download video and audio
- **Format Selection**: Choose quality and format
- **Audio Only**: MP3 extraction
- **Batch Downloads**: Queue multiple downloads
- **Download Location**: Configurable save path
- **Progress Tracking**: Real-time download progress

#### 4.3.9 @image (P2)
- **Image Converter**: Convert between formats (PNG, JPG, WebP, etc.)
- **Image Compression**: Reduce file size
- **Batch Processing**: Convert multiple images
- **Quality Settings**: Adjustable compression levels
- **Resize**: Batch resize images

#### 4.3.10 @music (P2)
- **Music Control**: System-wide media controls
- **Now Playing**: Display current track
- **Controls**: Play, pause, next, previous, volume
- **Spotify Integration**: 
  - Search Spotify library
  - Play specific tracks/playlists
  - Save to library
  - View currently playing

#### 4.3.11 @screenshot (P2)
- **Quick View**: Browse recent screenshots
- **Actions**: View, copy, delete, annotate
- **Auto-categorization**: Organize by date/application
- **OCR Support**: Extract text from screenshots (future)

#### 4.3.12 @reminder (P2)
- **Quick Reminders**: Set time-based reminders
- **Natural Language**: "Remind me in 2 hours"
- **Notifications**: Desktop notifications
- **Recurring**: Support for recurring reminders

#### 4.3.13 @pomodoro (P2)
- **Pomodoro Timer**: Built-in productivity timer
- **Customizable**: Configure work/break intervals
- **Notifications**: Session completion alerts
- **Statistics**: Track completed pomodoros
- **Do Not Disturb**: Auto-enable during focus sessions

#### 4.3.14 @filesearch (P1)
- **Deep File Search**: Search entire file system
- **Content Search**: Search within file contents
- **Filters**: File type, date modified, size
- **Quick Open**: Open file or reveal location

---

### 4.4 Web Integration

#### 4.4.1 Google Search (P1)
- Direct Google search from launcher
- Search suggestions
- Open results in default browser
- Quick actions: "Search Google for [query]"

#### 4.4.2 YouTube Search (P1)
- Search YouTube videos
- Preview thumbnails
- Open in browser or download
- View duration, channel, view count

---

## 5. Technical Architecture

### 5.1 Technology Stack
- **Primary Language**: Rust
  - Memory safety without garbage collection
  - Excellent performance for system-level applications
  - Strong concurrency primitives
  - Rich ecosystem via crates.io
  
- **GUI Framework**: Slint (formerly SixtyFPS)
  - Native performance with GPU acceleration
  - Declarative UI with `.slint` markup language
  - Excellent Rust integration
  - Cross-platform (X11, Wayland support)
  - Small binary size and low memory footprint
  - Built-in animations and transitions
  
- **Database**: SQLite via `rusqlite` crate
  - Local data storage for clipboard history, notes, projects
  - Fast full-text search with FTS5 extension
  - Zero-config embedded database
  
- **Search & Indexing**: 
  - **Tantivy**: Full-text search engine for file indexing
  - **Fuzzy Matching**: `sublime_fuzzy` or `nucleo` crates
  - **File Watching**: `notify` crate for real-time file system monitoring
  
- **System Integration**:
  - **Clipboard**: `arboard` or `copypasta-ext` crates
  - **Global Hotkeys**: `global-hotkey` crate
  - **System Tray**: `tray-icon` crate
  - **Desktop Entries**: Parse `.desktop` files with `freedesktop-entry-parser`
  
- **Additional Crates**:
  - **HTTP Client**: `reqwest` for YouTube/web integrations
  - **YouTube Download**: `youtube_dl` wrapper or `rustube`
  - **Image Processing**: `image` crate for conversion/compression
  - **Audio/Music**: `mpris` crate for media player control
  - **Spotify**: `rspotify` for Spotify API integration
  - **Async Runtime**: `tokio` for async operations
  - **Serialization**: `serde` with `serde_json` for config/data

### 5.2 Project File Structure

**Clean, Modular Architecture - Complete Backend + Frontend Separation:**

```
wayland-palette/
├── Cargo.toml                      # Dependencies and workspace config
├── build.rs                        # Slint compilation
├── README.md
├── LICENSE
│
├── src/                            # BACKEND - RUST
│   ├── main.rs                     # Entry point, coordinator (55 lines final)
│   │
│   ├── core/                       # Core application systems
│   │   ├── mod.rs
│   │   ├── config.rs               # TOML config loading/saving
│   │   ├── shortcuts.rs            # Global + in-app shortcut management
│   │   ├── window_manager.rs      # Wayland/GNOME window management
│   │   └── daemon.rs               # Background process management
│   │
│   ├── plugins/                    # ONE FILE PER PLUGIN - BACKEND LOGIC
│   │   ├── mod.rs                  # Plugin trait and registry
│   │   ├── app_launcher.rs         # App search/launch (250 lines)
│   │   ├── emoji.rs                # Emoji picker (120 lines)
│   │   ├── clipboard_history.rs   # Clipboard history (150 lines)
│   │   ├── system_actions.rs      # System actions grid (120 lines)
│   │   ├── note.rs                 # Quick notes (future)
│   │   ├── todo.rs                 # Todo list (future)
│   │   ├── project.rs              # Project manager (future)
│   │   ├── color_picker.rs         # Color picker (future)
│   │   ├── file_search.rs          # File search (future)
│   │   ├── web_search.rs           # Web/YouTube search (future)
│   │   ├── download.rs             # YouTube downloader (future)
│   │   ├── image_tools.rs          # Image tools (future)
│   │   ├── music_control.rs        # Music/Spotify (future)
│   │   ├── screenshot.rs           # Screenshot viewer (future)
│   │   ├── reminder.rs             # Reminders (future)
│   │   └── pomodoro.rs             # Pomodoro timer (future)
│   │
│   ├── utils/                      # Shared utilities
│   │   ├── mod.rs
│   │   ├── icon_resolver.rs        # Icon path resolution (150 lines)
│   │   ├── exec_parser.rs          # Exec field parsing (60 lines)
│   │   └── fuzzy_search.rs         # Fuzzy matching wrapper (40 lines)
│   │
│   └── models/                     # Shared data models
│       ├── mod.rs
│       ├── app_entry.rs            # Application entry struct
│       ├── emoji_data.rs           # Emoji data struct
│       ├── clipboard_item.rs       # Clipboard item struct
│       ├── note.rs                 # Note struct
│       ├── todo_item.rs            # Todo item struct
│       └── project.rs              # Project struct
│
├── ui/                             # FRONTEND - SLINT
│   ├── palette.slint               # Main launcher window (OVERLAY - 100 lines)
│   ├── onboarding.slint            # First-time setup (NORMAL - 100 lines)
│   ├── main_settings.slint         # Main settings screen (NORMAL - 80 lines)
│   │
│   ├── plugin/                     # ONE FILE PER PLUGIN - UI (OVERLAY)
│   │   ├── app_launcher.slint      # App launcher view (120 lines)
│   │   ├── emoji.slint             # Emoji picker grid (150 lines)
│   │   ├── clipboard_history.slint # Clipboard list (80 lines)
│   │   ├── system_actions.slint    # System actions grid (100 lines)
│   │   ├── note.slint              # Note editor (future)
│   │   ├── todo.slint              # Todo list (future)
│   │   ├── project.slint           # Project manager (future)
│   │   ├── color_picker.slint      # Color picker grid (future)
│   │   ├── file_search.slint       # File search (future)
│   │   └── web_search.slint        # Web search (future)
│   │
│   ├── settings/                   # Settings pages (NORMAL WINDOWS)
│   │   ├── general.slint           # General settings tab (60 lines)
│   │   ├── shortcuts.slint         # Shortcuts overview (80 lines)
│   │   └── plugins.slint           # Plugin configuration (150 lines)
│   │
│   └── settings/components/        # Reusable settings components
│       ├── plugin-card.slint       # Individual plugin config card
│       ├── hotkey-input.slint      # Shortcut recorder component
│       └── alias-editor.slint      # Alias management component
│
├── assets/                         # Static resources
│   ├── icons/
│   │   ├── app-icon.png
│   │   └── tray-icon.png
│   └── fonts/
│       └── NotoColorEmoji.ttf
│
├── config/                         # Default config templates
│   ├── config.toml.default         # Default configuration
│   ├── shortcuts.toml.default      # Default shortcuts (deprecated)
│   └── aliases.toml.default        # Default aliases (deprecated)
│
└── data/                           # Database and data files
    ├── emojis.db                   # Emoji database (or emojis.json)
    └── clipboard.sqlite            # Clipboard history database
```

**Key Architecture Principles:**

1. **Complete Separation:**
   - Each plugin: 1 Rust file (.rs) + 1 Slint file (.slint)
   - Backend logic completely separated from UI
   - Easy to find and debug (bug in emoji? Look in emoji.rs + emoji.slint)

2. **Window Type Organization:**
   - **Overlay Windows** (Hidden from dock): `palette.slint`, `ui/plugin/*.slint`
   - **Normal Windows** (Visible in dock): `main_settings.slint`, `onboarding.slint`, `ui/settings/*.slint`

3. **Scalability:**
   - Adding new feature = Add 1 .rs file + 1 .slint file
   - No touching existing code
   - Plugin trait ensures consistency

4. **Maintainability:**
   - One feature per file
   - Clear boundaries
   - Independent testing
   - Easy to enable/disable plugins

**File Count:**
- Core modules: 6 files
- Utilities: 3 files
- Models: 6 files
- Plugins (backend): 3-15 files (3 initial, 15 future)
- Plugin UIs (frontend): 3-15 files (matching plugins)
- Settings: 3 files + 3 components
- Total: ~30-50 files (vs 2 monolithic files)

### 5.3 Plugin Architecture

#### 5.3.1 Plugin Trait Interface
```rust
// src/plugins/mod.rs
pub trait Plugin: Send + Sync {
    /// Unique identifier for the plugin
    fn id(&self) -> &str;
    
    /// Display name
    fn name(&self) -> &str;
    
    /// Trigger command (e.g., "@emoji")
    fn trigger(&self) -> &str;
    
    /// Aliases for the trigger (e.g., ["@e", "emoji"])
    fn aliases(&self) -> Vec<&str>;
    
    /// Global shortcut (if any)
    fn global_shortcut(&self) -> Option<&str>;
    
    /// In-app shortcut (when launcher is active)
    fn in_app_shortcut(&self) -> Option<&str>;
    
    /// Search query handler
    fn search(&self, query: &str) -> Vec<SearchResult>;
    
    /// Execute action
    fn execute(&self, action: PluginAction) -> Result<(), PluginError>;
    
    /// Get plugin-specific UI component path
    fn ui_component(&self) -> &str;
    
    /// Initialize plugin (load data, connect to services, etc.)
    fn init(&mut self) -> Result<(), PluginError>;
    
    /// Cleanup on shutdown
    fn cleanup(&self) -> Result<(), PluginError>;
}
```

#### 5.3.2 Plugin Registry
```rust
// src/plugins/mod.rs
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    enabled_plugins: HashSet<String>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let mut registry = PluginRegistry {
            plugins: HashMap::new(),
            enabled_plugins: HashSet::new(),
        };
        
        // Register all built-in plugins
        registry.register(Box::new(AppLauncherPlugin::new()));
        registry.register(Box::new(EmojiPlugin::new()));
        registry.register(Box::new(ClipboardPlugin::new()));
        // ... more plugins
        
        registry
    }
    
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        let id = plugin.id().to_string();
        self.plugins.insert(id.clone(), plugin);
        self.enabled_plugins.insert(id);
    }
    
    pub fn get_by_trigger(&self, trigger: &str) -> Option<&dyn Plugin> {
        self.plugins.values()
            .find(|p| p.trigger() == trigger || p.aliases().contains(&trigger))
            .map(|b| b.as_ref())
    }
}
```

#### 5.3.3 Example Plugin Implementation
```rust
// src/plugins/emoji.rs
use super::{Plugin, PluginAction, PluginError, SearchResult};

pub struct EmojiPlugin {
    emoji_data: Vec<EmojiData>,
    db_connection: Option<rusqlite::Connection>,
}

impl EmojiPlugin {
    pub fn new() -> Self {
        EmojiPlugin {
            emoji_data: Vec::new(),
            db_connection: None,
        }
    }
}

impl Plugin for EmojiPlugin {
    fn id(&self) -> &str { "emoji" }
    fn name(&self) -> &str { "Emoji Picker" }
    fn trigger(&self) -> &str { "@emoji" }
    fn aliases(&self) -> Vec<&str> { vec!["@e", "emoji"] }
    fn global_shortcut(&self) -> Option<&str> { Some("Super+E") }
    fn in_app_shortcut(&self) -> Option<&str> { Some("Ctrl+E") }
    fn ui_component(&self) -> &str { "ui/plugin/emoji.slint" }
    
    fn init(&mut self) -> Result<(), PluginError> {
        // Load emoji database
        let conn = rusqlite::Connection::open("data/emojis.db")?;
        self.db_connection = Some(conn);
        // Load emoji data...
        Ok(())
    }
    
    fn search(&self, query: &str) -> Vec<SearchResult> {
        // Fuzzy search emoji names
        // Return results
        vec![]
    }
    
    fn execute(&self, action: PluginAction) -> Result<(), PluginError> {
        // Copy emoji to clipboard
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}
```

### 5.4 UI Architecture (Slint)

#### 5.4.1 Window Type Management
```rust
// src/core/window_manager.rs
pub enum WindowType {
    /// Overlay windows (no taskbar, hidden from alt+tab)
    Overlay,
    /// Normal windows (visible in taskbar, alt+tab)
    Normal,
}

pub trait WindowConfig {
    fn window_type(&self) -> WindowType;
    fn should_show_icon(&self) -> bool {
        matches!(self.window_type(), WindowType::Normal)
    }
}

// Implementation for different windows
impl WindowConfig for PaletteWindow {
    fn window_type(&self) -> WindowType { WindowType::Overlay }
}

impl WindowConfig for OnboardingWindow {
    fn window_type(&self) -> WindowType { WindowType::Normal }
}

impl WindowConfig for SettingsWindow {
    fn window_type(&self) -> WindowType { WindowType::Normal }
}
```

#### 5.4.2 Slint Component Organization

**Main Launcher (ui/palette.slint):**
```slint
import { VerticalBox } from "std-widgets.slint";

export component PaletteWindow inherits Window {
    // Overlay window configuration
    no-frame: true;
    always-on-top: true;
    background: transparent;
    
    // Will be set via Rust backend:
    // X11: _NET_WM_WINDOW_TYPE_UTILITY
    // Wayland: layer-shell protocol
}
```

**Onboarding (ui/onboarding.slint):**
```slint
export component OnboardingWindow inherits Window {
    // Normal window configuration
    title: "Welcome to Palette";
    icon: @image-url("../assets/icons/app-icon.png");
    
    // Standard window behavior
    // Shows in taskbar and alt+tab
}
```

**Settings (ui/main_settings.slint):**
```slint
export component SettingsWindow inherits Window {
    title: "Palette Settings";
    icon: @image-url("../assets/icons/app-icon.png");
    
    // Standard window behavior
    // Shows in taskbar and alt+tab
}
```

**Plugin UIs (ui/plugin/*.slint):**
- All inherit overlay/utility window properties
- Loaded dynamically based on active plugin
- Share common layout components
- Can be hot-reloaded during development

### 5.5 Configuration System

#### 5.5.1 Config File Structure
```
~/.config/wayland-palette/
├── config.toml          # Main application config
├── shortcuts.toml       # Keyboard shortcuts
├── aliases.toml         # Command aliases
├── plugins.toml         # Plugin enable/disable
└── cache/               # Cache directory
    ├── icons/           # Cached icons
    └── search_index/    # File search index
```

#### 5.5.2 Configuration Loading
```rust
// src/core/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub shortcuts: ShortcutConfig,
    pub plugins: PluginConfig,
    pub appearance: AppearanceConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::config_dir().join("config.toml");
        
        if !config_path.exists() {
            // First run - copy default config
            return Self::create_default();
        }
        
        let content = std::fs::read_to_string(config_path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::config_dir().join("config.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
    
    fn config_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap();
        PathBuf::from(home).join(".config/wayland-palette")
    }
}
```

### 5.6 Main Entry Point

```rust
// src/main.rs
mod core;
mod plugins;
mod utils;
mod models;

use core::{daemon::Daemon, config::AppConfig, window_manager::WindowManager};
use plugins::PluginRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Load configuration
    let config = AppConfig::load()?;
    
    // Check if first run
    if config.is_first_run() {
        // Show onboarding window
        show_onboarding()?;
    }
    
    // Initialize plugin registry
    let mut registry = PluginRegistry::new();
    registry.load_plugins(&config)?;
    
    // Start background daemon
    let daemon = Daemon::new(config, registry)?;
    daemon.start()?;
    
    // Register global hotkeys
    daemon.register_hotkeys()?;
    
    // Run Slint event loop
    slint::run_event_loop()?;
    
    Ok(())
}

fn show_onboarding() -> Result<(), Box<dyn std::error::Error>> {
    slint::include_modules!();
    let window = OnboardingWindow::new()?;
    window.run()?;
    Ok(())
}
```

### 5.7 Performance Requirements
- **Launcher Activation**: <50ms from shortcut press
  - Achieved through: Pre-loaded background process, optimized Slint rendering
- **Search Response**: <100ms for results
  - Achieved through: Tantivy indexing, async search, result streaming
- **Memory Footprint**: <50MB idle (Rust + Slint advantage)
  - Achieved through: Zero-cost abstractions, efficient data structures
- **Startup Time**: <300ms background process
  - Achieved through: Lazy loading, incremental indexing
- **Binary Size**: <15MB (optimized release build)
  - Achieved through: `strip` symbols, `opt-level = "z"`, link-time optimization
- **Plugin Loading**: <10ms per plugin
  - Achieved through: Lazy initialization, parallel loading

### 5.8 Build Configuration

#### 5.8.1 Cargo.toml Workspace
```toml
[package]
name = "wayland-palette"
version = "0.1.0"
edition = "2024"

[workspace]
members = [".", "plugins/*"]  # Future: external plugins

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"

[dependencies]
# ... (same as before, plus modular organization)

[features]
default = ["all-plugins"]
all-plugins = [
    "plugin-app-launcher",
    "plugin-emoji",
    "plugin-clipboard",
    "plugin-note",
    # ... etc
]
# Individual plugin features for selective compilation
plugin-app-launcher = []
plugin-emoji = []
plugin-clipboard = []
```

---

## 6. User Experience

### 6.1 Primary Workflow
1. User presses global shortcut (e.g., Super+Space)
2. Launcher window appears centered on screen
3. User types search query or `@extension`
4. Results appear in real-time
5. User selects result with Enter or click
6. Action executes and launcher auto-hides

### 6.2 Visual Design
- **Theme**: Dark mode with light mode option
- **Style**: Modern, minimal, blur background
- **Typography**: Clear, readable system font
- **Animations**: Smooth 200ms transitions
- **Accessibility**: High contrast, keyboard-only navigation

### 6.3 Settings UI (Raycast-Style Plugin Configuration)

**Main Settings Window:** `ui/main_settings.slint` (Normal Window - Visible in Dock)

**Settings Architecture:**
```
ui/
├── main_settings.slint          (Container - 80-100 lines)
│   ├── Window title: "Palette Settings"
│   ├── TabWidget with 3 tabs
│   └── Standard window decorations
│
├── settings/                    (Content pages)
│   ├── general.slint           (60-80 lines)
│   ├── shortcuts.slint         (80-100 lines)
│   └── plugins.slint           (Main configuration - 150+ lines)
│
└── settings/components/        (Reusable UI components)
    ├── plugin-card.slint       (Individual plugin config)
    ├── hotkey-input.slint      (Shortcut recorder)
    └── alias-editor.slint      (Alias management)
```

**Tab 1: General Settings** (`settings/general.slint`)
- Theme selection (Catppuccin Mocha, Nord, Dracula, etc.)
- Desktop environment info (auto-detected: GNOME/KDE)
- Platform info (Wayland/X11)
- Launch at startup toggle
- System tray icon toggle
- Privacy settings
- Language selection

**Tab 2: Shortcuts Overview** (`settings/shortcuts.slint`)
- Display all global shortcuts (read-only overview)
- Display all in-app shortcuts (read-only overview)
- Wayland notice for global shortcuts
- Link to GNOME/KDE system settings
- Conflict warnings display
- Note: "Configure individual shortcuts in Plugins tab"

**Tab 3: Plugins & Extensions** (`settings/plugins.slint`) - **MAIN CONFIGURATION**

**Layout:**
```
┌──────────────────────────────────────────────────┐
│  Plugins & Extensions                            │
├──────────────────────────────────────────────────┤
│  🔍 Search plugins...                            │
│                                                  │
│  ┌────────────────────────────────────────────┐ │
│  │ Application Launcher           [Expanded] │ │
│  │──────────────────────────────────────────│ │
│  │ ☑ Enabled                                 │ │
│  │                                           │ │
│  │ Global Hotkey                             │ │
│  │ ┌──────────────┐ [Record Shortcut]       │ │
│  │ │ Super+Space  │                          │ │
│  │ └──────────────┘                          │ │
│  │                                           │ │
│  │ Alias (Trigger in Launcher)              │ │
│  │ (root mode - always active)              │ │
│  │                                           │ │
│  │ Settings                                  │ │
│  │ ☑ Show package types                      │ │
│  │ ☑ Index Flatpak apps                      │ │
│  │ ☑ Index Snap apps                         │ │
│  └────────────────────────────────────────────┘ │
│                                                  │
│  ┌────────────────────────────────────────────┐ │
│  │ Emoji Picker                  [Collapsed] │ │
│  │──────────────────────────────────────────│ │
│  │ ☑ Enabled                                 │ │
│  │                                           │ │
│  │ Global Hotkey                             │ │
│  │ ┌──────────────┐ [Record]                │ │
│  │ │ Super+E      │                          │ │
│  │ └──────────────┘                          │ │
│  │                                           │ │
│  │ Aliases                                   │ │
│  │ @emoji [×]  @e [×]  emoji [×]            │ │
│  │ [+ Add Alias]                             │ │
│  │                                           │ │
│  │ In-App Hotkey                            │ │
│  │ ┌──────────────┐ [Record]                │ │
│  │ │ Ctrl+E       │                          │ │
│  │ └──────────────┘                          │ │
│  │                                           │ │
│  │ Settings                                  │ │
│  │ Recent count:    [20        ]            │ │
│  │ Default category: [Smileys ▼]           │ │
│  │ ☑ Show emoji names                        │ │
│  └────────────────────────────────────────────┘ │
│                                                  │
│  ┌────────────────────────────────────────────┐ │
│  │ Clipboard History            [Collapsed] │ │
│  │ ... (similar structure)                   │ │
│  └────────────────────────────────────────────┘ │
│                                                  │
│  ┌────────────────────────────────────────────┐ │
│  │ ☐ Quick Notes                [Disabled]  │ │
│  │ [Enable Plugin]                           │ │
│  └────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

**Per-Plugin Configuration Features:**

1. **Enable/Disable Toggle**
   - Click checkbox to enable/disable plugin
   - Disabled plugins don't load at startup
   - Can be enabled without restart

2. **Global Hotkey Configuration**
   - "Record Shortcut" button
   - Click → Press key combination → Saves
   - Conflict detection (warns if already used)
   - Clear button to remove shortcut
   - Examples: Super+E, Super+Shift+N, Ctrl+Alt+C

3. **Alias Management**
   - List of current aliases (user can remove with [×])
   - Add new alias button
   - Type new alias → Click "+ Add"
   - Validates no duplicates across plugins
   - Examples: @emoji, @e, emoji, emote

4. **In-App Hotkey Configuration**
   - Quick switch when launcher is open
   - Record like global hotkey
   - Examples: Ctrl+E, Ctrl+N, Ctrl+T
   - Optional (can be empty)

5. **Plugin-Specific Settings**
   - Number inputs (with min/max validation)
   - Dropdowns (predefined options)
   - Checkboxes (boolean settings)
   - Text inputs (paths, strings)
   - Color pickers (themes)

**Conflict Detection:**
- Real-time validation as user types/records
- Warning banner if conflict detected
- Shows which plugin uses the conflicting shortcut
- Prevents saving until resolved
- Example: "⚠️ Conflict: 'Super+E' is already used by Emoji Picker"

**Settings Components:**

**HotkeyInput Component** (`settings/components/hotkey-input.slint`)
```
┌─────────────────────────────────────┐
│ Global Hotkey                       │
│ ┌───────────────┐  [Record] [Clear]│
│ │ Super+E       │                   │
│ └───────────────┘                   │
└─────────────────────────────────────┘
```
- Shows current hotkey or "Not set"
- Record button → Captures key press
- Clear button → Removes shortcut
- Visual feedback during recording

**AliasEditor Component** (`settings/components/alias-editor.slint`)
```
┌─────────────────────────────────────┐
│ Aliases (Triggers in Launcher)      │
│ @emoji     [×]                      │
│ @e         [×]                      │
│ emoji      [×]                      │
│ ┌──────────────┐ [+ Add Alias]     │
│ │ Type here... │                    │
│ └──────────────┘                    │
└─────────────────────────────────────┘
```
- Chip-style display of current aliases
- Remove button on each alias
- Add new alias input field
- Validates no duplicates

**PluginCard Component** (`settings/components/plugin-card.slint`)
- Collapsible card for each plugin
- Header shows plugin name and enabled state
- Expand/collapse animation
- Contains all plugin configuration
- Reusable across all plugins

**Settings Persistence:**
- Auto-save on change (debounced)
- Writes to `~/.config/wayland-palette/config.toml`
- Validates before saving
- Shows "Saved" indicator
- Some changes require restart (shows notice)

**GNOME/KDE Integration:**
- Links to system keyboard settings for global shortcuts
- Detects desktop environment
- Shows platform-specific instructions
- Respects system theme

---

## 7. Feature Priority Matrix

### P0 - Must Have (MVP)
- Window overlay functionality
- App launcher with package type detection
- @emoji extension
- @clipboard extension
- Global shortcut configuration
- Settings window

### P1 - Should Have (v1.0)
- @system actions extension (grid with arrow navigation)
- @note, @project, @todo extensions
- File search
- Google/YouTube search
- @filesearch extension
- Keyboard alias system
- Per-plugin shortcut configuration

### P2 - Nice to Have (v1.1+)
- @color picker
- @download YouTube
- @image converter
- @music control & Spotify
- @screenshot viewer
- @reminder
- @pomodoro timer

### P3 - Future Considerations
- Custom extension marketplace
- Cloud sync for clipboard/notes
- Multi-monitor support improvements
- Wayland full feature parity

---

## 8. Technical Risks & Mitigations

### 8.1 Risks
1. **Wayland Compatibility**: Global shortcuts and window positioning limitations
   - *Mitigation*: Partner with compositor developers, use portals API
   
2. **Permission Issues**: File search indexing, clipboard access
   - *Mitigation*: Clear permission requests, sandboxing

3. **Performance**: Indexing large file systems
   - *Mitigation*: Incremental indexing, user-configurable paths

4. **Distribution**: Multiple package formats across distros
   - *Mitigation*: AppImage + Flatpak as universal options

### 8.2 Dependencies
- Clipboard monitoring libraries
- System tray integration
- Global hotkey registration
- File system monitoring

---

## 9. Success Criteria

### 9.1 Launch Criteria (MVP)
- ✅ Window doesn't appear in taskbar/Alt+Tab
- ✅ Global shortcut activation works
- ✅ App launcher with 5+ package type detection
- ✅ Minimum 3 working extensions (@emoji, @clipboard, @system)
- ✅ Settings page functional
- ✅ Stable on Ubuntu 22.04+

### 9.2 Quality Gates
- Zero crashes in 1-hour stress test
- <100ms search latency
- Works on top 3 Linux distros (Ubuntu, Fedora, Arch)
- Passes accessibility audit

---

## 10. Timeline & Milestones

### Current Status: **30% Complete** 
✅ Core infrastructure, app launcher, emoji picker, basic clipboard  
⚠️ Window overlay fix, background daemon, extensions system pending

---

### Phase 1: Critical Fixes & Stability (Week 1-2) - **CURRENT PRIORITY**
**Goal**: Convert to production-ready background daemon

**Tasks:**
- [ ] Fix window overlay issue (X11 + Wayland implementation)
  - Implement `_NET_WM_WINDOW_TYPE_UTILITY` for X11
  - Add Wayland layer-shell protocol support
  - Test on GNOME, KDE, Sway compositors
- [ ] Convert to background daemon
  - Remove `std::process::exit()` calls
  - Implement window show/hide on hotkey
  - Add system tray icon (optional)
- [ ] Implement clipboard monitoring
  - Background thread with clipboard watcher
  - Auto-insert new clipboard entries to SQLite
  - Add timestamp and deduplication
- [ ] Settings window foundation
  - Create basic Slint settings component
  - Implement hotkey configuration UI
  - Save/load config from TOML file

**Success Criteria:**
- ✅ Window doesn't appear in taskbar/Alt+Tab
- ✅ App runs continuously in background
- ✅ Clipboard auto-populates with new copies
- ✅ No crashes over 24-hour test
- ✅ Works on Ubuntu 24.04, Fedora 40

---

### Phase 2: Extension System Foundation (Week 3-4)
**Goal**: Implement `@` command system with core extensions

**Tasks:**
- [ ] Extension architecture
  - Create `Extension` trait interface
  - Build extension registry
  - Implement `@` trigger detection in search
- [ ] Migrate clipboard to extension
  - Refactor as `@clipboard` extension
  - Move logic to extension module
- [ ] Implement `@note` extension
  - Quick note creation/editing
  - SQLite storage
  - Markdown rendering (basic)
- [ ] Implement `@todo` extension
  - Task creation with due dates
  - Priority levels
  - Completion tracking
- [ ] Add `@system` extension
  - Lock screen
  - Sleep/shutdown/restart
  - Volume/brightness controls

**Success Criteria:**
- ✅ Typing `@` shows extension suggestions
- ✅ 3+ extensions working
- ✅ Extension state persists across restarts
- ✅ Fast switching between extensions (<100ms)

---

### Phase 3: Productivity Extensions (Week 5-6)
**Goal**: Add remaining P1 productivity features

**Tasks:**
- [ ] File search implementation
  - Index home directory with Tantivy
  - Real-time search results
  - File type filtering
  - Quick actions (open, reveal)
- [ ] Web integration
  - Google search with `!g` or direct query
  - YouTube search with `!yt`
  - Open in default browser
- [ ] `@project` extension
  - Project management
  - Task tracking per project
  - Quick links to folders/URLs
- [ ] Keyboard alias system
  - Custom command aliases
  - Text expansion support
  - Save to config file

**Success Criteria:**
- ✅ File search returns results in <200ms
- ✅ Web searches open correctly
- ✅ Can manage 10+ projects
- ✅ Aliases work reliably

---

### Phase 4: Advanced Extensions (Week 7-8)
**Goal**: P2 features for enhanced productivity

**Tasks:**
- [ ] `@color` color picker
  - System-wide color selection
  - Format conversion (HEX/RGB/HSL)
  - Color history
- [ ] `@download` YouTube downloader
  - Video + audio download
  - Format/quality selection
  - Progress tracking
- [ ] `@image` converter/compressor
  - Format conversion
  - Batch compression
  - Resize operations
- [ ] `@music` media controls
  - MPRIS integration
  - Spotify API connection
  - Now playing display
- [ ] `@pomodoro` timer
  - Customizable intervals
  - Desktop notifications
  - Session statistics

**Success Criteria:**
- ✅ All P2 extensions functional
- ✅ Spotify integration works
- ✅ YouTube downloads succeed
- ✅ No performance degradation

---

### Phase 5: Polish & Distribution (Week 9-12)
**Goal**: Production-ready v1.0 release

**Tasks:**
- [ ] Performance optimization
  - Reduce memory footprint to <50MB
  - Sub-50ms launcher activation
  - Optimize icon loading
- [ ] Cross-distro packaging
  - AppImage build
  - Flatpak manifest
  - AUR package (Arch)
  - .deb package (Debian/Ubuntu)
  - .rpm package (Fedora/RHEL)
- [ ] Documentation
  - User guide
  - Extension development docs
  - Installation instructions
  - Contribution guidelines
- [ ] Testing & QA
  - Unit tests for core modules
  - Integration tests
  - Multi-distro testing
  - Accessibility audit
- [ ] Marketing materials
  - Project website/landing page
  - Demo video/GIF
  - GitHub README polish
  - Social media announcement

**Success Criteria:**
- ✅ Passes all quality gates
- ✅ Available on 3+ package managers
- ✅ Complete documentation
- ✅ Beta testing with 20+ users

---

### Phase 6: v1.0 Launch & Beyond (Week 13+)
**Goal**: Public release and community growth

**Milestones:**
- **Week 13**: v1.0.0 release
- **Week 14-16**: Bug fixes from user feedback
- **Week 17-20**: Community extension ecosystem
- **Week 20+**: v1.1 with user-requested features

**Post-Launch Roadmap:**
- Extension marketplace
- Cloud sync (optional)
- Custom themes
- Plugin SDK
- Mobile companion app (stretch goal)

---

### Risk Buffer
- **+2 weeks**: Window manager edge cases (Wayland compositors)
- **+1 week**: Clipboard monitoring reliability issues
- **+1 week**: Extension system refactoring
- **Total Contingency**: 4 weeks

**Estimated Total Timeline**: 12-16 weeks to v1.0

---

## 11. Open Questions

1. **What will be the default global shortcut keys?**
   - Current: Needs configuration system
   - Suggestion: Super+Space (main), Super+E (emoji), Super+V (clipboard)

2. **Should we support multiple language inputs for search?**
   - Consider i18n for emoji names
   - UI localization priority?

3. **What's the maximum clipboard history size?**
   - Current: No limit set
   - Recommendation: 100-500 items with configurable limit

4. **Should extensions be bundled or downloadable?**
   - Phase 1: All bundled (core extensions)
   - Phase 2: Plugin marketplace for third-party

5. **Will there be telemetry/analytics (opt-in)?**
   - Crash reports?
   - Usage statistics?
   - Privacy-first approach required

6. **What's the licensing model?**
   - Recommendation: Open source (MIT or GPL-3.0)
   - Allow forks and contributions

7. **How to handle Wayland compositor differences?**
   - GNOME, KDE, Sway have different capabilities
   - Fallback strategies needed

8. **Should we support theming?**
   - Current: Hardcoded Catppuccin Mocha
   - Future: Theme system with presets

---

## 12. Implementation Notes (Based on New Architecture)

### 12.1 New Modular Architecture

#### Complete Separation: Backend + Frontend

**Current State (Monolithic):**
```
src/main.rs: 755 lines (ALL backend logic)
ui/palette.slint: 472 lines (ALL UI)
```

**Target State (Modular):**
```
Backend (Rust):
  src/main.rs: 55 lines (coordinator)
  src/plugins/app_launcher.rs: 250 lines
  src/plugins/emoji.rs: 120 lines
  src/plugins/clipboard_history.rs: 150 lines

Frontend (Slint):
  ui/palette.slint: 100 lines (shell)
  ui/plugin/app_launcher.slint: 120 lines
  ui/plugin/emoji.slint: 150 lines
  ui/plugin/clipboard_history.slint: 80 lines
```

**Each Feature Gets Two Files:**
```
App Launcher Feature:
├── src/plugins/app_launcher.rs       (Backend - 250 lines)
└── ui/plugin/app_launcher.slint      (Frontend - 120 lines)

Emoji Feature:
├── src/plugins/emoji.rs              (Backend - 120 lines)
└── ui/plugin/emoji.slint             (Frontend - 150 lines)

Clipboard Feature:
├── src/plugins/clipboard_history.rs  (Backend - 150 lines)
└── ui/plugin/clipboard_history.slint (Frontend - 80 lines)
```

#### File Structure Benefits

**Before:**
- ❌ All logic in main.rs (hard to find bugs)
- ❌ All UI in palette.slint (mixed together)
- ❌ Difficult to test individual features
- ❌ Tight coupling between features

**After:**
- ✅ One plugin per file (easy debugging)
- ✅ One UI per plugin (clear separation)
- ✅ Independent testing per feature
- ✅ Loose coupling via trait interface
- ✅ Easy to add/remove features

#### Raycast-Style Configuration System

**Per-Plugin Configuration:**
Each plugin has complete, independent settings:

```rust
// src/core/config.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub global_hotkey: Option<String>,    // e.g., "Super+E"
    pub in_app_hotkey: Option<String>,    // e.g., "Ctrl+E"
    pub aliases: Vec<String>,             // e.g., ["@emoji", "@e"]
    pub settings: HashMap<String, Value>, // Plugin-specific
}
```

**Example Configuration (TOML):**
```toml
[plugins.emoji]
enabled = true
global_hotkey = "Super+E"
in_app_hotkey = "Ctrl+E"
aliases = ["@emoji", "@e", "emoji"]

[plugins.emoji.settings]
recent_count = 20
default_category = "smileys"
show_names = true
```

**Settings UI Structure:**
```
ui/settings/plugins.slint (Main configuration page)
├── Plugin search bar
├── Plugin list (collapsible cards)
│   ├── App Launcher card
│   │   ├── Enable toggle
│   │   ├── Global hotkey recorder
│   │   ├── In-app hotkey recorder
│   │   ├── Alias editor
│   │   └── Plugin-specific settings
│   ├── Emoji card (same structure)
│   └── Clipboard card (same structure)
└── Conflict detection warnings

ui/settings/components/
├── plugin-card.slint       (Reusable plugin card)
├── hotkey-input.slint      (Shortcut recorder)
└── alias-editor.slint      (Add/remove aliases)
```

**Configuration Features:**
- ✅ Each plugin: enable/disable toggle
- ✅ Each plugin: custom global hotkey
- ✅ Each plugin: custom in-app hotkey
- ✅ Each plugin: multiple aliases (add/remove)
- ✅ Each plugin: specific settings
- ✅ Real-time conflict detection
- ✅ Auto-save to TOML
- ✅ No restart needed (hot reload)

### 12.2 Window Type Implementation

#### Two Window Type System

**Type 1: Overlay/Utility Windows** (Hidden from Dock)
```rust
// src/core/window_manager.rs
pub fn configure_overlay_window(window: &slint::Window) {
    #[cfg(target_os = "linux")]
    {
        use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
        
        match window.window_handle().raw_window_handle() {
            RawWindowHandle::Xlib(handle) => {
                configure_x11_overlay(handle.window);
            }
            RawWindowHandle::Wayland(handle) => {
                configure_wayland_overlay(handle.surface);
            }
            _ => eprintln!("Unsupported window system"),
        }
    }
}

#[cfg(target_os = "linux")]
fn configure_x11_overlay(window_id: u64) {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::*;
    
    let (conn, _) = x11rb::connect(None).unwrap();
    
    // Set window type to utility
    let type_atom = conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE").unwrap().reply().unwrap().atom;
    let utility_atom = conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_UTILITY").unwrap().reply().unwrap().atom;
    
    conn.change_property32(
        PropMode::REPLACE,
        window_id as u32,
        type_atom,
        AtomEnum::ATOM,
        &[utility_atom],
    ).unwrap();
    
    // Skip taskbar
    let state_atom = conn.intern_atom(false, b"_NET_WM_STATE").unwrap().reply().unwrap().atom;
    let skip_taskbar = conn.intern_atom(false, b"_NET_WM_STATE_SKIP_TASKBAR").unwrap().reply().unwrap().atom;
    let skip_pager = conn.intern_atom(false, b"_NET_WM_STATE_SKIP_PAGER").unwrap().reply().unwrap().atom;
    let above = conn.intern_atom(false, b"_NET_WM_STATE_ABOVE").unwrap().reply().unwrap().atom;
    
    conn.change_property32(
        PropMode::REPLACE,
        window_id as u32,
        state_atom,
        AtomEnum::ATOM,
        &[skip_taskbar, skip_pager, above],
    ).unwrap();
    
    conn.flush().unwrap();
}
```

**Type 2: Normal Windows** (Visible in Dock)
```slint
// ui/onboarding.slint
export component OnboardingWindow inherits Window {
    title: "Welcome to Palette";
    icon: @image-url("../assets/icons/app-icon.png");
    
    // Standard window - will show in taskbar automatically
    width: 800px;
    height: 600px;
}
```

### 12.3 Shortcut System Implementation

#### Two-Tier Shortcut Architecture

**Global Shortcuts (System-wide)**
```rust
// src/core/shortcuts.rs
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};

pub struct ShortcutManager {
    manager: GlobalHotKeyManager,
    hotkeys: HashMap<String, HotKey>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        ShortcutManager {
            manager: GlobalHotKeyManager::new().unwrap(),
            hotkeys: HashMap::new(),
        }
    }
    
    pub fn register_global(&mut self, id: &str, config: &str) -> Result<(), ShortcutError> {
        // Parse "Super+Space" → HotKey
        let hotkey = self.parse_shortcut(config)?;
        self.manager.register(hotkey)?;
        self.hotkeys.insert(id.to_string(), hotkey);
        Ok(())
    }
    
    pub fn load_from_config(&mut self, config: &ShortcutConfig) {
        // Register all global shortcuts from config.toml
        self.register_global("main_launcher", &config.global.main_launcher);
        self.register_global("emoji_picker", &config.global.emoji_picker);
        self.register_global("clipboard_history", &config.global.clipboard_history);
        // ... etc
    }
}
```

**In-App Shortcuts (Launcher active only)**
```rust
// Handled in Slint UI layer
// ui/palette.slint
export component PaletteWindow inherits Window {
    callback shortcut-triggered(string);
    
    FocusScope {
        key-pressed(event) => {
            // Ctrl+K → command palette
            if (event.modifiers.control && event.text == "k") {
                root.shortcut-triggered("command_palette");
                return accept;
            }
            // Ctrl+, → settings
            if (event.modifiers.control && event.text == ",") {
                root.shortcut-triggered("settings");
                return accept;
            }
            return reject;
        }
    }
}
```

#### Alias System
```rust
// src/core/config.rs
#[derive(Debug, Deserialize)]
pub struct AliasConfig {
    pub emoji: Vec<String>,        // ["@emoji", "@e", "emoji"]
    pub clipboard: Vec<String>,    // ["@clipboard", "@clip", "@c"]
    pub note: Vec<String>,         // ["@note", "@n"]
    // ... etc
}

impl PluginRegistry {
    pub fn find_by_alias(&self, query: &str) -> Option<&dyn Plugin> {
        for plugin in self.plugins.values() {
            // Check exact trigger match
            if query == plugin.trigger() {
                return Some(plugin.as_ref());
            }
            
            // Check aliases
            for alias in plugin.aliases() {
                if query == alias {
                    return Some(plugin.as_ref());
                }
            }
        }
        None
    }
}
```

### 12.4 Plugin Loading System

#### Dynamic Plugin Registration
```rust
// src/plugins/mod.rs
pub fn load_all_plugins(config: &AppConfig) -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    
    // Register plugins based on enabled status in config
    if config.plugins.app_launcher.enabled {
        registry.register(Box::new(AppLauncherPlugin::new()));
    }
    
    if config.plugins.emoji.enabled {
        registry.register(Box::new(EmojiPlugin::new()));
    }
    
    if config.plugins.clipboard.enabled {
        registry.register(Box::new(ClipboardPlugin::new()));
    }
    
    // ... load all other plugins
    
    // Initialize all registered plugins
    for plugin in registry.plugins.values_mut() {
        if let Err(e) = plugin.init() {
            eprintln!("Failed to initialize plugin {}: {}", plugin.name(), e);
        }
    }
    
    registry
}
```

#### Plugin Hot-Reload (Development)
```rust
// src/core/daemon.rs
#[cfg(debug_assertions)]
pub fn watch_plugin_changes(&mut self) {
    use notify::{Watcher, RecursiveMode, watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    
    watcher.watch("src/plugins", RecursiveMode::Recursive).unwrap();
    
    std::thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(_event) => {
                    println!("Plugin file changed, triggering rebuild...");
                    // Trigger cargo build and reload
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        }
    });
}
```

### 12.5 Configuration Management

#### TOML Config Structure
```toml
# ~/.config/wayland-palette/config.toml

[general]
first_run = false
theme = "catppuccin-mocha"
show_tray_icon = true

[shortcuts.global]
main_launcher = "Super+Space"
emoji_picker = "Super+E"
clipboard_history = "Super+V"
settings = "Super+Semicolon"

[shortcuts.in_app]
next_item = "Down"
prev_item = "Up"
execute = "Enter"
cancel = "Escape"
command_palette = "Ctrl+K"

[plugins.app_launcher]
enabled = true
show_package_type = true
index_snap = true
index_flatpak = true

[plugins.emoji]
enabled = true
recent_count = 20
default_category = "smileys"

[plugins.clipboard]
enabled = true
max_items = 100
exclude_passwords = true
```

#### Config Reload Without Restart
```rust
// src/core/config.rs
impl AppConfig {
    pub fn watch_for_changes(&self) -> Result<(), ConfigError> {
        let config_path = Self::config_dir().join("config.toml");
        
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        
        std::thread::spawn(move || {
            while let Ok(_) = rx.recv() {
                println!("Config file changed, reloading...");
                // Reload config without restart
                if let Ok(new_config) = AppConfig::load() {
                    // Apply new config
                }
            }
        });
        
        Ok(())
    }
}
```

### 12.6 UI Component Loading

#### Dynamic UI Loading Based on Plugin
```rust
// src/main.rs
fn show_plugin_ui(plugin_id: &str) -> Result<(), SlintError> {
    match plugin_id {
        "app_launcher" => {
            slint::include_modules!();
            let ui = AppLauncherUI::new()?;
            configure_overlay_window(&ui.window());
            ui.show()?;
        }
        "emoji" => {
            slint::include_modules!();
            let ui = EmojiUI::new()?;
            configure_overlay_window(&ui.window());
            ui.show()?;
        }
        // ... etc
        _ => return Err("Unknown plugin".into()),
    }
    Ok(())
}
```

### 12.7 Migration Roadmap

#### Phase 1: Extract Core Systems (Week 1)
```bash
# Step 1: Create directory structure
mkdir -p src/{core,plugins,utils,models}
mkdir -p ui/{settings,plugin}

# Step 2: Move icon resolution to utils
# Extract find_icon_path() → src/utils/icon_resolver.rs

# Step 3: Move config to core
# Create src/core/config.rs with AppConfig struct

# Step 4: Move shortcut handling to core
# Create src/core/shortcuts.rs
```

#### Phase 2: Extract Plugins (Week 2)
```bash
# Step 5: Extract app launcher
# main.rs → src/plugins/app_launcher.rs
# Move AppEntry, desktop parsing, fuzzy search

# Step 6: Extract emoji picker
# main.rs → src/plugins/emoji.rs
# Move EmojiData, database loading, grid navigation

# Step 7: Extract clipboard
# main.rs → src/plugins/clipboard_history.rs
# Add clipboard monitoring logic
```

#### Phase 3: Refactor UI (Week 2-3)
```bash
# Step 8: Split palette.slint
# ui/palette.slint → Main overlay window only
# Extract to ui/plugin/app_launcher.slint
# Extract to ui/plugin/emoji.slint

# Step 9: Create settings UI
# New ui/main_settings.slint
# New ui/settings/general.slint

# Step 10: Create onboarding UI
# New ui/onboarding.slint
```

### 12.8 Testing Strategy

#### Unit Tests Per Plugin
```rust
// src/plugins/emoji.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_emoji_search() {
        let mut plugin = EmojiPlugin::new();
        plugin.init().unwrap();
        
        let results = plugin.search("smile");
        assert!(results.len() > 0);
        assert!(results[0].name.contains("smile"));
    }
    
    #[test]
    fn test_emoji_categories() {
        let plugin = EmojiPlugin::new();
        let categories = plugin.get_categories();
        assert!(categories.contains(&"smileys"));
    }
}
```

#### Integration Tests
```rust
// tests/integration_test.rs
#[test]
fn test_plugin_registry() {
    let config = AppConfig::default();
    let registry = load_all_plugins(&config);
    
    // Test plugin loading
    assert!(registry.get_by_trigger("@emoji").is_some());
    
    // Test alias resolution
    assert!(registry.find_by_alias("@e").is_some());
}
```

### 12.9 Performance Optimizations

#### Lazy Plugin Loading
```rust
// Only initialize plugins when first accessed
pub struct LazyPlugin {
    plugin: OnceCell<Box<dyn Plugin>>,
    factory: Box<dyn Fn() -> Box<dyn Plugin>>,
}

impl LazyPlugin {
    fn get(&self) -> &dyn Plugin {
        self.plugin.get_or_init(|| (self.factory)()).as_ref()
    }
}
```

#### Parallel Plugin Initialization
```rust
use rayon::prelude::*;

pub fn init_all_plugins_parallel(plugins: &mut [Box<dyn Plugin>]) {
    plugins.par_iter_mut().for_each(|plugin| {
        if let Err(e) = plugin.init() {
            eprintln!("Plugin init error: {}", e);
        }
    });
}
```

### 12.10 Debugging Tools

#### Plugin-Specific Logging
```bash
# Enable debug logging for specific plugin
RUST_LOG=wayland_palette::plugins::emoji=debug cargo run

# Enable all plugin logging
RUST_LOG=wayland_palette::plugins=debug cargo run
```

#### Hot Reload in Development
```bash
# Watch for changes and rebuild
cargo watch -x run

# With Slint live preview
slint-viewer ui/palette.slint &
cargo watch -x run
```

---

## 13. Appendix

### 13.1 Project Naming
**Current Internal Name**: `wayland-palette`  
**Suggested Marketing Names**:
- Palette (simple, memorable)
- Nexus Launcher
- Flow (emphasizes speed)
- Lumen (light/enlightenment theme)

### 13.2 Competitor Analysis

| Feature | Raycast (macOS) | Albert | Ulauncher | KRunner | **This Project** |
|---------|----------------|--------|-----------|---------|------------------|
| Platform | macOS | Linux | Linux | KDE | **Any Linux** |
| Language | Swift | Python/C++ | Python | C++/QML | **Rust** |
| Framework | AppKit | Qt | GTK | Qt | **Slint** |
| Extensions | ✅ Marketplace | ⚠️ Limited | ✅ Python | ✅ Built-in | **🎯 Planned** |
| Performance | Excellent | Good | Moderate | Good | **Excellent** |
| Memory | ~80MB | ~60MB | ~100MB | ~70MB | **~40MB** |
| Clipboard History | ✅ | ❌ | ⚠️ Extension | ✅ | **✅** |
| Emoji Picker | ✅ | ❌ | ⚠️ Extension | ✅ | **✅** |
| File Search | ✅ | ✅ | ✅ | ✅ | **🎯 Planned** |
| Package Type Labels | ❌ | ❌ | ❌ | ✅ | **✅** |
| Glassmorphism UI | ✅ | ❌ | ❌ | ⚠️ Partial | **✅** |
| Wayland Support | N/A | ⚠️ Partial | ⚠️ Partial | ✅ | **✅** |
| License | Proprietary | GPL-3.0 | GPL-3.0 | LGPL | **TBD (suggest MIT)** |

### 13.3 Differentiation Strategy

**Key Advantages:**
1. **Performance**: Rust + Slint = native speed, low memory
2. **Modern UI**: Glassmorphism, smooth animations, polished design
3. **Package Awareness**: Shows Flatpak/Snap/APT labels
4. **True Wayland**: First-class Wayland support, not afterthought
5. **Batteries Included**: Emoji, clipboard, notes, etc. built-in
6. **Desktop Agnostic**: Works on GNOME, KDE, Sway, etc.

**Unique Features:**
- ✅ Package type labeling (Flatpak/Snap detection)
- ✅ Comprehensive icon resolution (handles all edge cases)
- ✅ Grid-based emoji picker with 2D navigation
- ✅ Catppuccin theme integration
- 🎯 Planned: In-app Pomodoro timer
- 🎯 Planned: YouTube downloader integration
- 🎯 Planned: Image converter/compressor

### 13.4 Technical Deep Dives

#### Why Rust + Slint?
- **Rust**: Memory safety, zero-cost abstractions, fearless concurrency
- **Slint**: GPU-accelerated, declarative UI, small runtime, native performance
- **vs GTK**: No GNOME dependencies, smaller binary, modern API
- **vs Qt**: No C++ complexity, permissive license, easier distribution
- **vs Electron**: 10x smaller binary, 5x less memory, native feel

#### Icon Resolution Edge Cases Handled
```rust
// Handles all of these correctly:
"firefox"                    // Simple name
"/usr/share/pixmaps/app.png" // Absolute path
"org.gnome.Calculator"       // Reverse-DNS
"org.localsend.localsend_app" // Flatpak with underscore
"code"                       // VS Code (checks extensions)
```

#### Clipboard Monitoring Strategy
**Options Evaluated:**
1. ❌ Polling clipboard every N seconds → Resource waste
2. ❌ X11 selection events only → No Wayland
3. ✅ **`arboard` with watcher** → Cross-platform, event-driven
4. ⚠️ `clipboard-master` → Unmaintained

**Chosen Approach**: `arboard::Clipboard::new()` with background thread

### 13.5 Development Tools & Resources

#### Build Commands
```bash
# Development build with hot reload
cargo run

# Release build (optimized)
cargo build --release --locked

# Format code
cargo fmt

# Lint
cargo clippy -- -W clippy::all

# Run with debug output
RUST_LOG=debug cargo run
```

#### Slint Live Preview
```bash
# Install Slint viewer
cargo install slint-viewer

# Preview UI changes in real-time
slint-viewer palette.slint
```

#### Debugging Tips
```rust
// Add to main.rs for verbose logging
env_logger::init();
log::debug!("Search query: {}", query);
```

#### Profiling
```bash
# CPU profiling
cargo flamegraph

# Memory profiling
valgrind --tool=massif target/release/wayland-palette
```

### 13.6 Distribution Checklist

#### AppImage
- [ ] Use `linuxdeploy` for bundling
- [ ] Include Slint runtime
- [ ] Embed icon and .desktop file
- [ ] Test on Ubuntu 20.04+ and Fedora 38+

#### Flatpak
- [ ] Create manifest file
- [ ] Submit to Flathub
- [ ] Include permissions for clipboard, hotkeys
- [ ] Add org.freedesktop.Platform runtime

#### AUR (Arch User Repository)
- [ ] Write PKGBUILD
- [ ] Upload to AUR
- [ ] Maintain dependencies list

#### Snap
- [ ] Create snapcraft.yaml
- [ ] Handle confinement (classic vs strict)
- [ ] Test hotkey registration

#### Debian/Ubuntu (.deb)
- [ ] Create debian/ directory structure
- [ ] Write control file
- [ ] Add postinst script for autostart
- [ ] Upload to PPA

### 13.7 Community & Marketing

#### Launch Strategy
1. **Week 1**: Beta announcement on Reddit r/linux, r/rust
2. **Week 2**: Demo video on YouTube
3. **Week 3**: Submit to Hacker News, Lobsters
4. **Week 4**: Blog post series (implementation details)
5. **Week 5**: Reach out to Linux YouTubers (The Linux Experiment, DistroTube)

#### Content Ideas
- "Building a Raycast Clone in Rust"
- "Why Slint is Perfect for Linux Desktop Apps"
- "Handling Icons on Linux: A Complete Guide"
- "Fuzzy Search Performance: Nucleo vs Fzf vs Skim"

#### Potential Partnerships
- Feature in Distro showcase (Manjaro, Pop!_OS)
- Integration with tiling window managers (i3, Sway configs)
- Theme collaboration with Catppuccin project

### 13.8 License Recommendation

**Recommended: MIT License**

**Rationale:**
- ✅ Permissive, allows commercial use
- ✅ Easy for distros to package
- ✅ Encourages contributions
- ✅ Compatible with Slint (MIT/GPL dual-license)
- ✅ Allows proprietary extensions (monetization path)

**Alternative: GPL-3.0**
- ✅ Ensures all forks remain open source
- ✅ Stronger copyleft protection
- ❌ May limit adoption by proprietary projects

### 13.9 Future Vision (v2.0+)

**Extension Marketplace**
- Web-based extension store
- One-click installation
- Extension ratings and reviews
- Revenue sharing for developers

**Cloud Sync** (Optional)
- End-to-end encrypted sync
- Clipboard history across devices
- Settings and shortcuts sync
- Self-hosted option

**AI Integration**
- Natural language commands
- Smart file suggestions
- Contextual actions
- Local LLM integration (privacy-first)

**Mobile Companion**
- Android/iOS app
- Remote clipboard access
- Command execution
- File transfer

**Advanced Features**
- Window management (tiling shortcuts)
- Screenshot OCR
- Translation integration
- Calculator with unit conversion
- Cryptocurrency prices
- Weather integration
- Email search

---

**Document Owner**: Development Team  
**Last Updated**: March 13, 2026  
**Next Review**: After Phase 1 completion (Week 2)  
**Feedback**: GitHub Issues or Discord
