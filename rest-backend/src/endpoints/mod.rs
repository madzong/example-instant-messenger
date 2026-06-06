use axum::body::Body;
use serde::de::DeserializeOwned;

use crate::error::AppError;

pub use connect::*;
pub use disconnect::*;
pub use login::*;
pub use regen_token::*;
pub use register::*;
pub use send_message::*;
pub use set_status::*;

pub mod connect;
pub mod disconnect;
pub mod login;
pub mod regen_token;
pub mod register;
pub mod send_message;
pub mod set_status;

async fn json_from_body<T: DeserializeOwned>(body: Body) -> Result<T, AppError> {
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
    let body_str = String::from_utf8(body_bytes.to_vec())?;
    let body_json: T = serde_json::from_str(&body_str)?;

    Ok(body_json)
}
