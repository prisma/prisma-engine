use crate::{Field, ModelRef};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrderBy {
    pub field: Field,
    pub sort_order: SortOrder,
}

pub trait IntoOrderBy {
    fn into_order_by(self, model: ModelRef) -> OrderBy;
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn is_ascending(self) -> bool {
        match self {
            Self::Ascending => true,
            _ => false,
        }
    }

    pub fn is_descending(self) -> bool {
        match self {
            Self::Descending => true,
            _ => false,
        }
    }

    pub fn abbreviated(self) -> &'static str {
        match self {
            SortOrder::Ascending => "ASC",
            SortOrder::Descending => "DESC",
        }
    }
}
