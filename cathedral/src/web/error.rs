use crate::{db::schema::FilterQueryError, web::schema::ErrorResult};

use axum::{http::StatusCode, response::ErrorResponse, Json};
use sqlx::Error as SqlxError;

pub fn pass_sqlx_error(err: SqlxError) -> ErrorResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResult {
            reason: format!("db error: {}", err),
        }),
    )
        .into()
}

pub fn pass_filter_query_error(err: FilterQueryError) -> ErrorResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResult {
            reason: format!("filter error: {}", err),
        }),
    )
        .into()
}

pub fn pass_not_found_error(subreason: &str) -> ErrorResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResult {
            reason: format!("not found: {subreason}"),
        }),
    )
        .into()
}

pub fn pass_token_error() -> ErrorResponse {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResult {
            reason: "unauthorized webhook token".to_string(),
        }),
    )
        .into()
}
