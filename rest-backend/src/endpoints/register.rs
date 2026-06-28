use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use instant_messenger_common::{RegisterReqBody, RegisterRetBody};

use crate::{error::AppError, services, state::AppState};

pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterReqBody>,
) -> Result<Response, AppError> {
    let secret = &state.secret;

    let (refresh_token, refresh_token_exp) =
        services::auth::register_user(&body, &state.db_client, secret).await?;

    let resp = RegisterRetBody {
        refresh_token,
        refresh_token_exp,
    };

    Ok((StatusCode::OK, serde_json::to_string(&resp)?).into_response())
}
