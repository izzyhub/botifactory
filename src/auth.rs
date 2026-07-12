// pattern: Imperative Shell

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::configuration::Settings;
use crate::routes::error::APIError;
use crate::validation::token_matches;

/// Extractor that gates write endpoints behind the configured API token
/// (`Authorization: Bearer <token>`).
///
/// When no token is configured the request is allowed through: that is a
/// development-only convenience, because `get_configuration` refuses to start
/// a production instance without a token.
pub struct RequireApiToken;

impl FromRequestParts<(SqlitePool, Arc<Settings>)> for RequireApiToken {
    type Rejection = APIError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &(SqlitePool, Arc<Settings>),
    ) -> Result<Self, Self::Rejection> {
        let (_db, settings) = state;
        let Some(expected) = &settings.application.api_token else {
            return Ok(RequireApiToken);
        };

        let provided = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(APIError::Unauthorized)?;

        if token_matches(expected, provided) {
            Ok(RequireApiToken)
        } else {
            Err(APIError::Unauthorized)
        }
    }
}
