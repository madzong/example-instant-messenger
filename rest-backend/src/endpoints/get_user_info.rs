use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    db_services::UserIdentifier, error::AppError, services::user, state::AppState
};

#[derive(Clone, Debug, Deserialize)]
pub struct GetUserInfoParams {
    user_id: Option<i32>,
    username: Option<String>,
}

pub async fn get_user_info_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetUserInfoParams>,
) -> Result<impl IntoResponse, AppError> {
    let access_token = headers
        .get("Authorization")
        .ok_or(AppError::NoAuthorization)?
        .to_str()?
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?
        .to_string();

    let user_identifier = if params.user_id.is_some() {
        Some(UserIdentifier::ID(params.user_id.unwrap()))
    } else if params.username.is_some() {
        Some(UserIdentifier::Username(params.username.unwrap()))
    } else {
        None
    };

    let resp = user::get_user_info(&access_token, user_identifier, &state).await?;

    Ok((StatusCode::OK, serde_json::to_string(&resp)?))
}
