use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{patch, post},
};
use instant_messenger_common::{MessageReqBody, UpdateStatusReqBody};
use tokio::net::TcpListener;

use crate::{error::AppError, state::AppState};

pub async fn run_comms_server(sock: TcpListener, state: Arc<AppState>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/update_status", patch(update_status_handler))
        .route("/new_message", post(new_message_handler))
        .method_not_allowed_fallback(method_not_allowed)
        .with_state(state);

    log::debug!("Comms server starting");

    axum::serve(sock, app).await?;

    log::debug!("Comms server stopping");

    Ok(())
}

pub async fn method_not_allowed() -> impl IntoResponse {
    (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed")
}

pub async fn update_status_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateStatusReqBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = body.user_id;
    let new_status = body.new_status;
    let friends = body.send_to;

    state.update_user_status(user_id, friends, new_status).await?;

    Ok(StatusCode::OK)
}

pub async fn new_message_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<MessageReqBody>,
) -> impl IntoResponse {
    log::debug!("/new_message: New request:\n{:#?}", body);
    let content = body.content;
    let timestamp = body.timestamp;
    let sender_id = body.sender;
    let receiver_id = body.receiver;

    state
        .send_message(receiver_id, sender_id, content, timestamp)
        .await;

    StatusCode::OK
}
