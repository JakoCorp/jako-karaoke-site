//! Building blocks for the text search and `WHERE` clauses of list queries.

use sqlx::{MySql, QueryBuilder};
use uuid::Uuid;

/// Appends `WHERE` for the first condition and `AND` for every following one.
pub struct WhereClause<'builder> {
    builder: &'builder mut QueryBuilder<MySql>,
    started: bool,
}

impl<'builder> WhereClause<'builder> {
    /// Wraps `builder`, which must not yet contain a `WHERE` clause.
    pub fn new(builder: &'builder mut QueryBuilder<MySql>) -> Self {
        Self {
            builder,
            started: false,
        }
    }

    /// Pushes the keyword for the next condition and returns the builder to append it to.
    pub fn condition(&mut self) -> &mut QueryBuilder<MySql> {
        let keyword = if self.started { " AND " } else { " WHERE " };
        self.started = true;
        self.builder.push(keyword)
    }
}

/// Builds the pattern for a case insensitive substring match with `LIKE`.
///
/// Wildcard characters in `q` are escaped, so they match themselves.
pub fn like_pattern(q: &str) -> String {
    let mut pattern = String::with_capacity(q.len() + 2);
    pattern.push('%');
    for character in q.chars() {
        if matches!(character, '\\' | '%' | '_') {
            pattern.push('\\');
        }
        pattern.push(character);
    }
    pattern.push('%');
    pattern
}

/// Appends `<column> LIKE ? OR id = ?`, matching `q` as a substring of `column` or as an exact id.
///
/// The id side matches nothing when `q` is not a UUID. The caller wraps the condition in
/// parentheses when it is combined with others.
pub fn push_text_or_id(builder: &mut QueryBuilder<MySql>, column: &'static str, q: &str) {
    builder
        .push(column)
        .push(" LIKE ")
        .push_bind(like_pattern(q))
        .push(" OR id = ")
        .push_bind(Uuid::parse_str(q).ok());
}
