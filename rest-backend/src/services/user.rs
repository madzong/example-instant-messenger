use instant_messenger_common::{
    ConnectRetBody, MessageReqBody, NewFriendshipReqBody, SendMessageReqBody, SetStatusReqBody,
    UpdateStatusReqBody, UserInfo, UserStatus,
    tokens::{self, ClaimsAccess, TokenType},
};

use crate::{error::AppError, state::State};

pub async fn set_status(
    new_status: UserStatus,
    access_token: &str,
    state: &State,
) -> Result<(), AppError> {
    let db = &state.db_client;

    let access_claims: ClaimsAccess = tokens::decode_token(access_token, &state.secret)?;

    if let TokenType::Refresh = access_claims.typ {
        return Err(AppError::Unauthorized);
    }

    let user_id = access_claims.sub;

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
        .patch(format!("{}/update_status", broker_host))
        .body(serde_json::to_string(&request_body).unwrap())
        .send()
        .await?;

    Ok(())
}

pub async fn send_message(
    send_message_body: &SendMessageReqBody,
    access_token: &str,
    state: &State,
) -> Result<(), AppError> {
    let db = &state.db_client;
    let receiver_id = send_message_body.receiver;
    let content = &send_message_body.content;

    let access_claims: ClaimsAccess = tokens::decode_token(access_token, &state.secret)?;

    if let TokenType::Refresh = access_claims.typ {
        return Err(AppError::Unauthorized);
    }

    let sender_id = access_claims.sub;

    if sender_id == receiver_id {
        return Err(AppError::BadRequest);
    }

    db.query(
        "INSERT INTO messages (sender_id, receiver_id, contents, sent_at)
                   VALUES ($1, $2, $3, CURRENT_TIMESTAMP)",
        &[&sender_id, &receiver_id, &content],
    )
    .await?;

    let friends: bool;

    if let Ok(_) = db
        .query_one(
            "SELECT * FROM friendships WHERE user_id = $1 AND friend_id = $2",
            &[&sender_id, &receiver_id],
        )
        .await
    {
        friends = true;
    } else {
        friends = false;
    }

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

        http_client
            .patch(format!("{}/new_friendship", broker_host))
            .body(serde_json::to_string(&request_body).unwrap())
            .send()
            .await?;
    }

    let request_body = MessageReqBody {
        content: content.clone(),
        sender: sender_id,
        receiver: receiver_id,
    };
    http_client
        .post(format!("{}/new_message", broker_host))
        .body(serde_json::to_string(&request_body).unwrap())
        .send()
        .await?;

    Ok(())
}

pub async fn connect(
    access_token: &str,
    internal_token: &str,
    state: &State,
) -> Result<ConnectRetBody, AppError> {
    let db = &state.db_client;
    let comms_secret = &state.comms_secret;

    if internal_token != comms_secret {
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

    let friendships = rows.iter().map(|row| row.get("id")).collect();

    Ok(ConnectRetBody {
        user_info,
        friendships,
    })
}

pub async fn disconnect(
    access_token: &str,
    internal_token: &str,
    state: &State,
) -> Result<(), AppError> {
    let comms_secret = &state.comms_secret;

    if internal_token != comms_secret {
        return Err(AppError::Unauthorized);
    }

    let new_status = UserStatus::Online;
    set_status(new_status, access_token, state).await?;

    Ok(())
}
