# TODO

## Goal

Add first-page and last-page jump buttons to the pagination footer.

## Tasks

### 1. Add `first_offset()` and `last_offset()` to Pagination (pagination.rs)

- [x] Add `first_offset()` — returns `Some(0)` if offset > 0, else `None`
- [x] Add `last_offset()` — returns last valid offset (total - page_size rounded down), or `None` if on last page

### 2. Add first/last buttons to footer (search_view.rs)

- [x] Split footer into 3 ul elements for centered layout:
  - Left ul: info, "« First", "← Prev"
  - Center ul: page number buttons
  - Right ul: "Next →", "Last »"
- [ ] Disable "« First" when on page 1
- [ ] Disable "Last »" when on last page

## Notes

- Current layout (2 ul): `info ← Prev | 1 2 3 4 5 | Next →`
- Desired layout (3 ul): `info « First ← Prev | 1 2 3 4 5 | Next → Last »`
- Page numbers centered between prev/next via middle ul
- `first_offset()` is always `0` (if not already there)
- `last_offset()` = `total` rounded down to page boundary, but only if not already on last page
