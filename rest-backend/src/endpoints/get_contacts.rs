use std::sync::Arc;

use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use instant_messenger_common::GetContactsRetBody;
use reqwest::StatusCode;

use crate::{error::AppError, services::user, state::AppState};

pub async fn get_contacts_handler(headers: HeaderMap, State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let access_token = headers
        .get("Authorization")
        .ok_or(AppError::NoAuthorization)?
        .to_str()?
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?
        .to_string();

    let friends = user::get_contacts(&access_token, &state).await?;

    let resp = GetContactsRetBody {
        friends,
    };

    Ok((StatusCode::OK, serde_json::to_string(&resp)?))
}
