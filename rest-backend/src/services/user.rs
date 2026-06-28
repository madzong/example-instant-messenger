use chrono::{DateTime, Utc};
use instant_messenger_common::{
    ConnectRetBody, DisconnectReqBody, GetMessagesRetBody, GetUserInfoRetBody, MessageReqBody,
    MessageSide, NewFriendshipReqBody, SendMessageReqBody, UpdateStatusReqBody, UserInfo,
    UserMessage, UserStatus,
    tokens::{self, ClaimsAccess, TokenType},
};
use log::error;

use crate::{error::AppError, state::AppState};

pub async fn set_status(
    new_status: UserStatus,
    access_token: &str,
    state: &AppState,
) -> Result<(), AppError> {
    let access_claims: ClaimsAccess = tokens::decode_token(access_token, &state.secret)?;

    if let TokenType::Refresh = access_claims.typ {
        return Err(AppError::Unauthorized);
    }

    let user_id = access_claims.sub;

    _set_status(new_status, user_id, state).await?;

    Ok(())
}

async fn _set_status(
    new_status: UserStatus,
    user_id: i32,
    state: &AppState,
) -> Result<(), AppError> {
    let db = &state.db_client;

    db.query(
        "UPDATE users SET status = $1 WHERE id = $2",
        &[&(new_status as i32), &user_id],
    )
    .await?;

    let http_client = &state.http_client;
    let broker_host = &state.broker_host;
    let request_body = UpdateStatusReqBody {
        user_id,
        new_status,
    };
    http_client
        .patch(format!("http://{}/update_status", broker_host))
        .json(&request_body)
        .send()
        .await?;

    Ok(())
}

pub async fn send_message(
    send_message_body: &SendMessageReqBody,
    access_token: &str,
    state: &AppState,
) -> Result<(), AppError> {
    let db = &state.db_client;
    let receiver_id = send_message_body.receiver;
    let content = &send_message_body.content;
    let timestamp = Utc::now();

    let access_claims: ClaimsAccess = tokens::decode_token(access_token, &state.secret)?;

    if let TokenType::Refresh = access_claims.typ {
        return Err(AppError::Unauthorized);
    }

    let sender_id = access_claims.sub;

    if sender_id == receiver_id {
        return Err(AppError::BadRequest);
    }

    log::debug!("/send_message: Inserting into DB");
    db.query(
        "INSERT INTO messages (sender_id, receiver_id, contents, sent_at)
                   VALUES ($1, $2, $3, $4)",
        &[&sender_id, &receiver_id, content, &timestamp],
    )
    .await?;

    let friends = is_friends_with(sender_id, receiver_id, state).await;

    let http_client = &state.http_client;
    let broker_host = &state.broker_host;

    if !friends {
        db.query(
            "INSERT INTO friendships (user_id, friend_id)
                  VALUES ($1, $2), ($2, $1)",
            &[&sender_id, &receiver_id],
        )
        .await?;

        let request_body = NewFriendshipReqBody {
            user_id: sender_id,
            friend_id: receiver_id,
        };

        log::debug!("/send_message: Sending friendship info to broker");
        http_client
            .patch(format!("http://{}/new_friendship", broker_host))
            .json(&request_body)
            .send()
            .await?;
    }

    let request_body = MessageReqBody {
        content: content.clone(),
        sender: sender_id,
        receiver: receiver_id,
        timestamp,
    };
    log::debug!("/send_message: Sending message to broker");
    let resp = http_client
        .post(format!("http://{}/new_message", broker_host))
        .json(&request_body)
        .send()
        .await?;

    log::debug!("/send_message: Broker response:\n{:#?}", resp);

    Ok(())
}

