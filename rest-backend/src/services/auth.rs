use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{DateTime, TimeDelta, Utc};
use instant_messenger_common::{
    LoginReqBody, RegenTokenReqBody, RegisterReqBody,
    tokens::{self, ClaimsAccess, ClaimsRefresh, TokenType},
};

use crate::error::AppError;

async fn create_refresh_token(
    db: &tokio_postgres::Client,
    secret: &str,
    user_id: i32,
) -> Result<(String, DateTime<Utc>), AppError> {
    let claims = ClaimsRefresh::new(user_id);
    let token = tokens::encode_token(&claims, secret).unwrap();

    db.query(
        "DELETE FROM refresh_tokens WHERE user_id = $1 AND expires_at <= CURRENT_TIMESTAMP",
        &[&user_id],
    )
    .await?;

    db.query(
        "INSERT INTO refresh_tokens (id, user_id, expires_at)
              VALUES ($1, $2, $3)",
        &[&claims.jwi, &user_id, &claims.exp],
    )
    .await?;

    Ok((token, claims.exp))
}

pub async fn authenticate_user(
    login_body: &LoginReqBody,
    db: &tokio_postgres::Client,
    secret: &str,
) -> Result<(String, DateTime<Utc>), AppError> {
    let row = db
        .query_one(
            "SELECT passwords.hash, users.id
               FROM users JOIN passwords ON users.password_id = passwords.id
              WHERE users.username = $1",
            &[&login_body.login],
        )
        .await
        .map_err(|_| AppError::PasswordIncorrect)?;

    let hash_str: String = row.get("hash");
    let user_id: i32 = row.get("id");
    let password = login_body.password.clone(); // Cloned because of move

    tokio::task::spawn_blocking(move || {
        let argon = Argon2::default();
        let password_hash =
            PasswordHash::new(&hash_str).map_err(|e| AppError::HashParsingFailed(e))?;
        argon
            .verify_password(password.as_bytes(), &password_hash)
            .map_err(|_| AppError::Unauthorized)
    })
    .await??;

    let token = create_refresh_token(db, secret, user_id).await?;

    Ok(token)
}

pub async fn register_user(
    register_body: &RegisterReqBody,
    db: &tokio_postgres::Client,
    secret: &str,
) -> Result<(String, DateTime<Utc>), AppError> {
    let row = db
        .query_one(
            "SELECT username FROM users WHERE username = $1",
            &[&register_body.login],
        )
        .await;

    if let Ok(_) = row {
        return Err(AppError::UserExists);
    }

    let password = register_body.password.clone();

    let hash = tokio::task::spawn_blocking(move || {
        let argon = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::HashGenerationFailed(e))?;
        Ok::<String, AppError>(password_hash.to_string())
    })
    .await??;

    let password_id: i32 = db
        .query_one(
            "INSERT INTO passwords (hash) VALUES ($1) RETURNING id",
            &[&hash],
        )
        .await?
        .get(0);

    let user_id: i32 = db
        .query_one(
            "INSERT INTO users (username, password_id)
                  VALUES ($1, $2) RETURNING id",
            &[&register_body.login, &password_id],
        )
        .await?
        .get(0);

    let token = create_refresh_token(db, secret, user_id).await?;

    Ok(token)
}

pub enum GetTokenReturn {
    /// Fields: `new_access`, `exp_access`
    OnlyAccess(String, DateTime<Utc>),
    /// Fields: `new_access`, `exp_access`, `new_refresh`, `exp_refresh`
    WithRefresh(String, DateTime<Utc>, String, DateTime<Utc>),
}

pub async fn get_access_token(
    regen_token_body: &RegenTokenReqBody,
    db: &tokio_postgres::Client,
    secret: &str,
) -> Result<GetTokenReturn, AppError> {
    let refresh_claims: ClaimsRefresh =
        tokens::decode_token(&regen_token_body.refresh_token, secret)?;
    let user_id = refresh_claims.sub;

    if let TokenType::Access = refresh_claims.typ {
        return Err(AppError::Unauthorized);
    }

    let access_claims = ClaimsAccess::new(user_id);
    let access_token = tokens::encode_token(&access_claims, secret).unwrap();

    let result: GetTokenReturn;

    if refresh_claims.exp < Utc::now() + TimeDelta::days(5) {
        db.query(
            "DELETE FROM refresh_tokens WHERE uuid = $1",
            &[&refresh_claims.jwi],
        )
        .await?;

        let new_token = create_refresh_token(db, secret, user_id).await?;

        result =
            GetTokenReturn::WithRefresh(access_token, access_claims.exp, new_token.0, new_token.1);
    } else {
        result = GetTokenReturn::OnlyAccess(access_token, access_claims.exp);
    }

    Ok(result)
}
