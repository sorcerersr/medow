# TODO

## Goal

Add pagination controls (prev/next buttons + page numbers) and page info text to the search view footer.

## Tasks

### 1. Wire up pagination controls in search_view.rs

- [x] Add prev/next button handlers in `header_bar` using `pagination.previous_offset()` / `pagination.next_offset()`
- [x] Replace footer placeholder text with: page info string (using `pagination.info()`), prev button, page number buttons (1 to `total_pages()`), next button
- [x] Style prev/next buttons as disabled when on first/last page respectively

## Notes

- Page size is fixed at 15 (from `pagination.page_size()`)
- Search always resets offset to 0 (existing behavior preserved)
- Uses Pico CSS framework (already in project)
- Footer already has two `<ul>` elements — left ul for info+prev, right ul for next+page numbers
