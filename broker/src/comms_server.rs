use axum::{Router, http::StatusCode, response::{IntoResponse, Response}, routing::patch};
use tokio::net::TcpListener;

pub enum AppError {}

pub async fn run_comms_server(sock: TcpListener) -> anyhow::Result<()> {
    let app = Router::new()
        .route(
            "/update_status",
            patch(update_status_handler),
        )
        .route(
            "/new_message",
            patch(new_message_handler),
        )
        .route(
            "/update_status",
            patch(new_friendship_handler),
        );

    axum::serve(sock, app).await?;

    Ok(())
}

pub async fn update_status_handler() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn new_message_handler() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn new_friendship_handler() -> impl IntoResponse {
    StatusCode::OK
}
