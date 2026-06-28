use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use instant_messenger_common::DisconnectReqBody;
use reqwest::StatusCode;

use crate::{error::AppError, services::user, state::AppState};

pub async fn disconnect_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<DisconnectReqBody>,
) -> Result<Response, AppError> {
    let internal_token = headers
        .get("X-Internal-Communication")
        .ok_or(AppError::Unauthorized)?
        .to_str()?;

    user::disconnect(body, internal_token, &state).await?;

    Ok(StatusCode::OK.into_response())
}