pub async fn connect(
    access_token: &str,
    internal_token: &str,
    state: &AppState,
) -> Result<ConnectRetBody, AppError> {
    let db = &state.db_client;
    let comms_secret = &state.comms_secret;

    if internal_token != comms_secret {
        error!("Internal token invalid");
        return Err(AppError::Unauthorized);
    }

    let access_claims: ClaimsAccess = tokens::decode_token(access_token, &state.secret)?;

    if let TokenType::Refresh = access_claims.typ {
        return Err(AppError::Unauthorized);
    }

    let user_id = access_claims.sub;

    let new_status = UserStatus::Online;
    set_status(new_status, access_token, state).await?;

    let user_info = UserInfo {
        id: user_id,
        status: new_status,
    };

    let rows = db
        .query(
            "SELECT friendships.friend_id FROM friendships
              WHERE friendships.user_id = $1",
            &[&user_id],
        )
        .await?;

    let friendships = rows.iter().map(|row| row.get("friend_id")).collect();

    Ok(ConnectRetBody {
        user_info,
        friendships,
    })
}

pub async fn disconnect(
    disconnect_body: DisconnectReqBody,
    internal_token: &str,
    state: &AppState,
) -> Result<(), AppError> {
    let comms_secret = &state.comms_secret;

    if internal_token != comms_secret {
        return Err(AppError::Unauthorized);
    }

    let new_status = UserStatus::Offline;
    _set_status(new_status, disconnect_body.user_id, state).await?;

    Ok(())
}

pub async fn is_friends_with(user_id: i32, friend_id: i32, state: &AppState) -> bool {
    if let Ok(_) = state
        .db_client
        .query_one(
            "SELECT * FROM friendships WHERE user_id = $1 AND friend_id = $2",
            &[&user_id, &friend_id],
        )
        .await
    {
        true
    } else {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserIdentifier {
    Username(String),
    ID(i32),
}

pub async fn get_user_info(
    access_token: &str,
    user_identifier: UserIdentifier,
    state: &AppState,
) -> Result<GetUserInfoRetBody, AppError> {
    let access_claims: ClaimsAccess = tokens::decode_token(access_token, &state.secret)?;

    if let TokenType::Refresh = access_claims.typ {
        return Err(AppError::Unauthorized);
    }

    let db = &state.db_client;

    let user_id = match user_identifier {
        UserIdentifier::ID(id) => id,
        UserIdentifier::Username(name) => {
            let row = db
                .query_one("SELECT id FROM users WHERE username = $1", &[&name])
                .await?;

            row.get("id")
        }
    };

    let _sender_id = access_claims.sub;

    let row = db
        .query_one(
            "SELECT username, status FROM users WHERE id = $1",
            &[&user_id],
        )
        .await?;

    let username = row.get("username");
    let status: i32 = row.get("status");

    let ret_body = GetUserInfoRetBody {
        id: user_id,
        status: serde_json::from_str(&format!("{}", status))?,
        username,
    };

    Ok(ret_body)
}

pub async fn get_messages(
    access_token: &str,
    user_id: i32,
    limit: i32,
    page: i32,
    state: &AppState,
) -> Result<GetMessagesRetBody, AppError> {
    let access_claims: ClaimsAccess = tokens::decode_token(access_token, &state.secret)?;

    if let TokenType::Refresh = access_claims.typ {
        return Err(AppError::Unauthorized);
    }

    let sender_id = access_claims.sub;

    let db = &state.db_client;

    let offset = page * limit;

    let rows = db
        .query(
            "SELECT *, COUNT(*) OVER() AS row_count
           FROM messages
          WHERE (sender_id = $1 AND receiver_id = $2) OR (receiver_id = $1 AND sender_id = $2)
          ORDER BY sent_at DESC
          LIMIT $3
         OFFSET $4",
            &[&sender_id, &user_id, &(limit as i64), &(offset as i64)],
        )
        .await?;

    if rows.len() == 0 {
        return Ok(GetMessagesRetBody {
            messages: vec![],
            row_count: 0,
        });
    }

    let row_count: i64 = rows[0].get("row_count");

    let mut messages: Vec<UserMessage> = vec![];
    for row in rows {
        let sender_id: i32 = row.get("sender_id");
        let content: String = row.get("contents");
        let timestamp: DateTime<Utc> = row.get("sent_at");

        let side = if sender_id == user_id {
            MessageSide::Recipient
        } else {
            MessageSide::Sender
        };

        messages.push(UserMessage {
            side,
            content,
            timestamp,
        });
    }

    let ret_body = GetMessagesRetBody {
        messages,
        row_count,
    };

    Ok(ret_body)
}
