use std::{sync::Arc, time::Duration};

use log::error;
use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    http::HeaderMap,
    response::IntoResponse,
    routing::any,
};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use instant_messenger_common::ConnectRetBody;
use log::info;
use reqwest::{StatusCode, header};
use tokio::{net::TcpListener, sync::mpsc::UnboundedReceiver, time};

use crate::{error::AppError, state::AppState, types};

pub async fn run_ws_server(sock: TcpListener, state: Arc<AppState>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/ws", any(ws_handshake))
        .with_state(state);

    axum::serve(sock, app).await?;

    Ok(())
}

async fn ws_handshake(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let http_client = &state.http_client;
    let api_host = &state.api_host;
    let comms_secret = &state.comms_secret;

    let mut req_headers = header::HeaderMap::default();
    req_headers.insert(
        header::AUTHORIZATION,
        headers
            .get("Authorization")
            .ok_or(AppError::NoAuthorization)?
            .to_str()?
            .parse()
            .unwrap(),
    );
    req_headers.insert("X-Internal-Communication", comms_secret.parse().unwrap());

    let req = http_client
        .post(format!("{}/connect", api_host))
        .headers(req_headers)
        .send()
        .await?;

    let status = req.status();

    match status {
        StatusCode::UNAUTHORIZED => return Err(AppError::Unauthorized),
        StatusCode::UNPROCESSABLE_ENTITY => return Err(AppError::Unprocessable),
        _ => (),
    }

    let text_body = req.text().await?;
    let json_body: ConnectRetBody = serde_json::from_str(&text_body)?;
    let user_id = json_body.user_info.id;
    let user_status = json_body.user_info.status;
    let friendships = json_body.friendships;

    let rx = state.add_client(user_id, friendships, user_status).await;

    Ok(ws.on_upgrade(move |sock| {
        let state = Arc::clone(&state);
        handle_connection(sock, rx, user_id, state)
    }))
}

async fn handle_connection(
    socket: WebSocket,
    mut user_rx: UnboundedReceiver<types::Message>,
    user_id: i32,
    state: Arc<AppState>,
) {
    info!("Established connection with {user_id}");

    let (mut tx, mut rx) = socket.split();

    let mut heartbeat_interval = time::interval(Duration::from_secs(30));
    let mut heartbeat_waiting = false;

    // Tick so that we don't immediately send a Ping packet
    heartbeat_interval.tick().await;

    loop {
        tokio::select! {
            msg = rx.next() => {
                let msg = if let Some(Ok(msg)) = msg {
                    msg
                } else {
                    break;
                };

                match msg {
                    ws::Message::Pong(_) => heartbeat_waiting = false,
                    ws::Message::Close(_) => break,
                    _ => (),
                }
            }
            msg = user_rx.recv() => {
                let msg = if let Some(msg) = msg {
                    msg
                } else {
                    break;
                };

                if tx.send(ws::Message::Binary(msg.into())).await.is_err() {
                    break;
                }
            }
            _ = heartbeat_interval.tick() => {
                if heartbeat_waiting {
                    break;
                }

                heartbeat_waiting = true;

                if tx.send(ws::Message::Ping(Bytes::new())).await.is_err() {
                    break;
                }
            }
        }
    }

    info!("Closing connection with {user_id}");
    if let Err(err) = state.remove_user(user_id).await {
        error!("Error while removing user: {:?}", err);
    }
}
