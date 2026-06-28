use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use instant_messenger_common::GetMessagesQuery;
use reqwest::StatusCode;

use crate::{error::AppError, services::user, state::AppState};

pub async fn get_messages_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetMessagesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let access_token = headers
        .get("Authorization")
        .ok_or(AppError::NoAuthorization)?
        .to_str()?
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?
        .to_string();

    let limit = params.limit.unwrap_or(100);
    let page = params.page.unwrap_or(0);

    let resp = user::get_messages(&access_token, params.user_id, limit, page, &state).await?;

    Ok((StatusCode::OK, serde_json::to_string(&resp)?))
}
