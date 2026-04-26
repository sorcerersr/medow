# Findings

## Architecture

- Dioxus 0.7 desktop app (Rust)
- Global state via `GlobalSignal<AppState>` in main.rs
- Config via `GlobalSignal<AppConfig>` in main.rs
- Navigation via `APP_STATE.write().view = view`

## Key Types

- `SearchItem` — has `selected: bool` field already
- `Pagination` — local signal in search_view, gets reset on page switch
- `AppConfig` — has `default_download_dir` field

## Current State

- `SearchItem.selected` exists but only lives in Pagination (local signal)
- Pagination signal resets on each page switch → selections lost
- `downloads_view.rs` is a stub

## Discovery: Selection Persistence Strategy

- Pagination is local to search_view (use_signal)
- Each page switch clears and repopulates items
- Need GLOBAL selection set outside pagination
- Options:
  1. Global signal with Set of selected item identifiers (title+timestamp)
  2. Map title→selected state globally
  3. Store selected titles in global signal, merge on page load

## Decision: Use global signal with Set<String> of selected titles

- Simple, survives pagination
- Title is unique enough for this use case
- On page switch, search_logic merges global selection into new items

## Download Engine

- Use reqwest for HTTP downloads
- Need tokio for async file I/O
- Add `tokio` with `fs` feature to dependencies
- Download URL from `SearchItem.video_url`
- Filename: title.replace(" ", "\_") + extension from URL path
