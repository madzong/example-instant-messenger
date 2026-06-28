use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use instant_messenger_common::SendMessageReqBody;
use reqwest::StatusCode;

use crate::{error::AppError, services::user, state::AppState};

pub async fn send_message_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<SendMessageReqBody>,
) -> Result<Response, AppError> {
    let access_token = headers
        .get("Authorization")
        .ok_or(AppError::NoAuthorization)?
        .to_str()?
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?
        .to_string();

    user::send_message(&body, &access_token, &state).await?;

    Ok(StatusCode::OK.into_response())
}
