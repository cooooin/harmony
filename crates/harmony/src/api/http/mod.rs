pub mod request;
pub mod response;

pub mod prelude {
    use super::*;

    pub use request::body::Json;
    pub use request::headers::{Claim, Path, Query};

    pub use response::{Response, ResponseResult};

    pub use state::State;
}

pub mod state {
    use std::sync::Arc;

    use crate::time::timestamp;

    pub type State = axum::extract::State<Arc<StateInner>>;

    pub struct StateInner {}

    impl StateInner {
        pub fn new() -> Self {
            Self {}
        }

        pub fn timestamp_millis(&self) -> u128 {
            timestamp().as_millis()
        }
    }
}

/// Converts `page` and `page_size` into `limit` and `offset` for pagination.
///
/// # Arguments
/// - `page`: The current page number (1-based).
/// - `page_size`: The number of items per page.
///
/// # Returns
/// A tuple `(limit, offset)` where:
/// - `limit` is the number of items per page (i.e., `page_size`).
/// - `offset` is the starting point for fetching records.
pub fn paginate(page: usize, page_size: usize) -> (usize, usize) {
    let page = if page == 0 { 1 } else { page }; // Handle page = 0 by defaulting to 1
    let page_size = if page_size == 0 { 64 } else { page_size }; // Handle page_size = 0 by defaulting to 64
    let offset = (page - 1) * page_size; // Calculate offset
    (page_size, offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test pagination with valid inputs.
    #[test]
    fn test_paginate_valid() {
        let (limit, offset) = paginate(3, 64);
        assert_eq!(limit, 64); // Limit should be equal to page_size
        assert_eq!(offset, 128); // Offset should be (page - 1) * page_size
    }

    /// Test pagination with page = 0 (should default to page = 1).
    #[test]
    fn test_paginate_page_zero() {
        let (limit, offset) = paginate(0, 64);
        assert_eq!(limit, 64); // Limit should be equal to page_size
        assert_eq!(offset, 0); // Offset should be 0 since page defaults to 1
    }

    /// Test pagination with page_size = 0 (should default to page_size = 64).
    #[test]
    fn test_paginate_page_size_zero() {
        let (limit, offset) = paginate(3, 0);
        assert_eq!(limit, 64); // Limit should default to 64
        assert_eq!(offset, 128); // Offset should be (page - 1) * default page_size
    }

    /// Test pagination with both page and page_size as 0 (should default to page = 1 and page_size = 64).
    #[test]
    fn test_paginate_both_zero() {
        let (limit, offset) = paginate(0, 0);
        assert_eq!(limit, 64); // Limit should default to 64
        assert_eq!(offset, 0); // Offset should be 0 since page defaults to 1
    }

    /// Test pagination with large values.
    #[test]
    fn test_paginate_large_values() {
        let (limit, offset) = paginate(100, 1000);
        assert_eq!(limit, 1000); // Limit should be equal to page_size
        assert_eq!(offset, 99000); // Offset should be (page - 1) * page_size
    }
}
