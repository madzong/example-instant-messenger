use crate::{
    services::storage,
    types::{
        ErrActions, LoginReqBody, LoginResBody, RegenTokenReqBody, RegenTokenResBody,
        RegisterReqBody, ResMessage,
    },
};
use gloo_net::http::Request;

pub async fn login(login: String, password: String) -> anyhow::Result<Result<(), ErrActions>> {
    let req_body = LoginReqBody { login, password };

    let response = Request::post("http://api.localhost/login")
        .json(&req_body)
        .expect("Should be valid JSON")
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

    let resp_data: LoginResBody = response.json().await?;

    let token = resp_data.refresh_token;
    let token_exp = resp_data.refresh_token_exp;

    storage::store_refresh_token(&token, token_exp);
    refresh_access_token().await
}

pub async fn register(login: String, password: String) -> anyhow::Result<Result<(), ErrActions>> {
    let req_body = RegisterReqBody { login, password };

    let response = Request::post("http://api.localhost/register")
        .json(&req_body)
        .expect("Should be valid JSON")
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

    let resp_data: LoginResBody = response.json().await?;

    let token = resp_data.refresh_token;
    let token_exp = resp_data.refresh_token_exp;

    storage::store_refresh_token(&token, token_exp);
    // uuhhhh sure?
    refresh_access_token().await
}

pub async fn refresh_access_token() -> anyhow::Result<Result<(), ErrActions>> {
    let refresh_token = if let Some(token) = storage::get_refresh_token() {
        token
    } else {
        return Ok(Err(ErrActions::Relogin));
    };

    let req_body = RegenTokenReqBody { refresh_token };

    let response = Request::post("http://api.localhost/regen_token")
        .json(&req_body)
        .expect("Should be valid JSON")
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

    let resp_data: RegenTokenResBody = response.json().await?;

    let access_token = resp_data.access_token;
    let access_token_exp = resp_data.access_token_exp;

    storage::store_access_token(&access_token, access_token_exp);

    if let Some(refresh_token) = resp_data.refresh_token {
        let refresh_token_exp = resp_data
            .refresh_token_exp
            .expect("If refresh_token is not None, then refresh_token_exp also is not");
        storage::store_refresh_token(&refresh_token, refresh_token_exp);
    }

    Ok(Ok(()))
}

pub async fn check_token() -> Result<(), ErrActions> {
    if let None = storage::get_access_token() {
        match refresh_access_token().await {
            Err(_) => Err(ErrActions::Relogin),
            Ok(Err(ErrActions::Relogin)) => Err(ErrActions::Relogin),
            Ok(Err(ErrActions::Other)) => Err(ErrActions::Other),
            Ok(Err(_)) => unreachable!(),
            Ok(Ok(())) => Ok(()),
        }
    } else {
        Ok(())
    }
}
