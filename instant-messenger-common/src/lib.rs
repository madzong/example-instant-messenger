use axum::response::IntoResponse;
use chrono::{
    DateTime, Utc,
    serde::{ts_seconds, ts_seconds_option},
};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::FromRepr;

pub mod tokens;

#[derive(Deserialize_repr, Serialize_repr, Clone, Copy, Debug, PartialEq, FromRepr)]
#[repr(u8)]
pub enum UserStatus {
    Offline = 0,
    Online = 1,
    DnD = 2,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub status: UserStatus,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MessageRet {
    msg: String,
    action: Option<String>,
}

impl MessageRet {
    pub fn new(msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
            action: None,
        }
    }

    pub fn with_action(msg: impl ToString, action: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
            action: Some(action.to_string()),
        }
    }
}

impl IntoResponse for MessageRet {
    fn into_response(self) -> axum::response::Response {
        serde_json::to_string(&self).unwrap().into_response()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginReqBody {
    pub login: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginRetBody {
    pub refresh_token: String,
    #[serde(with = "ts_seconds")]
    pub refresh_token_exp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterReqBody {
    pub login: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterRetBody {
    pub refresh_token: String,
    #[serde(with = "ts_seconds")]
    pub refresh_token_exp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegenTokenReqBody {
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegenTokenRetBody {
    pub refresh_token: Option<String>,
    #[serde(with = "ts_seconds_option")]
    pub refresh_token_exp: Option<DateTime<Utc>>,
    pub access_token: String,
    #[serde(with = "ts_seconds")]
    pub access_token_exp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateStatusReqBody {
    pub user_id: i32,
    pub new_status: UserStatus,
    pub send_to: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SetStatusReqBody {
    pub status: UserStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendMessageReqBody {
    pub content: String,
    pub receiver: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageReqBody {
    pub sender: i32,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub receiver: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct ConnectRetBody {
    pub user_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetFriendshipsRetBody {
    pub friendships: Vec<UserInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewFriendshipReqBody {
    pub user_id: i32,
    pub friend_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DisconnectReqBody {
    pub user_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetUserInfoRetBody {
    pub status: UserStatus,
    pub username: String,
    pub id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMessagesQuery {
    pub user_id: i32,
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserMessage {
    pub sender: i32,
    pub content: String,
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetMessagesRetBody {
    pub messages: Vec<UserMessage>,
    pub row_count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetContactsRetBody {
    pub friends: Vec<UserInfo>,
}
