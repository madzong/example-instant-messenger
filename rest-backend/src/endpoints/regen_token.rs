use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use instant_messenger_common::{RegenTokenReqBody, RegenTokenRetBody};

use crate::{
    endpoints::json_from_body,
    error::AppError,
    services::{
        self,
        auth::GetTokenReturn::{OnlyAccess, WithRefresh},
    },
    state::State,
};

pub async fn regen_token_handler(
    req: Request<Body>,
    state: Arc<State>,
) -> Result<Response, AppError> {
    let body_json: RegenTokenReqBody = json_from_body(req.into_body()).await?;
    let secret = &state.secret;

    let data = match services::auth::get_access_token(&body_json, &state.db_client, secret).await? {
        OnlyAccess(access_token, access_token_exp) => (
            StatusCode::OK,
            serde_json::to_string(&RegenTokenRetBody {
                refresh_token: None,
                refresh_token_exp: None,
                access_token,
                access_token_exp,
            })?,
        ),
        WithRefresh(access_token, access_token_exp, refresh_token, refresh_token_exp) => (
            StatusCode::OK,
            serde_json::to_string(&RegenTokenRetBody {
                refresh_token: Some(refresh_token),
                refresh_token_exp: Some(refresh_token_exp),
                access_token,
                access_token_exp,
            })?,
        ),
    };

    Ok(data.into_response())
}
