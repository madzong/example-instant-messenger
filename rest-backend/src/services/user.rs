use chrono::Utc;
use instant_messenger_common::{
    ConnectRetBody, DisconnectReqBody, GetMessagesRetBody, GetUserInfoRetBody, MessageReqBody,
    UpdateStatusReqBody, UserInfo, UserStatus, tokens::ClaimsAccess,
};

use crate::{
    db_services::{DBError, UserIdentifier},
    error::AppError,
    state::AppState,
};

pub async fn set_status(
    new_status: UserStatus,
    access_token: &str,
    state: &AppState,
) -> Result<(), AppError> {
    let access_claims: ClaimsAccess = state.validate_access_token(access_token)?;

    let user_id = access_claims.sub;

    _set_status(new_status, user_id, state).await?;

    Ok(())
}

async fn _set_status(
    new_status: UserStatus,
    user_id: i32,
    state: &AppState,
) -> Result<(), AppError> {
    let identifier = UserIdentifier::ID(user_id);

    let db = &state.db_manager;
    db.set_user_status(&identifier, new_status).await?;
    let friends = db.get_user_friendships(&identifier).await?;

    let http_client = &state.http_client;
    let broker_host = &state.broker_host;
    let request_body = UpdateStatusReqBody {
        user_id,
        new_status,
        send_to: friends,
    };

    http_client
        .patch(format!("http://{}/update_status", broker_host))
        .json(&request_body)
        .send()
        .await?;

    Ok(())
}

pub async fn send_message(
    message_content: String,
    receiver_id: i32,
    access_token: &str,
    state: &AppState,
) -> Result<(), AppError> {
    let db = &state.db_manager;

    let access_claims: ClaimsAccess = state.validate_access_token(access_token)?;

    let sender_id = access_claims.sub;

    if sender_id == receiver_id {
        return Err(AppError::BadRequest);
    }

    let sender = UserIdentifier::ID(sender_id);
    let receiver = UserIdentifier::ID(receiver_id);
    let timestamp = Utc::now();

    db.insert_message(&sender, &receiver, timestamp, &message_content)
        .await?;

    let friends = db.is_user_friends_with(&sender, &receiver).await?;

    if !friends {
        db.insert_friendship(&sender, &receiver).await?;
    }

    let request_body = MessageReqBody {
        sender: sender_id,
        receiver: receiver_id,
        content: message_content,
        timestamp,
    };

    let http_client = &state.http_client;
    let broker_host = &state.broker_host;

    let _ = http_client
        .post(format!("http://{}/new_message", broker_host))
        .json(&request_body)
        .send()
        .await?;

    Ok(())
}

pub async fn connect(
    access_token: &str,
    internal_token: &str,
    state: &AppState,
) -> Result<ConnectRetBody, AppError> {
    state.validate_comms_secret(internal_token)?;

    let access_claims: ClaimsAccess = state.validate_access_token(access_token)?;

    let user_id = access_claims.sub;

    let new_status = UserStatus::Online;
    set_status(new_status, access_token, state).await?;

    Ok(ConnectRetBody { user_id })
}

pub async fn disconnect(
    disconnect_body: DisconnectReqBody,
    internal_token: &str,
    state: &AppState,
) -> Result<(), AppError> {
    state.validate_comms_secret(internal_token)?;

    let new_status = UserStatus::Offline;
    _set_status(new_status, disconnect_body.user_id, state).await?;

    Ok(())
}

pub async fn get_user_info(
    access_token: &str,
    user_identifier: Option<UserIdentifier>,
    state: &AppState,
) -> Result<GetUserInfoRetBody, AppError> {
    let access_claims = state.validate_access_token(access_token)?;

    let user_identifier = user_identifier.unwrap_or(UserIdentifier::ID(access_claims.sub));

    let db = &state.db_manager;

    let user_info = db.get_user_info(&user_identifier).await?;

    let ret_body = GetUserInfoRetBody {
        id: user_info.id,
        status: user_info.status,
        username: user_info.username,
    };

    Ok(ret_body)
}

pub async fn get_messages(
    access_token: &str,
    user_id: i32,
    limit: i64,
    page: i64,
    state: &AppState,
) -> Result<GetMessagesRetBody, AppError> {
    let access_claims: ClaimsAccess = state.validate_access_token(access_token)?;

    let sender_id = access_claims.sub;

    let user1_id = UserIdentifier::ID(sender_id);
    let user2_id = UserIdentifier::ID(user_id);

    let db = &state.db_manager;
    let messages = db
        .get_messages_paginated(&user1_id, &user2_id, limit, page)
        .await;

    if let Err(DBError::NotFound) = messages {
        return Ok(GetMessagesRetBody {
            messages: vec![],
            row_count: 0,
        });
    }

    let messages = messages?;

    let ret_body = GetMessagesRetBody {
        messages: messages.messages,
        row_count: messages.rows_total,
    };

    Ok(ret_body)
}

pub async fn get_contacts(access_token: &str, state: &AppState) -> Result<Vec<UserInfo>, AppError> {
    let access_claims = state.validate_access_token(access_token)?;
    let user_id = access_claims.sub;
    let identifier = UserIdentifier::ID(user_id);

    let friends = state
        .db_manager
        .get_user_friendships_full(&identifier)
        .await?;

    Ok(friends)
}
