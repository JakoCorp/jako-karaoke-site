//! Filter catalog endpoint and the `FiltersApi` OpenAPI spec struct.

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};

use api_types::filters::{CatalogGroup, CatalogResponse, CatalogValue, FilterField, FilterSubject};
use db::{
    MySqlPool,
    filter::{self, Subject},
    queries,
};

use crate::{
    error::ApiError,
    filters::{applies_to, target_for},
    state::AppState,
};

/// Fields offered in the catalog. Fields backed by large lists, such as artists and songs, are
/// left out.
const CATALOG_FIELDS: [FilterField; 7] = [
    FilterField::Genre,
    FilterField::Source,
    FilterField::Language,
    FilterField::Modifier,
    FilterField::Instrument,
    FilterField::Misc,
    FilterField::Date,
];

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(get_filter_catalog),
    components(schemas(
        FilterField,
        FilterSubject,
        CatalogValue,
        CatalogGroup,
        CatalogResponse
    ))
)]
pub(crate) struct FiltersApi;

/// Query parameters for `GET /api/filters/catalog`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct CatalogParams {
    /// Kind of result the values are for. Defaults to `performances`.
    pub subject: Option<FilterSubject>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/catalog", get(get_filter_catalog))
}

fn db_subject(subject: FilterSubject) -> Subject {
    match subject {
        FilterSubject::Performances => Subject::Performance,
        FilterSubject::Songs => Subject::Song,
    }
}

async fn field_values(
    pool: &MySqlPool,
    subject: Subject,
    field: FilterField,
) -> Result<Vec<CatalogValue>, ApiError> {
    if field == FilterField::Date {
        let years = queries::performances::years(pool).await?;
        return Ok(years
            .into_iter()
            .map(|year| CatalogValue {
                id: year.to_string(),
                name: year.to_string(),
            })
            .collect());
    }

    let Some(target) = target_for(field) else {
        return Ok(Vec::new());
    };
    let values = filter::catalog_values(pool, subject, target).await?;
    Ok(values
        .into_iter()
        .map(|value| CatalogValue {
            id: value.id.to_string(),
            name: value.name,
        })
        .collect())
}

#[utoipa::path(
    get,
    path = "/api/filters/catalog",
    params(CatalogParams),
    responses(
        (status = 200, description = "Values that can be used as filters", body = CatalogResponse),
    ),
    tag = "filters"
)]
pub(crate) async fn get_filter_catalog(
    State(state): State<AppState>,
    Query(params): Query<CatalogParams>,
) -> Result<Json<CatalogResponse>, ApiError> {
    let subject = db_subject(params.subject.unwrap_or(FilterSubject::Performances));

    let mut groups = Vec::new();
    for field in CATALOG_FIELDS {
        if !applies_to(field, subject) {
            continue;
        }
        let values = field_values(&state.pool, subject, field).await?;
        if !values.is_empty() {
            groups.push(CatalogGroup { field, values });
        }
    }
    Ok(Json(CatalogResponse { groups }))
}
