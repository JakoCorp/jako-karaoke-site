//! Parsing of `filter` query parameters into database clauses.

use api_types::{
    filters::FilterField,
    tags::{PerformanceTagKind, SongTagKind},
};
use chrono::NaiveDate;
use db::filter::{Clause, FilterTarget, Subject, TagScope};
use uuid::Uuid;

use crate::error::ApiError;

/// Conditions parsed from a request's `filter` parameters.
#[derive(Debug, Default)]
pub struct ParsedFilters {
    pub clauses: Vec<Clause>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
}

/// Returns the relation a field reads, or `None` for the date field.
pub fn target_for(field: FilterField) -> Option<FilterTarget> {
    let song_tag = |kind: SongTagKind| FilterTarget::Tag {
        kind: kind.as_str(),
        scope: TagScope::Song,
    };
    let performance_tag = |kind: PerformanceTagKind| FilterTarget::Tag {
        kind: kind.as_str(),
        scope: TagScope::Performance,
    };

    match field {
        FilterField::Singer => Some(FilterTarget::Singer),
        FilterField::OriginalArtist => Some(FilterTarget::OriginalArtist),
        FilterField::Song => Some(FilterTarget::Song),
        FilterField::Genre => Some(song_tag(SongTagKind::Genre)),
        FilterField::Source => Some(song_tag(SongTagKind::Source)),
        FilterField::Language => Some(song_tag(SongTagKind::Language)),
        FilterField::Modifier => Some(performance_tag(PerformanceTagKind::Modifier)),
        FilterField::Instrument => Some(performance_tag(PerformanceTagKind::Instrument)),
        FilterField::Misc => Some(FilterTarget::Tag {
            kind: SongTagKind::Misc.as_str(),
            scope: TagScope::Either,
        }),
        FilterField::Date => None,
    }
}

/// Returns whether `field` can filter the given kind of result.
pub fn applies_to(field: FilterField, subject: Subject) -> bool {
    match subject {
        Subject::Performance => true,
        Subject::Song => matches!(
            field,
            FilterField::Singer
                | FilterField::OriginalArtist
                | FilterField::Genre
                | FilterField::Source
                | FilterField::Language
                | FilterField::Misc
        ),
    }
}

/// Parses `filter` parameters for the given kind of result.
///
/// # Errors
///
/// Returns [`ApiError::BadRequest`] for unknown fields, fields that do not apply to `subject`,
/// malformed IDs or dates, and excluded date ranges.
pub fn parse_filters(raw: &[String], subject: Subject) -> Result<ParsedFilters, ApiError> {
    let mut parsed = ParsedFilters::default();

    for expression in raw {
        let (exclude, rest) = match expression.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, expression.as_str()),
        };
        let (name, raw_values) = rest.split_once(':').ok_or_else(|| {
            ApiError::BadRequest(format!("filter {expression:?} must look like field:value"))
        })?;
        let field: FilterField = name
            .parse()
            .map_err(|()| ApiError::BadRequest(format!("unknown filter field {name:?}")))?;
        if !applies_to(field, subject) {
            return Err(ApiError::BadRequest(format!(
                "filter field {name:?} does not apply here"
            )));
        }
        let values: Vec<&str> = raw_values
            .split(',')
            .filter(|value| !value.is_empty())
            .collect();
        if values.is_empty() {
            return Err(ApiError::BadRequest(format!(
                "filter {expression:?} has no values"
            )));
        }

        match target_for(field) {
            Some(target) => {
                let ids = values
                    .iter()
                    .map(|value| {
                        Uuid::parse_str(value).map_err(|_| {
                            ApiError::BadRequest(format!("invalid id {value:?} for {name}"))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                parsed.clauses.push(Clause {
                    target,
                    ids,
                    exclude,
                });
            }
            None => {
                if exclude {
                    return Err(ApiError::BadRequest(
                        "date filters cannot be excluded".to_string(),
                    ));
                }
                apply_date_ranges(&mut parsed, &values)?;
            }
        }
    }

    Ok(parsed)
}

/// Narrows the date bounds of `parsed` by each `from..to` range.
fn apply_date_ranges(parsed: &mut ParsedFilters, ranges: &[&str]) -> Result<(), ApiError> {
    for range in ranges {
        let (from, to) = range.split_once("..").ok_or_else(|| {
            ApiError::BadRequest(format!("date {range:?} must look like from..to"))
        })?;
        if let Some(from) = parse_date(from)? {
            parsed.date_from = Some(parsed.date_from.map_or(from, |current| current.max(from)));
        }
        if let Some(to) = parse_date(to)? {
            parsed.date_to = Some(parsed.date_to.map_or(to, |current| current.min(to)));
        }
    }
    Ok(())
}

fn parse_date(value: &str) -> Result<Option<NaiveDate>, ApiError> {
    if value.is_empty() {
        return Ok(None);
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| ApiError::BadRequest(format!("invalid date {value:?}, expected YYYY-MM-DD")))
}
