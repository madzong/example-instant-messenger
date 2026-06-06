use std::env;

use log::info;
use tokio::net::TcpListener;
use comms_server::run_comms_server;
use ws_server::run_ws_server;

mod comms_server;
mod ws_server;
mod context;
mod types;
mod state;
mod error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = env_logger::try_init();
    let ws_addr = env::var("WS_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let comms_addr = env::var("API_COMMS_ADDRESS").unwrap_or_else(|_| "127.0.0.1:4000".to_string());

    let ws_sock = TcpListener::bind(&ws_addr).await?;
    info!("Websocket listening on: {}", ws_addr);
    let comms_sock = TcpListener::bind(&comms_addr).await?;
    info!("API communications server listening on: {}", comms_addr);

    let ws_handle = tokio::spawn(run_ws_server(ws_sock));
    let comms_handle = tokio::spawn(run_comms_server(comms_sock));

    tokio::select! {
        res = ws_handle => res?,
        res = comms_handle => res?,
    }?;

    Ok(())
}
