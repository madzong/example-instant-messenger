use axum::{http::StatusCode, response::IntoResponse};
use instant_messenger_common::MessageRet;
use log::error;
use reqwest::header::ToStrError;

#[derive(Debug)]
pub enum AppError {
    NonexistentUser,
    NoAuthorization,
    Unauthorized,
    Unprocessable,
    InvalidJSON,
    Other,
}

impl From<ToStrError> for AppError {
    fn from(err: ToStrError) -> Self {
        error!("Error while parsing: {err}");
        AppError::Unprocessable
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        error!("Request error: {err}");
        AppError::Other
    }
}

impl From<serde_json::error::Error> for AppError {
    fn from(err: serde_json::error::Error) -> Self {
        error!("serde_json failed: {err}");
        AppError::InvalidJSON
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NonexistentUser => (
                StatusCode::NOT_FOUND,
                MessageRet::new("User is not connected"),
            ),
            AppError::NoAuthorization => (
                StatusCode::UNAUTHORIZED,
                MessageRet::new("Authorization header missing"),
            ),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, MessageRet::new("Token invalid")),
            AppError::Unprocessable => (
                StatusCode::UNPROCESSABLE_ENTITY,
                MessageRet::new("Request is unprocessable"),
            ),
            AppError::Other => (
                StatusCode::INTERNAL_SERVER_ERROR,
                MessageRet::new("An unknown error occurred"),
            ),
            AppError::InvalidJSON => (
                StatusCode::BAD_REQUEST,
                MessageRet::new("Could not decode JSON"),
            )
        }
        .into_response()
    }
}
