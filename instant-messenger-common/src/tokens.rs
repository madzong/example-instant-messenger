use chrono::{DateTime, TimeDelta, Utc, serde::ts_seconds};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

type JWTErrorKind = jsonwebtoken::errors::ErrorKind;

// To make things a bit simpler
#[derive(Debug, Clone)]
pub enum TokenError {
    Expired,
    Other(jsonwebtoken::errors::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

impl From<jsonwebtoken::errors::Error> for TokenError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            JWTErrorKind::ExpiredSignature => TokenError::Expired,
            _ => TokenError::Other(err),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaimsAccess {
    pub sub: i32,
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
    pub typ: TokenType,
}

impl ClaimsAccess {
    pub fn new(user_id: i32) -> Self {
        Self {
            sub: user_id,
            exp: Utc::now() + TimeDelta::hours(1),
            typ: TokenType::Access,
        }
    }
}

// This is a separate type, because in the future
// we might want different fields.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaimsRefresh {
    pub sub: i32,
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
    pub jwi: Uuid,
    pub typ: TokenType,
}

impl ClaimsRefresh {
    pub fn new(user_id: i32) -> Self {
        let jwi = Uuid::new_v4();

        Self {
            sub: user_id,
            exp: chrono::Utc::now() + TimeDelta::days(10),
            jwi,
            typ: TokenType::Refresh,
        }
    }
}

pub fn encode_token(claims: &impl Serialize, secret: &str) -> anyhow::Result<String> {
    Ok(jsonwebtoken::encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?)
}

pub fn decode_token<T: DeserializeOwned>(token: &str, secret: &str) -> Result<T, TokenError> {
    let token = jsonwebtoken::decode::<T>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token.claims)
}
