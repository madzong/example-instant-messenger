use std::env;

use log::info;
use tokio::signal::unix::{SignalKind, signal};

use crate::api::run_api;

mod api;
pub mod db_services;
pub mod endpoints;
mod error;
pub mod services;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = env_logger::try_init();
    let addr = env::var("ADDRESS").unwrap_or("127.0.0.1:3000".to_string());

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        err = run_api(listener) => err?,
        _ = sigterm.recv() => info!("Received SIGTERM"),
        _ = sigint.recv() => info!("Received SIGINT"),
    };

    Ok(())
}
