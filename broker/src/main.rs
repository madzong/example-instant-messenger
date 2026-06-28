use std::{env, sync::Arc};

use comms_server::run_comms_server;
use log::info;
use tokio::{
    net::TcpListener,
    signal::unix::{SignalKind, signal},
};
use ws_server::run_ws_server;

use crate::state::AppState;

mod api_gateway;
mod comms_server;
mod error;
mod state;
mod types;
mod ws_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = env_logger::try_init();
    let ws_addr = env::var("WS_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let comms_addr = env::var("API_COMMS_ADDRESS").unwrap_or_else(|_| "127.0.0.1:4000".to_string());

    let ws_sock = TcpListener::bind(&ws_addr).await?;
    info!("Websocket listening on: {}", ws_addr);
    let comms_sock = TcpListener::bind(&comms_addr).await?;
    info!("API communications server listening on: {}", comms_addr);

    let state = Arc::new(AppState::new());

    let ws_handle = tokio::spawn(run_ws_server(ws_sock, Arc::clone(&state)));
    let comms_handle = tokio::spawn(run_comms_server(comms_sock, Arc::clone(&state)));
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        res = ws_handle => res?,
        res = comms_handle => res?,
        _ = sigterm.recv() => {
            info!("Received SIGTERM");
            Ok(())
        }
        _ = sigint.recv() => {
            info!("Received SIGINT");
            Ok(())
        }
    }?;

    state.cleanup().await;

    Ok(())
}
