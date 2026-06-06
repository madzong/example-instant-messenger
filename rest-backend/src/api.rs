use std::env;
use std::sync::Arc;

use crate::endpoints;
use axum::routing::patch;
use axum::{Router, routing::post};
use tokio::net::TcpListener;

use crate::state::State;

pub async fn run_api(sock: TcpListener) -> anyhow::Result<()> {
    let pg_dbname = env::var("PG_DBNAME").expect("PG_DBNAME environment variable not set");
    let pg_host = env::var("PG_HOST").expect("PG_HOST environment variable not set");
    let pg_user = env::var("PG_USER").expect("PG_USER environment variable not set");
    let pg_password = env::var("PG_PASSWORD").expect("PG_PASSWORD environment variable not set");

    let state = Arc::new(State::new(&pg_host, &pg_user, &pg_password, &pg_dbname).await?);

    let app = Router::new()
        .route(
            "/connect",
            post({
                let state = Arc::clone(&state);
                move |req| endpoints::connect_handler(req, state)
            }),
        )
        .route(
            "/disconnect",
            post({
                let state = Arc::clone(&state);
                move |req| endpoints::disconnect_handler(req, state)
            }),
        )
        .route(
            "/login",
            post({
                let state = Arc::clone(&state);
                move |req| endpoints::login_handler(req, state)
            }),
        )
        .route(
            "/register",
            post({
                let state = Arc::clone(&state);
                move |req| endpoints::register_handler(req, state)
            }),
        )
        .route(
            "/regen_token",
            post({
                let state = Arc::clone(&state);
                move |req| endpoints::regen_token_handler(req, state)
            }),
        )
        .route(
            "/send_message",
            post({
                let state = Arc::clone(&state);
                move |req| endpoints::send_message_handler(req, state)
            }),
        )
        .route(
            "/set_status",
            patch({
                let state = Arc::clone(&state);
                move |req| endpoints::set_status_handler(req, state)
            }),
        );

    axum::serve(sock, app).await?;

    Ok(())
}
