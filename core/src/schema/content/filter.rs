use super::prelude::*;

use super::SameAsContent;

#[derive(Debug, Clone, PartialEq)]
pub struct FilterContent {
    pub on: Box<Content>,
    pub closure: Vec<SameAsContent>,
    pub query: String,
}

mod filter_content {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub(super) struct SerdeFilterContent {
	on: Content,
	query: String,
    }

    impl SerdeFilterContent {
	pub(super) fn into_filter_content(self) -> Result<FilterContent> {
	    // 1. Extract capture groups for exprs like ${...}
	    // 2. Parse captured groups as FieldRefs and wrap them in SameAsContents
	    // 3. Profit
	    todo!()
	}

	pub(super) fn from_filter_content(fc: FilterContent) -> Self {
	    Self {
		on: *fc.on,
		query: fc.query,
	    }
	}
    }
}
