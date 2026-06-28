use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{DateTime, TimeDelta, Utc};

use crate::{error::AppError, state::AppState};

async fn create_refresh_token(
    state: &AppState,
    user_id: i32,
) -> Result<(String, DateTime<Utc>), AppError> {
    let (claims, token) = state.create_refresh_token(user_id);

    state.db_manager.clean_stale_tokens(user_id).await?;
    state.db_manager.insert_refresh_token(claims.jwi, user_id, claims.exp).await?;

    Ok((token, claims.exp))
}

pub async fn authenticate_user(
    state: &AppState,
    username: String,
    password: String,
) -> Result<(String, DateTime<Utc>), AppError> {
    let (user_id, hash_str) = state.db_manager.get_hash_and_id(&username).await?;

    tokio::task::spawn_blocking(move || {
        let argon = Argon2::default();
        let password_hash =
            PasswordHash::new(&hash_str).map_err(|e| AppError::HashParsingFailed(e))?;
        argon
            .verify_password(password.as_bytes(), &password_hash)
            .map_err(|_| AppError::Unauthorized)
    })
    .await??;

    let token = create_refresh_token(state, user_id).await?;

    Ok(token)
}

pub async fn register_user(
    state: &AppState,
    login: String,
    password: String,
) -> Result<(String, DateTime<Utc>), AppError> {
    if state.db_manager.does_user_exist(&login).await? {
        return Err(AppError::UserExists);
    }

    let hash = tokio::task::spawn_blocking(move || {
        let argon = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::HashGenerationFailed(e))?;
        Ok::<String, AppError>(password_hash.to_string())
    })
    .await??;

    let user_id = state.db_manager.insert_user(&login, &hash).await?;

    let token = create_refresh_token(state, user_id).await?;

    Ok(token)
}

pub enum GetTokenReturn {
    /// Fields: `new_access`, `exp_access`
    OnlyAccess(String, DateTime<Utc>),
    /// Fields: `new_access`, `exp_access`, `new_refresh`, `exp_refresh`
    WithRefresh(String, DateTime<Utc>, String, DateTime<Utc>),
}

pub async fn get_access_token(
    state: &AppState,
    refresh_token: &str,
) -> Result<GetTokenReturn, AppError> {
    let refresh_claims = state.validate_refresh_token(refresh_token)?;
    let user_id = refresh_claims.sub;
    let (access_claims, access_token) = state.create_access_token(user_id);

    let result: GetTokenReturn;

    if refresh_claims.exp < Utc::now() + TimeDelta::days(5) {
        let (new_token, expiry) = create_refresh_token(state, user_id).await?;

        result =
            GetTokenReturn::WithRefresh(access_token, access_claims.exp, new_token, expiry);
    } else {
        result = GetTokenReturn::OnlyAccess(access_token, access_claims.exp);
    }

    Ok(result)
}
