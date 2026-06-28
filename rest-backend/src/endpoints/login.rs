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
    let db = &state.db_client;
    let secret = &state.secret;

    let (refresh_token, refresh_token_exp) =
        services::auth::authenticate_user(&body, db, secret).await?;

    let resp = LoginRetBody {
        refresh_token,
        refresh_token_exp,
    };

    Ok((StatusCode::OK, serde_json::to_string(&resp)?).into_response())
}
