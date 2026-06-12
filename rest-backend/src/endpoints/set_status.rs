use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
};
use instant_messenger_common::SetStatusReqBody;
use reqwest::StatusCode;
use log::debug;

use crate::{endpoints::json_from_body, error::AppError, services::user, state::State};

pub async fn set_status_handler(
    req: Request<Body>,
    state: Arc<State>,
) -> Result<Response, AppError> {
    debug!("/set_status requested");
    let headers = req.headers();
    let access_token = headers
        .get("Authorization")
        .ok_or(AppError::NoAuthorization)?
        .to_str()?
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?
        .to_string();

    let body_json: SetStatusReqBody = json_from_body(req.into_body()).await?;
    let new_status = body_json.status;

    user::set_status(new_status, &access_token, &state).await?;

    Ok(StatusCode::OK.into_response())
}
