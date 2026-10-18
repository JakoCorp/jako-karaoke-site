//! Filter fields and filter catalog response types for list endpoints.
//!
//! List endpoints accept repeated `filter` query parameters of the form
//! `[-]field:value[,value...]`. Values of one parameter are alternatives, separate parameters
//! must all hold, and a leading `-` excludes the matches.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A property list results can be filtered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterField {
    /// Artist that sang in a performance. Values are artist IDs.
    Singer,
    /// Original artist of a song. Values are artist IDs.
    OriginalArtist,
    /// Song included in a performance. Values are song IDs.
    Song,
    /// Song genre tag. Values are tag IDs.
    Genre,
    /// Song source tag such as anime or game. Values are tag IDs.
    Source,
    /// Song language tag. Values are tag IDs.
    Language,
    /// Performance modifier tag. Values are tag IDs.
    Modifier,
    /// Performance instrument tag. Values are tag IDs.
    Instrument,
    /// Miscellaneous tag on a performance or on any of its songs. Values are tag IDs.
    Misc,
    /// Performance date. The value is an inclusive `from..to` range of `YYYY-MM-DD` dates where
    /// either side may be empty.
    Date,
}

impl FilterField {
    /// Returns the name used in filter expressions.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Singer => "singer",
            Self::OriginalArtist => "original_artist",
            Self::Song => "song",
            Self::Genre => "genre",
            Self::Source => "source",
            Self::Language => "language",
            Self::Modifier => "modifier",
            Self::Instrument => "instrument",
            Self::Misc => "misc",
            Self::Date => "date",
        }
    }
}

impl std::str::FromStr for FilterField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "singer" => Ok(Self::Singer),
            "original_artist" => Ok(Self::OriginalArtist),
            "song" => Ok(Self::Song),
            "genre" => Ok(Self::Genre),
            "source" => Ok(Self::Source),
            "language" => Ok(Self::Language),
            "modifier" => Ok(Self::Modifier),
            "instrument" => Ok(Self::Instrument),
            "misc" => Ok(Self::Misc),
            "date" => Ok(Self::Date),
            _ => Err(()),
        }
    }
}

/// The kind of result a filter catalog is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterSubject {
    Performances,
    Songs,
}

/// A selectable filter value.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatalogValue {
    /// Value to send back in a filter expression. An ID, or a year for the date field.
    pub id: String,
    /// Display name.
    pub name: String,
}

/// The values available for one filter field.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatalogGroup {
    pub field: FilterField,
    /// Values that at least one result uses, most used first.
    pub values: Vec<CatalogValue>,
}

/// Response body for `GET /api/filters/catalog`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatalogResponse {
    /// One group per field that has values.
    pub groups: Vec<CatalogGroup>,
}
