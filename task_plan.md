# Task Plan: Download Feature

## Goal

Implement download feature with persistent selection across pagination, download/reset buttons, and async download view.

## Requirements

1. Selection persists across all pages on page switches
2. "Download" and "Reset" buttons below search results, right-aligned
3. Download button starts downloads, switches to downloads view
4. Downloads view shows status per file
5. Reset button clears all selections across all pages
6. Non-blocking/async downloads
7. Destination = config download path
8. Output filename = title (spaces→underscores) + original extension

## Phases

### Phase 1: Global Selection State (complete)

- Create `downloads.rs` module with global selection signal
- Selection state must survive pagination resets
- Track all items globally, not per-page

### Phase 2: Search View Integration (complete)

- Wire checkboxes to global selection
- Add download/reset buttons below pagination (right-aligned)
- Download → collect selected items, switch to downloads view, start downloads
- Reset → clear all selections

### Phase 3: Download Engine (complete)

- Async download using reqwest
- Filename: title with underscores + extension from URL
- Track per-file status (idle/Downloading/Complete/Failed)
- Non-blocking

### Phase 4: Downloads View (complete)

- Show table with file info + status per download
- Progress indicators
- Status: idle, downloading, complete, failed

### Phase 5: CSS Styling (complete)

- Right-align download/reset buttons
- Download status styling

## Dependencies

- Need reqwest streaming for downloads (already in Cargo.toml)
- Need to add async file writing support

## Errors Encountered

| Error | Attempt | Resolution |
| ----- | ------- | ---------- |
