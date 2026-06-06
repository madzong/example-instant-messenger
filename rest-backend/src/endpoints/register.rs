use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use instant_messenger_common::{RegisterReqBody, RegisterRetBody};

use crate::{endpoints::json_from_body, error::AppError, services, state::State};

pub async fn register_handler(req: Request<Body>, state: Arc<State>) -> Result<Response, AppError> {
    let body_json: RegisterReqBody = json_from_body(req.into_body()).await?;
    let secret = &state.secret;

    let (refresh_token, refresh_token_exp) =
        services::auth::register_user(&body_json, &state.db_client, secret).await?;

    let resp = RegisterRetBody {
        refresh_token,
        refresh_token_exp,
    };

    Ok((StatusCode::OK, serde_json::to_string(&resp)?).into_response())
}
