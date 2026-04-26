# TODO

## Goal

Implement settings view with default download directory picker and quality preference, XDG-compliant TOML config, auto-save on blur, and global CSS toast notifications.

## Tasks

### 1. Git Branch

- [x] Create local git branch `feature/settings-view`

### 2. Dependencies

- [x] Add `toml = "0.8"` to Cargo.toml (already present)

### 3. Config Module (`src/config.rs`)

- [x] Create `src/config.rs` with:
  - `AppConfig` struct: `default_download_dir: String`, `quality_preference: String`
  - `load_config()` → reads `$XDG_CONFIG_HOME/medow/config.toml` (falls back to `$HOME/.config/medow/config.toml`)
  - `save_config(config: &AppConfig)` → writes TOML, creates dir if needed
  - `default_config()` → returns AppConfig with sensible defaults

### 4. Toast System (`src/toast.rs`)

- [x] Create `src/toast.rs` with:
  - `ToastType` enum: `Success`, `Error(String)`
  - `Toast` struct: `id: u64`, `message: String`, `type: ToastType`
  - Global `TOASTS: GlobalSignal<Vec<Toast>>` signal
  - `show_toast(message: &str, type: ToastType)` fn
  - `dismiss_toast(id: u64)` fn
- [x] Create CSS toast styles in `assets/main.css`:
  - Success toast: green bg, fades out after 3s (CSS animation + transition)
  - Error toast: red bg, remains until closed, has x button
  - Toast container: fixed position, top-center (transform translate-x-50% left-1/2)
- [x] Add `<ToastContainer />` component to `App` in `main.rs`

### 5. Settings View (`src/settings_view.rs`)

- [x] Replace `settings_view.rs` with:
  - Header bar (reuse navigation pattern)
  - Default download directory: read-only input + "Browse" button → `rfd` file dialog (directory selection)
  - Quality preference: dropdown/select (HD, SD, LQ)
  - Auto-save on `onblur` for both fields → calls `config::save_config()` → triggers `show_toast()`
  - On mount: load config via `config::load_config()`, populate fields

### 6. Integration

- [x] Wire `config::load_config()` in `main.rs` App component on mount
- [x] Store loaded config in AppState or global signals
- [x] Apply `quality_preference` to search results mapping in `search_logic.rs` (use preference instead of hardcoded SD/HD/LQ logic)
- [x] Export and use `TOASTS` signal in App for rendering

### 7. Build & Verify

- [x] Run `dx serve --platform desktop` to verify build
- [x] Test: open settings, change values, blur → toast appears, file dialog opens
- [x] Test: config file created at `$HOME/.config/medow/config.toml`

## Notes

- XDG spec: config at `$XDG_CONFIG_HOME/medow/config.toml`, fallback `$HOME/.config/medow/config.toml`
- TOML format for config
- Auto-save on blur (not on every keystroke)
- CSS toast messages only (no desktop notifications via notify-rust)
- `rfd` crate already in dependencies for file dialog
- Toast uses global signal for cross-module access
- Quality preference: HD, SD, LQ (matches existing quality labels)
