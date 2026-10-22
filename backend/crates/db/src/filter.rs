//! Shared building blocks for filtered list queries and the values offered as filters.
//!
//! A [`Clause`] matches entities through one relation (singers, original artists, songs or
//! tags). The same relation definitions drive both the `WHERE` conditions and the list of values
//! that can be picked, so every offered value matches at least one entity.

use sqlx::{Executor, MySql, QueryBuilder};
use uuid::Uuid;

use crate::error::DbError;
use crate::search::WhereClause;

/// Which join tables a tag relation reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagScope {
    /// Tags attached to songs.
    Song,
    /// Tags attached to performances.
    Performance,
    /// Tags attached to a performance or to any of its songs.
    Either,
}

/// The relation a [`Clause`] or a catalog of values reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterTarget {
    /// Artists that sang in a performance. For songs, artists that sang it in any performance.
    Singer,
    /// Artists credited as the original artist of a song.
    OriginalArtist,
    /// Songs included in a performance. Not applicable when filtering songs.
    Song,
    /// Tags of the given kind.
    Tag {
        /// Kind stored on the join row, for example `genre`.
        kind: &'static str,
        /// Join tables the tag may come from.
        scope: TagScope,
    },
}

/// The kind of entity a list query returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    Performance,
    Song,
}

/// One condition on a relation. Entities match when they relate to any of `ids`, or to none of
/// them when `exclude` is set.
#[derive(Debug, Clone)]
pub struct Clause {
    pub target: FilterTarget,
    pub ids: Vec<Uuid>,
    pub exclude: bool,
}

/// A selectable value for a filter field.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct CatalogValue {
    pub id: Uuid,
    pub name: String,
}

/// Appends `SELECT entity_id, value_id FROM ...` for the relation. Returns `false` and appends
/// nothing when the relation does not exist for `subject`.
fn push_pairs(builder: &mut QueryBuilder<MySql>, subject: Subject, target: FilterTarget) -> bool {
    match (subject, target) {
        (Subject::Performance, FilterTarget::Singer) => {
            builder.push(
                "SELECT performance_id AS entity_id, artist_id AS value_id \
                 FROM performance_singers",
            );
        }
        (Subject::Performance, FilterTarget::OriginalArtist) => {
            builder.push(
                "SELECT ps.performance_id AS entity_id, soa.artist_id AS value_id \
                 FROM performance_songs ps \
                 JOIN song_original_artists soa ON soa.song_id = ps.song_id",
            );
        }
        (Subject::Performance, FilterTarget::Song) => {
            builder.push(
                "SELECT performance_id AS entity_id, song_id AS value_id FROM performance_songs",
            );
        }
        (Subject::Performance, FilterTarget::Tag { kind, scope }) => match scope {
            TagScope::Song => push_performance_song_tags(builder, kind),
            TagScope::Performance => push_performance_tags(builder, kind),
            TagScope::Either => {
                push_performance_song_tags(builder, kind);
                builder.push(" UNION ALL ");
                push_performance_tags(builder, kind);
            }
        },
        (Subject::Song, FilterTarget::Singer) => {
            builder.push(
                "SELECT ps.song_id AS entity_id, singers.artist_id AS value_id \
                 FROM performance_songs ps \
                 JOIN performance_singers singers ON singers.performance_id = ps.performance_id",
            );
        }
        (Subject::Song, FilterTarget::OriginalArtist) => {
            builder.push(
                "SELECT song_id AS entity_id, artist_id AS value_id FROM song_original_artists",
            );
        }
        (Subject::Song, FilterTarget::Tag { kind, scope }) if scope != TagScope::Performance => {
            builder
                .push(
                    "SELECT song_id AS entity_id, tag_id AS value_id FROM song_tags WHERE kind = ",
                )
                .push_bind(kind);
        }
        (Subject::Song, FilterTarget::Song | FilterTarget::Tag { .. }) => return false,
    }
    true
}

fn push_performance_song_tags(builder: &mut QueryBuilder<MySql>, kind: &'static str) {
    builder
        .push(
            "SELECT ps.performance_id AS entity_id, st.tag_id AS value_id \
             FROM performance_songs ps \
             JOIN song_tags st ON st.song_id = ps.song_id WHERE st.kind = ",
        )
        .push_bind(kind);
}

fn push_performance_tags(builder: &mut QueryBuilder<MySql>, kind: &'static str) {
    builder
        .push(
            "SELECT performance_id AS entity_id, tag_id AS value_id \
             FROM performance_tags WHERE kind = ",
        )
        .push_bind(kind);
}

/// Table and name column holding the selectable values of a relation.
fn value_source(target: FilterTarget) -> (&'static str, &'static str) {
    match target {
        FilterTarget::Singer | FilterTarget::OriginalArtist => ("artists", "name"),
        FilterTarget::Song => ("songs", "title"),
        FilterTarget::Tag { .. } => ("tags", "name"),
    }
}

/// Appends the condition for `clause` against `entity_column`.
///
/// A relation that does not exist for `subject` never matches, so its exclusion matches
/// everything.
fn push_clause(
    builder: &mut QueryBuilder<MySql>,
    subject: Subject,
    entity_column: &str,
    clause: &Clause,
) {
    builder.push(entity_column);
    builder.push(if clause.exclude { " NOT IN (" } else { " IN (" });
    builder.push("SELECT p.entity_id FROM (");
    if !push_pairs(builder, subject, clause.target) {
        builder.push("SELECT NULL AS entity_id, NULL AS value_id");
    }
    builder.push(") p WHERE p.value_id IN (");
    let mut separated = builder.separated(", ");
    for id in &clause.ids {
        separated.push_bind(*id);
    }
    builder.push("))");
}

/// Appends a condition for every clause against `entity_column`. Clauses without ids are skipped.
pub fn push_clauses(
    conditions: &mut WhereClause,
    subject: Subject,
    entity_column: &str,
    clauses: &[Clause],
) {
    for clause in clauses.iter().filter(|clause| !clause.ids.is_empty()) {
        push_clause(conditions.condition(), subject, entity_column, clause);
    }
}

/// Lists the values of `target` that at least one entity of `subject` relates to.
///
/// Values are ordered by how many entities use them, then by name. Returns an empty list when
/// the relation does not exist for `subject`.
pub async fn catalog_values(
    executor: impl Executor<'_, Database = MySql>,
    subject: Subject,
    target: FilterTarget,
) -> Result<Vec<CatalogValue>, DbError> {
    let (table, name_column) = value_source(target);
    let mut builder =
        QueryBuilder::new(format!("SELECT v.id AS id, v.{name_column} AS name FROM ("));
    if !push_pairs(&mut builder, subject, target) {
        return Ok(Vec::new());
    }
    builder.push(format!(
        ") p JOIN {table} v ON v.id = p.value_id \
         GROUP BY v.id, v.{name_column} \
         ORDER BY COUNT(DISTINCT p.entity_id) DESC, v.{name_column}"
    ));

    builder
        .build_query_as::<CatalogValue>()
        .fetch_all(executor)
        .await
        .map_err(DbError::from)
}
