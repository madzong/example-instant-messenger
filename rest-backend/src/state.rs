use std::env;

use instant_messenger_common::tokens::{self, ClaimsAccess, ClaimsRefresh, TokenType};

use crate::{
    db_services::{DBCredentials, DBManager},
    error::AppError,
};

#[derive(Debug)]
pub struct AppState {
    pub db_manager: DBManager,
    pub http_client: reqwest::Client,
    pub secret: String,
    pub broker_host: String,
    comms_secret: String,
}

impl AppState {
    pub async fn new(db_creds: DBCredentials) -> anyhow::Result<Self> {
        let db_manager = DBManager::new(db_creds).await?;

        let http_client = reqwest::Client::new();
        let secret = env::var("ENC_SECRET").expect("ENC_SECRET environment variable not set");
        let broker_host =
            env::var("BROKER_HOST").expect("BROKER_HOST environment variable not set");
        let comms_secret =
            env::var("COMMS_SECRET").expect("COMMS_SECRET environment variable not set");

        Ok(Self {
            db_manager,
            http_client,
            secret,
            broker_host,
            comms_secret,
        })
    }

    pub fn validate_comms_secret(&self, secret: &str) -> Result<(), AppError> {
        if self.comms_secret != secret {
            log::error!("Internal token invalid");
            return Err(AppError::Unauthorized);
        }

        Ok(())
    }

    pub fn validate_access_token(&self, token: &str) -> Result<ClaimsAccess, AppError> {
        let claims: ClaimsAccess = tokens::decode_token(token, &self.secret)?;

        if claims.typ != TokenType::Access {
            return Err(AppError::Unauthorized);
        }

        Ok(claims)
    }

    pub fn validate_refresh_token(&self, token: &str) -> Result<ClaimsRefresh, AppError> {
        let claims: ClaimsRefresh = tokens::decode_token(token, &self.secret)?;

        if claims.typ != TokenType::Refresh {
            return Err(AppError::Unauthorized);
        }

        Ok(claims)
    }

    pub fn create_refresh_token(&self, user_id: i32) -> (ClaimsRefresh, String) {
        let claims = ClaimsRefresh::new(user_id);
        let token = tokens::encode_token(&claims, &self.secret).unwrap();

        (claims, token)
    }

    pub fn create_access_token(&self, user_id: i32) -> (ClaimsAccess, String) {
        let claims = ClaimsAccess::new(user_id);
        let token = tokens::encode_token(&claims, &self.secret).unwrap();

        (claims, token)
    }
}
