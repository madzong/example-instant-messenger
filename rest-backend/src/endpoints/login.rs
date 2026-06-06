use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use instant_messenger_common::{LoginReqBody, LoginRetBody};

use crate::{endpoints::json_from_body, error::AppError, services, state::State};

pub async fn login_handler(req: Request<Body>, state: Arc<State>) -> Result<Response, AppError> {
    let body_json: LoginReqBody = json_from_body(req.into_body()).await?;
    let db = &state.db_client;
    let secret = &state.secret;

    let (refresh_token, refresh_token_exp) =
        services::auth::authenticate_user(&body_json, db, secret).await?;

    let resp = LoginRetBody {
        refresh_token,
        refresh_token_exp,
    };

    Ok((StatusCode::OK, serde_json::to_string(&resp)?).into_response())
}
