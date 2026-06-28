use std::sync::Arc;

use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;

use crate::{error::AppError, services::user, state::AppState};

pub async fn connect_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let access_token = headers
        .get("Authorization")
        .ok_or(AppError::NoAuthorization)?
        .to_str()?
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let internal_token = headers
        .get("X-Internal-Communication")
        .ok_or(AppError::Unauthorized)?
        .to_str()?;

    let ret_body = user::connect(access_token, internal_token, &state).await?;

    Ok((StatusCode::OK, serde_json::to_string(&ret_body).unwrap()).into_response())
}
