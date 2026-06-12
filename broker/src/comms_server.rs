use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{patch, post},
};
use instant_messenger_common::{MessageReqBody, NewFriendshipReqBody, UpdateStatusReqBody};
use tokio::net::TcpListener;

use crate::{error::AppError, state::AppState};

pub async fn run_comms_server(sock: TcpListener, state: Arc<AppState>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/update_status", patch(update_status_handler))
        .route("/new_message", post(new_message_handler))
        .route("/new_friendship", patch(new_friendship_handler))
        .with_state(state);

    axum::serve(sock, app).await?;

    Ok(())
}

pub async fn update_status_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateStatusReqBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = body.user_id;
    let new_status = body.new_status;

    state.update_user_status(user_id, new_status).await?;

    Ok(StatusCode::OK)
}

pub async fn new_message_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<MessageReqBody>,
) -> impl IntoResponse {
    let content = body.content;
    let timestamp = body.timestamp;
    let sender_id = body.sender;
    let receiver_id = body.receiver;

    state
        .send_message(receiver_id, sender_id, content, timestamp)
        .await;

    StatusCode::OK
}

pub async fn new_friendship_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<NewFriendshipReqBody>,
) -> impl IntoResponse {
    let user_id = body.user_id;
    let friend_id = body.friend_id;

    state.new_friendship(user_id, friend_id).await;

    StatusCode::OK
}
