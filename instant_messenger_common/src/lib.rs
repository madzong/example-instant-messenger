use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use chrono::{DateTime, Utc, serde::{ts_seconds, ts_seconds_option}};

pub mod tokens;

#[derive(Deserialize_repr, Serialize_repr, Clone, Copy, Debug)]
#[repr(u8)]
pub enum UserStatus {
    Offline = 0,
    Online = 1,
    DnD = 2,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub struct UserInfo {
    pub id: i32,
    pub status: UserStatus,
}

impl From<tokio_postgres::Row> for UserInfo {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            status: serde_json::from_str(&row.get::<_, String>("status")).unwrap(),
        }
    }
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct UpdateStatusReqBody {
    pub user_id: i32,
    pub new_status: UserStatus,
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConnectRetBody {
    pub user_info: UserInfo,
    pub friendships: Vec<i32>,
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
