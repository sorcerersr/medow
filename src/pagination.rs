#[derive(Clone, Debug, PartialEq)]
pub struct SearchItem {
    pub selected: bool,
    pub title: String,
    pub topic: String,
    pub timestamp: String,
    pub duration: String,
    pub quality: String,
    pub video_url: String,
}

pub struct Pagination {
    pub total: usize,
    pub offset: usize,
    pub items: Vec<SearchItem>,
}

impl Pagination {
    /// Creates a new Pagination instance with default values
    pub fn new() -> Self {
        Pagination {
            total: 0,
            offset: 0,
            items: Vec::new(),
        }
    }

    /// Gets the page size (fixed at 15 as specified)
    pub fn page_size(&self) -> usize {
        15
    }

    /// Calculates the total number of pages
    pub fn total_pages(&self) -> usize {
        if self.total == 0 {
            0
        } else {
            (self.total + self.page_size() - 1) / self.page_size()
        }
    }

    /// Checks if there is a next page
    pub fn has_next_page(&self) -> bool {
        self.offset + self.page_size() < self.total
    }

    /// Checks if there is a previous page
    pub fn has_previous_page(&self) -> bool {
        self.offset > 0
    }

    /// Gets the current page number (1-indexed)
    pub fn current_page(&self) -> usize {
        if self.offset == 0 {
            1
        } else {
            (self.offset / self.page_size()) + 1
        }
    }

    /// Gets the next offset for the next page
    pub fn next_offset(&self) -> Option<usize> {
        let next_offset = self.offset + self.page_size();
        if next_offset < self.total {
            Some(next_offset)
        } else {
            None
        }
    }

    /// Gets the previous offset for the previous page
    pub fn previous_offset(&self) -> Option<usize> {
        if self.offset > 0 {
            Some(self.offset.saturating_sub(self.page_size()))
        } else {
            None
        }
    }

    /// Gets the number of items on the current page
    pub fn items_on_page(&self) -> usize {
        let remaining = self.total - self.offset;
        if remaining >= self.page_size() {
            self.page_size()
        } else {
            remaining
        }
    }

    /// Checks if the pagination is valid (offset is within bounds)
    pub fn is_valid(&self) -> bool {
        self.offset <= self.total
    }

    /// Gets the range of items for the current page
    pub fn item_range(&self) -> (usize, usize) {
        let start = self.offset;
        let end = (self.offset + self.page_size()).min(self.total);
        (start, end)
    }

