# TODO

## Goal

Fix `AlreadyBorrowed` crash when clicking pagination buttons and replace all `unwrap()` calls with graceful error handling.

## Tasks

### 1. Fix borrow conflict in footer onclick handlers (search_view.rs)

- [x] Read offset into a local variable before calling `perform_search` — drops the `ReadGuard` before `.write()` is called
- [x] Apply fix to all 3 footer handlers: Next →, ← Prev, page number buttons

### 2. Fix MEDOW_USER_AGENT unwrap in search_logic.rs

- [x] Replace `MEDOW_USER_AGENT.try_into().unwrap()` with `match` → set `APP_STATE.error` and return early

### 3. Fix search result mapping unwrap in search_logic.rs

- [ ] Replace `item.url_video_low.unwrap_or(String::from(""))` — already uses `unwrap_or`, safe
- [x] Replace any other `unwrap()` calls with graceful error handling

### 4. Improve error handling in search_logic.rs

- [x] Log all errors with `eprintln!` (not just `println!` for search start)
- [x] Ensure all error paths set `APP_STATE.write().is_loading = false`

## Notes

- The `AlreadyBorrowed` error: `pagination.read()` returns a `ReadGuard` that lives until the end of the `if` block in `if let Some(offset) = pagination.read().next_offset()`. When `perform_search` calls `pagination.write()`, the read guard is still active → panic.
- Fix: `let offset = pagination.read().next_offset(); if let Some(offset) = offset { ... }` — guard dropped at end of statement.
- `MEDOW_USER_AGENT.try_into().unwrap()` can panic if the string is too long for HeaderValue (max 64KB, but still — should be graceful).
