use axum::http;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use instant_messenger_common::MessageRet;
use log::error;
use std::error::Error;
use tokio::task::JoinError;

use crate::db_services::DBError;

type JWTTokenError = instant_messenger_common::tokens::TokenError;
type PasswordHashError = argon2::password_hash::Error;

pub enum AppError {
    NoAuthorization,
    InvalidHeader(http::header::ToStrError),
    Unauthorized,
    BodyTooLarge(axum::Error),
    InvalidUtf8(std::string::FromUtf8Error),
    JsonDecodeError(serde_json::error::Error),
    InternalDBError,
    DBEntryNotFound,
    DBNoResults,
    TokenExpired,
    HashParsingFailed(argon2::password_hash::Error),
    HashGenerationFailed(argon2::password_hash::Error),
    PasswordIncorrect,
    UserExists,
    BadRequest,
    Other(anyhow::Error),
}

impl From<DBError> for AppError {
    fn from(err: DBError) -> Self {
        match err {
            DBError::NoResults => AppError::DBEntryNotFound,
            DBError::NotFound => AppError::DBNoResults,
            DBError::ConnectError | DBError::Other => AppError::InternalDBError,
        }
    }
}

impl From<http::header::ToStrError> for AppError {
    fn from(err: http::header::ToStrError) -> Self {
        AppError::InvalidHeader(err)
    }
}

impl From<axum::Error> for AppError {
    fn from(err: axum::Error) -> Self {
        if err
            .source()
            .unwrap()
            .is::<http_body_util::LengthLimitError>()
        {
            AppError::BodyTooLarge(err)
        } else {
            AppError::Other(err.into())
        }
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        AppError::InvalidUtf8(err)
    }
}

impl From<serde_json::error::Error> for AppError {
    fn from(err: serde_json::error::Error) -> Self {
        AppError::JsonDecodeError(err)
    }
}

impl From<JWTTokenError> for AppError {
    fn from(err: JWTTokenError) -> Self {
        match err {
            JWTTokenError::Expired => AppError::TokenExpired,
            JWTTokenError::Other(_) => AppError::Unauthorized,
        }
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        match err {
            PasswordHashError::Password => AppError::PasswordIncorrect,
            e => AppError::HashParsingFailed(e),
        }
    }
}

impl From<JoinError> for AppError {
    fn from(err: JoinError) -> Self {
        AppError::Other(err.into())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Other(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NoAuthorization => {
                error!("Authentication header not set");
                (
                    StatusCode::UNAUTHORIZED,
                    MessageRet::new("No Authentication header"),
                )
            }
            AppError::Unauthorized => {
                error!("Request unauthorized");
                (StatusCode::UNAUTHORIZED, MessageRet::new("Token invalid"))
            }
            AppError::InvalidUtf8(e) => {
                error!("Failed to decode request body: {e}");
                (
                    StatusCode::BAD_REQUEST,
                    MessageRet::new("Body is not valid UTF-8"),
                )
            }
            AppError::BodyTooLarge(e) => {
                error!("Request body size exceeded: {e}");
                (
                    StatusCode::PAYLOAD_TOO_LARGE,
                    MessageRet::new("Request body size exceeded"),
                )
            }
            AppError::InvalidHeader(e) => {
                error!("Authentication token validation failure: {}", e);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    MessageRet::new("Invalid header"),
                )
            }
            AppError::JsonDecodeError(e) => {
                error!("Failed to decode JSON body: {e}");
                (StatusCode::BAD_REQUEST, MessageRet::new("Invalid JSON"))
            }
            AppError::InternalDBError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                MessageRet::new("Internal database error"),
            ),
            AppError::DBEntryNotFound => (
                StatusCode::NOT_FOUND,
                MessageRet::new("The query returned no data"),
            ),
            AppError::DBNoResults => (
                StatusCode::NOT_FOUND,
                MessageRet::new("An entry like that doesn't exist"),
            ),
            AppError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                MessageRet::with_action("Token expired", "refresh_token"),
            ),
            AppError::HashParsingFailed(e) => {
                error!("Hash parsing failed: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    MessageRet::new("Hash parsing error"),
                )
            }
            AppError::HashGenerationFailed(e) => {
                error!("Hash generation failed: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    MessageRet::new("Hash generation failure"),
                )
            }
            AppError::PasswordIncorrect => (
                StatusCode::UNAUTHORIZED,
                MessageRet::with_action("Incorrect password", "pass_invalid"),
            ),
            AppError::UserExists => (
                StatusCode::CONFLICT,
                MessageRet::with_action("User already exists", "user_exists"),
            ),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, MessageRet::new("Invalid request")),
            AppError::Other(err) => {
                error!("An unknown error has occurred: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    MessageRet::new("Unknown error"),
                )
            }
        }
        .into_response()
    }
}
