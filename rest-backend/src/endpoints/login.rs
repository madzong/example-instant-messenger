use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use instant_messenger_common::{LoginReqBody, LoginRetBody};

use crate::{error::AppError, services, state::AppState};

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginReqBody>,
) -> Result<Response, AppError> {
    let username = body.login;
    let password = body.password;

    let (refresh_token, refresh_token_exp) =
        services::auth::authenticate_user(&state, username, password).await?;

    let resp = LoginRetBody {
        refresh_token,
        refresh_token_exp,
    };

    Ok((StatusCode::OK, serde_json::to_string(&resp)?).into_response())
}
