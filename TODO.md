# TODO

## Goal

Connect pagination to the mediathekview API by reading `total_results` from the search response.

## Tasks

### 1. Wire API total_results into Pagination in search_logic.rs

- [x] Read `search_result.query_info.total_results` from API response
- [x] Store it in `pagination.write().total` so pagination UI calculates correct page counts
- [x] Verify `total_pages()` and `has_next_page()` work correctly with real API data
- [x] Add `visible_pages()` method — returns max 5 page numbers centered on current page
- [x] Update footer to use `visible_pages()` instead of all pages

## Notes

- API response: `QueryResult.query_info.total_results` (u64) = total matching items across all pages
- API response: `QueryResult.results` = items for current page only (already used)
- `size(15)` + `offset` already passed correctly to API
- Only missing link: `pagination.total` was never set from API response
