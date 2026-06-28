use std::env;

use log::error;
use tokio_postgres::NoTls;

#[derive(Debug)]
pub struct AppState {
    pub db_client: tokio_postgres::Client,
    pub http_client: reqwest::Client,
    pub secret: String,
    pub broker_host: String,
    pub comms_secret: String,
}

impl AppState {
    pub async fn new(
        db_host: &str,
        db_user: &str,
        db_password: &str,
        db_name: &str,
    ) -> anyhow::Result<Self> {
        let (db_client, db_connection) = tokio_postgres::connect(
            &format!("user={db_user} password={db_password} dbname={db_name} host={db_host}"),
            NoTls,
        )
        .await?;

        tokio::spawn(async move {
            if let Err(e) = db_connection.await {
                error!("Database connection error: {}", e);
            }
        });

        db_client.query("UPDATE users SET status=0", &[]).await?;

        let http_client = reqwest::Client::new();
        let secret = env::var("ENC_SECRET").expect("ENC_SECRET environment variable not set");
        let broker_host =
            env::var("BROKER_HOST").expect("BROKER_HOST environment variable not set");
        let comms_secret =
            env::var("COMMS_SECRET").expect("COMMS_SECRET environment variable not set");

        Ok(Self {
            db_client,
            http_client,
            secret,
            broker_host,
            comms_secret,
        })
    }
}
