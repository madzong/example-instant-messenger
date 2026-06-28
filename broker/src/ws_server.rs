use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::IntoResponse,
    routing::any,
};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::mpsc::UnboundedReceiver, time};

use crate::{error::AppError, state::AppState, types};

pub async fn run_ws_server(sock: TcpListener, state: Arc<AppState>) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", any(ws_handshake))
        .with_state(state);

    axum::serve(sock, app).await?;

    Ok(())
}

#[derive(Deserialize, Debug, Clone)]
struct WSHandshakeParams {
    token: String,
}

async fn ws_handshake(
    ws: WebSocketUpgrade,
    Query(params): Query<WSHandshakeParams>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let token = params.token;

    let (user_id, rx) = state.add_client(&token).await?;

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
                    ws::Message::Pong(_) => {
                        heartbeat_waiting = false;
                        log::debug!("Conn {user_id}: Received pong");
                    },
                    ws::Message::Close(_) => break,
                    _ => (),
                }
            }
            msg = user_rx.recv() => {
                log::debug!("WS connection {}: received message: {:#?}", user_id, msg);
                let msg = if let Some(msg) = msg {
                    msg
                } else {
                    break;
                };

                if msg == types::Message::Close {
                    break;
                }

                if tx.send(ws::Message::Binary(msg.into())).await.is_err() {
                    break;
                }
            }
            _ = heartbeat_interval.tick() => {
                log::debug!("Conn {user_id}: Pinging {heartbeat_waiting}");

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
