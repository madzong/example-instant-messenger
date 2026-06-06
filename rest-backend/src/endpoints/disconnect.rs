use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;

use crate::{endpoints::json_from_body, error::AppError, services::user, state::State};

pub async fn disconnect_handler(
    req: Request<Body>,
    state: Arc<State>,
) -> Result<Response, AppError> {
    let headers = req.headers().clone();

    let internal_token = headers
        .get("X-Internal-Communication")
        .ok_or(AppError::Unauthorized)?
        .to_str()?;

    let req_body = req.into_body();
    let json_body = json_from_body(req_body).await?;

    user::disconnect(json_body, internal_token, &state).await?;

    Ok(StatusCode::OK.into_response())
}
