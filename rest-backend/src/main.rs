use std::env;

use log::info;

use crate::api::run_api;

mod api;
pub mod endpoints;
mod error;
pub mod services;
mod state;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = env_logger::try_init();
    let addr = env::var("ADDRESS").unwrap_or("127.0.0.1:3000".to_string());

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    run_api(listener).await?;

    Ok(())
}
