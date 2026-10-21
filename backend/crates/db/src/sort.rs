//! Sort types shared by list queries.

/// Direction of a sorted list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Smallest or oldest first.
    Asc,
    /// Largest or newest first.
    Desc,
}

impl SortDirection {
    /// Returns the SQL keyword for this direction.
    pub(crate) fn as_sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}
