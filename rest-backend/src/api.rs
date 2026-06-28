use std::env;
use std::sync::Arc;

use crate::db_services::DBCredentials;
use crate::endpoints;
use axum::http::HeaderValue;
use axum::routing::{get, patch};
use axum::{Router, routing::post};
use reqwest::Method;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::state::AppState;

pub async fn run_api(sock: TcpListener) -> anyhow::Result<()> {
    let pg_dbname = env::var("PG_DBNAME").expect("PG_DBNAME environment variable not set");
    let pg_host = env::var("PG_HOST").expect("PG_HOST environment variable not set");
    let pg_user = env::var("PG_USER").expect("PG_USER environment variable not set");
    let pg_password = env::var("PG_PASSWORD").expect("PG_PASSWORD environment variable not set");

    let creds = DBCredentials {
        db_name: pg_dbname,
        db_host: pg_host,
        db_user: pg_user,
        db_password: pg_password,
    };

    let state = Arc::new(AppState::new(creds).await?);

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| {
            // For an example we don't need to care
            !origin.is_empty()
        }))
        .allow_credentials(true)
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ]);

    let app = Router::new()
        .route("/connect", post(endpoints::connect_handler))
        .route("/disconnect", post(endpoints::disconnect_handler))
        .route("/login", post(endpoints::login_handler))
        .route("/register", post(endpoints::register_handler))
        .route("/regen_token", post(endpoints::regen_token_handler))
        .route("/send_message", post(endpoints::send_message_handler))
        .route("/set_status", patch(endpoints::set_status_handler))
        .route("/get_user_info", get(endpoints::get_user_info_handler))
        .route("/get_messages", get(endpoints::get_messages_handler))
        .route("/get_contacts", get(endpoints::get_contacts_handler))
        .with_state(state)
        .layer(cors);

    axum::serve(sock, app).await?;

    Ok(())
}
