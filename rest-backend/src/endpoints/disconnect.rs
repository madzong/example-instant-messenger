use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;

use crate::{error::AppError, services::user, state::State};

pub async fn disconnect_handler(
    req: Request<Body>,
    state: Arc<State>,
) -> Result<Response, AppError> {
    let headers = req.headers();
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

    user::disconnect(access_token, internal_token, &state).await?;

    Ok(StatusCode::OK.into_response())
}
