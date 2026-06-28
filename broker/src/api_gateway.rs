use std::{env, time::Duration};

use instant_messenger_common::ConnectRetBody;
use reqwest::StatusCode;
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug)]
pub struct ApiGateway {
    http_client: reqwest::Client,
    api_host: String,
    comms_secret: String,
}

impl ApiGateway {
    pub fn new() -> Self {
        let http_client = reqwest::Client::new();
        let api_host = env::var("API_HOST").expect("API_HOST environment variable not set");
        let comms_secret =
            env::var("COMMS_SECRET").expect("COMMS_SECRET environment variable not set");

        Self {
            http_client,
            api_host,
            comms_secret,
        }
    }

    async fn make_request_post(
        &self,
        endpoint: &str,
        body: Option<impl Serialize>,
        access_token: Option<&str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let builder = self.http_client
            .post(&format!("http://{}/{}", self.api_host, endpoint))
            .json(&body)
            .header("X-Internal-Communication", &self.comms_secret);

        let builder = if let Some(token) = access_token {
            builder.bearer_auth(token)
        } else {
            builder
        };

        // Example retry functionality
        let mut response = builder.try_clone().unwrap().send().await;
        let mut attempts = 0;
        let failure_timeout = 300;
        while response.is_err() && attempts <= 3 {
            let timeout = failure_timeout * attempts;
            log::error!("Request failed. Retrying in {}ms", timeout);
            tokio::time::sleep(Duration::from_millis(timeout)).await;

            response = builder.try_clone().unwrap().send().await;
            attempts += 1;
        }

        Ok(response?)
    }

    pub async fn connect_client(&self, token: &str) -> Result<ConnectRetBody, AppError> {
        let response = self.make_request_post("connect", None::<()>, Some(token)).await?;

        let status = response.status();
        log::debug!("/connect: Status {status}");

        match status {
            StatusCode::UNAUTHORIZED => return Err(AppError::Unauthorized),
            StatusCode::UNPROCESSABLE_ENTITY => return Err(AppError::Unprocessable),
            _ => (),
        }

        let resp_body = response.json().await?;

        Ok(resp_body)
    }

    pub async fn disconnect_client(&self, body: impl Serialize) -> Result<(), AppError> {
        let response = self.make_request_post("disconnect", Some(body), None).await?;

        let status = response.status();
        log::debug!("/disconnect: Status {status}");

        Ok(())
    }
}
