use gloo_net::http::Request;

use crate::{
    services::{auth, storage},
    types::{
        ErrActions, GetContactsRetBody, GetMessagesRetBody, GetUserInfoResBody, ResMessage,
        SendMessageReqBody, SetStatusReqBody, UserInfo, UserStatus,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub enum UserIdentifier {
    ID(i32),
    Username(String),
}

pub async fn get_user_info(
    user_identifier: Option<UserIdentifier>,
) -> anyhow::Result<Result<UserInfo, ErrActions>> {
    let token = match storage::get_access_token() {
        Some(t) => t,
        None => {
            if let Err(e) = auth::check_token().await {
                return Ok(Err(e));
            } else {
                storage::get_access_token().unwrap()
            }
        }
    };

    let response = Request::get(&match user_identifier {
        Some(UserIdentifier::ID(user_id)) => {
            format!("http://api.localhost/get_user_info?user_id={user_id}")
        }
        Some(UserIdentifier::Username(username)) => {
            format!("http://api.localhost/get_user_info?username={username}")
        }
        None => {
            "http://api.localhost/get_user_info".to_string()
        }
    })
    .header("Authorization", &format!("Bearer {token}"))
    .send()
    .await?;

    if !response.ok() {
        let msg: ResMessage = response.json().await?;
        let action = msg
            .action
            .map(|a| ErrActions::from(a.as_str()))
            .unwrap_or(ErrActions::Other);

        return Ok(Err(action));
    }

    let resp_data: GetUserInfoResBody = response.json().await?;

    let user_info = UserInfo {
        id: resp_data.id,
        username: resp_data.username.into(),
        status: resp_data.status,
    };

    return Ok(Ok(user_info));
}

pub async fn change_status(new_status: UserStatus) -> anyhow::Result<Result<(), ErrActions>> {
    let token = match storage::get_access_token() {
        Some(t) => t,
        None => {
            if let Err(e) = auth::check_token().await {
                return Ok(Err(e));
            } else {
                storage::get_access_token().unwrap()
            }
        }
    };

    let body = SetStatusReqBody { status: new_status };

    let response = Request::patch(&format!("http://api.localhost/set_status"))
        .header("Authorization", &format!("Bearer {token}"))
        .json(&body)?
        .send()
        .await?;

    if !response.ok() {
        let msg: ResMessage = response.json().await?;
        let action = msg
            .action
            .map(|a| ErrActions::from(a.as_str()))
            .unwrap_or(ErrActions::Other);

        return Ok(Err(action));
    }

    Ok(Ok(()))
}

pub async fn send_message(
    receiver_id: i32,
    content: String,
) -> anyhow::Result<Result<(), ErrActions>> {
    let token = match storage::get_access_token() {
        Some(t) => t,
        None => {
            if let Err(e) = auth::check_token().await {
                return Ok(Err(e));
            } else {
                storage::get_access_token().unwrap()
            }
        }
    };

    let body = SendMessageReqBody {
        content,
        receiver: receiver_id,
    };

    let response = Request::post("http://api.localhost/send_message")
        .header("Authorization", &format!("Bearer {token}"))
        .json(&body)?
        .send()
        .await?;

    if !response.ok() {
        let msg: ResMessage = response.json().await?;
        let action = msg
            .action
            .map(|a| ErrActions::from(a.as_str()))
            .unwrap_or(ErrActions::Other);

        return Ok(Err(action));
    }

    Ok(Ok(()))
}

pub async fn get_messages(
    user_id: i32,
    page: i32,
) -> anyhow::Result<Result<GetMessagesRetBody, ErrActions>> {
    let token = match storage::get_access_token() {
        Some(t) => t,
        None => {
            if let Err(e) = auth::check_token().await {
                return Ok(Err(e));
            } else {
                storage::get_access_token().unwrap()
            }
        }
    };

    let response = Request::get(&format!(
        "http://api.localhost/get_messages?user_id={}&page={}",
        user_id, page
    ))
    .header("Authorization", &format!("Bearer {token}"))
    .send()
    .await?;

    if !response.ok() {
        let msg: ResMessage = response.json().await?;
        let action = msg
            .action
            .map(|a| ErrActions::from(a.as_str()))
            .unwrap_or(ErrActions::Other);

        return Ok(Err(action));
    }

    let resp_data: GetMessagesRetBody = response.json().await?;

    Ok(Ok(resp_data))
}

pub async fn get_contacts() -> anyhow::Result<Result<Vec<UserInfo>, ErrActions>> {
    let token = match storage::get_access_token() {
        Some(t) => t,
        None => {
            if let Err(e) = auth::check_token().await {
                return Ok(Err(e));
            } else {
                storage::get_access_token().unwrap()
            }
        }
    };

    let response = Request::get("http://api.localhost/get_contacts")
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?;

    if !response.ok() {
        let msg: ResMessage = response.json().await?;
        let action = msg
            .action
            .map(|a| ErrActions::from(a.as_str()))
            .unwrap_or(ErrActions::Other);

        return Ok(Err(action));
    }

    let resp_data: GetContactsRetBody = response.json().await?;

    Ok(Ok(resp_data.friends))
}
