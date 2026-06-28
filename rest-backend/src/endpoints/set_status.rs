use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use instant_messenger_common::SetStatusReqBody;
use reqwest::StatusCode;

use crate::{error::AppError, services::user, state::AppState};

pub async fn set_status_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetStatusReqBody>,
) -> Result<Response, AppError> {
    let access_token = headers
        .get("Authorization")
        .ok_or(AppError::NoAuthorization)?
        .to_str()?
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?
        .to_string();

    let new_status = body.status;

    user::set_status(new_status, &access_token, &state).await?;

    Ok(StatusCode::OK.into_response())
}