    /// Gets pagination info as a string
    pub fn info(&self) -> String {
        format!(
            "Page {}/{} (Items {}-{} of {})",
            self.current_page(),
            self.total_pages(),
            self.offset + 1,
            self.offset + self.items_on_page(),
            self.total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pagination() {
        let pagination = Pagination::new();
        assert_eq!(pagination.total, 0);
        assert_eq!(pagination.offset, 0);
        assert_eq!(pagination.items.len(), 0);
    }

    #[test]
    fn test_page_size() {
        let pagination = Pagination::new();
        assert_eq!(pagination.page_size(), 15);
    }

    #[test]
    fn test_total_pages_empty() {
        let pagination = Pagination {
            total: 0,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.total_pages(), 0);
    }

    #[test]
    fn test_total_pages_single_page() {
        let pagination = Pagination {
            total: 15,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.total_pages(), 1);
    }

    #[test]
    fn test_total_pages_multiple_pages() {
        let pagination = Pagination {
            total: 30,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.total_pages(), 2);
    }

    #[test]
    fn test_total_pages_partial_page() {
        let pagination = Pagination {
            total: 16,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.total_pages(), 2);
    }

    #[test]
    fn test_has_next_page_no_items() {
        let pagination = Pagination {
            total: 0,
            offset: 0,
            items: vec![],
        };
        assert!(!pagination.has_next_page());
    }

    #[test]
    fn test_has_next_page_single_page() {
        let pagination = Pagination {
            total: 15,
            offset: 0,
            items: vec![],
        };
        assert!(!pagination.has_next_page());
    }

    #[test]
    fn test_has_next_page_multiple_pages() {
        let pagination = Pagination {
            total: 30,
            offset: 0,
            items: vec![],
        };
        assert!(pagination.has_next_page());
    }

    #[test]
    fn test_has_next_page_last_page() {
        let pagination = Pagination {
            total: 30,
            offset: 15,
            items: vec![],
        };
        assert!(!pagination.has_next_page());
    }

    #[test]
    fn test_has_previous_page_no_items() {
        let pagination = Pagination {
            total: 0,
            offset: 0,
            items: vec![],
        };
        assert!(!pagination.has_previous_page());
    }

    #[test]
    fn test_has_previous_page_first_page() {
        let pagination = Pagination {
            total: 30,
            offset: 0,
            items: vec![],
        };
        assert!(!pagination.has_previous_page());
    }

    #[test]
    fn test_has_previous_page_middle_page() {
        let pagination = Pagination {
            total: 30,
            offset: 15,
            items: vec![],
        };
        assert!(pagination.has_previous_page());
    }

    #[test]
    fn test_current_page_first_page() {
        let pagination = Pagination {
            total: 30,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.current_page(), 1);
    }

    #[test]
    fn test_current_page_middle_page() {
        let pagination = Pagination {
            total: 30,
            offset: 15,
            items: vec![],
        };
        assert_eq!(pagination.current_page(), 2);
    }

    #[test]
    fn test_next_offset_no_items() {
        let pagination = Pagination {
            total: 0,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.next_offset(), None);
    }

    #[test]
    fn test_next_offset_single_page() {
        let pagination = Pagination {
            total: 15,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.next_offset(), None);
    }

    #[test]
    fn test_next_offset_multiple_pages() {
        let pagination = Pagination {
            total: 30,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.next_offset(), Some(15));
    }

    #[test]
    fn test_next_offset_last_page() {
        let pagination = Pagination {
            total: 30,
            offset: 15,
            items: vec![],
        };
        assert_eq!(pagination.next_offset(), None);
    }

    #[test]
    fn test_previous_offset_no_items() {
        let pagination = Pagination {
            total: 0,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.previous_offset(), None);
    }

    #[test]
    fn test_previous_offset_first_page() {
        let pagination = Pagination {
            total: 30,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.previous_offset(), None);
    }

    #[test]
    fn test_previous_offset_middle_page() {
        let pagination = Pagination {
            total: 30,
            offset: 15,
            items: vec![],
        };
        assert_eq!(pagination.previous_offset(), Some(0));
    }

    #[test]
    fn test_items_on_page_full_page() {
        let pagination = Pagination {
            total: 30,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.items_on_page(), 15);
    }

    #[test]
    fn test_items_on_page_partial_page() {
        let pagination = Pagination {
            total: 16,
            offset: 15,
            items: vec![],
        };
        assert_eq!(pagination.items_on_page(), 1);
    }

    #[test]
    fn test_items_on_page_last_page() {
        let pagination = Pagination {
            total: 30,
            offset: 15,
            items: vec![],
        };
        assert_eq!(pagination.items_on_page(), 15);
    }

    #[test]
    fn test_is_valid_valid_offset() {
        let pagination = Pagination {
            total: 30,
            offset: 15,
            items: vec![],
        };
        assert!(pagination.is_valid());
    }

    #[test]
    fn test_is_valid_invalid_offset() {
        let pagination = Pagination {
            total: 15,
            offset: 20,
            items: vec![],
        };
        assert!(!pagination.is_valid());
    }

    #[test]
    fn test_item_range_full_page() {
        let pagination = Pagination {
            total: 30,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.item_range(), (0, 15));
    }

    #[test]
    fn test_item_range_partial_page() {
        let pagination = Pagination {
            total: 16,
            offset: 15,
            items: vec![],
        };
        assert_eq!(pagination.item_range(), (15, 16));
    }

    #[test]
    fn test_info_empty() {
        let pagination = Pagination {
            total: 0,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.info(), "Page 1/0 (Items 1-0 of 0)");
    }

    #[test]
    fn test_info_single_page() {
        let pagination = Pagination {
            total: 15,
            offset: 0,
            items: vec![],
        };
        assert_eq!(pagination.info(), "Page 1/1 (Items 1-15 of 15)");
    }

    #[test]
    fn test_info_multiple_pages() {
        let pagination = Pagination {
            total: 30,
            offset: 15,
            items: vec![],
        };
        assert_eq!(pagination.info(), "Page 2/2 (Items 16-30 of 30)");
    }
}
