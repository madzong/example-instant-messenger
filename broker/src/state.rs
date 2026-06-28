use chrono::{DateTime, Utc};
use instant_messenger_common::{DisconnectReqBody, UserStatus};
use scc::HashMap;
use tokio::sync::mpsc;

use crate::{api_gateway::ApiGateway, error::AppError, types::Message};

pub struct AppState {
    pub api_gateway: ApiGateway,
    pub client_map: HashMap<i32, mpsc::UnboundedSender<Message>>,
}

impl AppState {
    pub fn new() -> Self {
        let api_gateway = ApiGateway::new();
        let client_map = HashMap::default();

        Self {
            api_gateway,
            client_map,
        }
    }

    pub async fn add_client(
        &self,
        token: &str,
    ) -> Result<(i32, mpsc::UnboundedReceiver<Message>), AppError> {
        let (tx, rx) = mpsc::unbounded_channel();

        let user_info = self.api_gateway.connect_client(token).await?;
        let id = user_info.user_id;

        log::debug!("Adding client {id}");

        if let Some(old) = self.client_map.upsert_async(id, tx).await {
            let _ = old.send(Message::Close);
        }

        Ok((id, rx))
    }

    pub async fn remove_user(&self, user_id: i32) -> Result<(), AppError> {
        log::debug!("Removing user {user_id}");

        self.api_gateway.disconnect_client(&DisconnectReqBody { user_id }).await?;

        self.client_map.remove_async(&user_id).await;

        Ok(())
    }

    pub async fn update_user_status(
        &self,
        id: i32,
        friends: Vec<i32>,
        new_status: UserStatus,
    ) -> Result<(), AppError> {
        for friend in &friends {
            self.client_map
                .read_async(friend, |_, v| {
                    let _ = v.send(Message::ChangeStatus(new_status, id));
                })
                .await;
        }

        Ok(())
    }

    pub async fn send_message(
        &self,
        receiver_id: i32,
        sender_id: i32,
        content: String,
        timestamp: DateTime<Utc>,
    ) {
        self.client_map
            .read_async(&receiver_id, |_, v| {
                let _ = v
                    .send(Message::SendMessage(content, timestamp, sender_id));
            })
            .await;
    }

    pub async fn cleanup(&self) {
        let mut users = vec![];
        self.client_map
            .iter_async(|k, _| {
                users.push(*k);
                true
            })
            .await;

        for user in &users {
            let _ = self.remove_user(*user).await;
        }
    }
}
