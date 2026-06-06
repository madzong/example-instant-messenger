use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;

use crate::{endpoints::json_from_body, error::AppError, services::user, state::State};

pub async fn send_message_handler(
    req: Request<Body>,
    state: Arc<State>,
) -> Result<Response, AppError> {
    let headers = req.headers();
    let access_token = headers
        .get("Authorization")
        .ok_or(AppError::NoAuthorization)?
        .to_str()?
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?
        .to_string();

    let body_json = json_from_body(req.into_body()).await?;

    user::send_message(&body_json, &access_token, &state).await?;

    Ok(StatusCode::OK.into_response())
}
