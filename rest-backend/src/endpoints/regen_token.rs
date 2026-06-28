use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use instant_messenger_common::{RegenTokenReqBody, RegenTokenRetBody};

use crate::{
    error::AppError,
    services::{
        self,
        auth::GetTokenReturn::{OnlyAccess, WithRefresh},
    },
    state::AppState,
};

pub async fn regen_token_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegenTokenReqBody>,
) -> Result<Response, AppError> {
    let refresh_token = body.refresh_token;

    let data = match services::auth::get_access_token(&state, &refresh_token).await? {
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
