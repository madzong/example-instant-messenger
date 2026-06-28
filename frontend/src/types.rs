use chrono::{
    DateTime, Utc,
    serde::{ts_seconds, ts_seconds_option},
};
use implicit_clone::ImplicitClone;
use num_traits::FromBytes;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use yew::AttrValue;

use crate::components::Msg;

#[derive(Debug, Clone, Serialize)]
pub struct LoginReqBody {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResBody {
    pub refresh_token: String,
    #[serde(with = "ts_seconds")]
    pub refresh_token_exp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterReqBody {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterResBody {
    pub refresh_token: String,
    #[serde(with = "ts_seconds")]
    pub refresh_token_exp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegenTokenReqBody {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegenTokenResBody {
    pub access_token: String,
    #[serde(with = "ts_seconds")]
    pub access_token_exp: DateTime<Utc>,
    pub refresh_token: Option<String>,
    #[serde(with = "ts_seconds_option")]
    pub refresh_token_exp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUserInfoResBody {
    pub username: String,
    pub status: UserStatus,
    pub id: i32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SetStatusReqBody {
    pub status: UserStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendMessageReqBody {
    pub content: String,
    pub receiver: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetMessagesRetBody {
    pub messages: Vec<Msg>,
    pub row_count: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrActions {
    RefreshToken,
    PasswordInvalid,
    UserExists,
    Relogin,
    Other,
}

impl From<&str> for ErrActions {
    fn from(value: &str) -> Self {
        match value {
            "refresh_token" => ErrActions::RefreshToken,
            "pass_invalid" => ErrActions::PasswordInvalid,
            "user_exists" => ErrActions::UserExists,
            _ => ErrActions::Other,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResMessage {
    pub msg: String,
    pub action: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
pub enum UserStatus {
    #[default]
    OFFLINE = 0,
    ONLINE = 1,
    DND = 2,
}

#[derive(Debug, Clone, PartialEq, ImplicitClone)]
pub struct UserInfo {
    pub id: i32,
    pub name: AttrValue,
    pub status: UserStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WSMessage {
    // user_status, user_id
    ChangeStatus(UserStatus, i32),
    // timestamp, sender_id, content
    SendMessage(DateTime<Utc>, i32, String),
    // me_id, friend_ids
    Sync(i32, Vec<i32>),
}

impl WSMessage {
    /// This will return None if the byte buffer:
    /// - is the wrong size
    /// - has malformed data inside
    pub fn from_bytes(buffer: &Vec<u8>) -> Option<Self> {
        if buffer.len() < size_of::<u8>() {
            return None;
        }

        let identifier = buffer[0];
        let mut offset = size_of::<u8>();

        match identifier {
            // ChangeStatus
            0x01 => {
                // 0 - Identifier
                // 1 - new_status
                // 2 - user_id
                // Layout: | 0 (u8) | 1 (u8) | 2 (i32) |

                if buffer.len() != size_of::<u8>() * 2 + size_of::<i32>() {
                    return None;
                }

                let new_status: UserStatus = serde_json::from_str(&format!("{}", buffer[offset]))
                    .expect("Message malformed");

                offset += size_of::<u8>();

                let user_id: i32 = from_byte_slice(&buffer[..], &mut offset);

                Some(WSMessage::ChangeStatus(new_status, user_id))
            }
            // SendMessage
            0x02 => {
                // 0 - Identifier
                // 1 - timestamp
                // 9 - user_id
                // 13 - content
                // Layout: | 0 (u8) | 1 (i64) | 9 (i32) | 13 (String) |

                if !(buffer.len() > size_of::<u8>() + size_of::<i64>() + size_of::<i32>()) {
                    return None;
                }

                let slice = buffer.as_slice();

                let timestamp: i64 = from_byte_slice(slice, &mut offset);
                let timestamp = if let Some(time) = DateTime::from_timestamp_secs(timestamp) {
                    time
                } else {
                    return None;
                };

                let user_id: i32 = from_byte_slice(slice, &mut offset);

                let content = if let Ok(str) = String::from_utf8(buffer[offset..].to_vec()) {
                    str
                } else {
                    return None;
                };

                Some(WSMessage::SendMessage(timestamp, user_id, content))
            }
            // Sync
            0x03 => {
                // 0 - Identifier
                // 1 - me_id
                // 2 - friend_ids
                // Layout: | 0 (u8) | 1 (i32) | 5 (Vec<i32>) |

                if !(buffer.len() >= size_of::<u8>() + size_of::<i32>()
                    && (buffer.len() - offset) % size_of::<i32>() == 0)
                {
                    return None;
                }

                let slice = buffer.as_slice();

                let me_id: i32 = from_byte_slice(slice, &mut offset);

                let friend_ids: Vec<i32> = (offset..buffer.len())
                    .step_by(size_of::<i32>())
                    .map(|_| from_byte_slice::<i32, _>(slice, &mut offset))
                    .collect();

                Some(WSMessage::Sync(me_id, friend_ids))
            }
            _ => unreachable!(),
        }
    }
}

fn from_byte_slice<T, const N: usize>(slice: &[u8], offset: &mut usize) -> T
where
    T: FromBytes<Bytes = [u8; N]>,
{
    let mut stack_buf = [0u8; N];
    stack_buf.copy_from_slice(&slice[*offset..*offset + size_of::<T>()]);
    *offset += size_of::<T>();
    T::from_be_bytes(&stack_buf)
}
