use std::env;

use chrono::{DateTime, Utc};
use instant_messenger_common::{DisconnectReqBody, UserStatus};
use scc::HashMap;
use tokio::sync::mpsc;

use crate::{
    error::AppError,
    types::{Message, UserData},
};

pub struct AppState {
    pub http_client: reqwest::Client,
    pub client_map: HashMap<i32, UserData>,
    pub api_host: String,
    pub comms_secret: String,
}

impl AppState {
    pub fn new() -> Self {
        let http_client = reqwest::Client::new();
        let client_map = HashMap::default();
        let api_host = env::var("API_HOST").expect("API_HOST environment variable not set");
        let comms_secret =
            env::var("COMMS_SECRET").expect("COMMS_SECRET environment variable not set");

        Self {
            http_client,
            client_map,
            api_host,
            comms_secret,
        }
    }

    pub async fn add_client(
        &self,
        id: i32,
        friends: Vec<i32>,
        status: UserStatus,
    ) -> mpsc::UnboundedReceiver<Message> {
        let (tx, rx) = mpsc::unbounded_channel();

        log::debug!("Adding client {id}");

        if let Some(old) = self
            .client_map
            .upsert_async(
                id,
                UserData {
                    friends,
                    sender: tx,
                    status,
                },
            )
            .await
        {
            let _ = old.sender.send(Message::Close);
        }

        let _ = self.update_user_status(id, status).await;

        rx
    }

    pub async fn remove_user(&self, user_id: i32) -> Result<(), AppError> {
        log::debug!("Removing user {user_id}");

        let http_client = &self.http_client;
        let api_host = &self.api_host;
        let comms_secret = &self.comms_secret;

        http_client
            .post(format!("http://{}/disconnect", api_host))
            .header("X-Internal-Communication", comms_secret)
            .json(&DisconnectReqBody { user_id })
            .send()
            .await?;

        self.client_map.remove_async(&user_id).await;

        Ok(())
    }

    pub async fn update_user_status(
        &self,
        id: i32,
        new_status: UserStatus,
    ) -> Result<(), AppError> {
        let mut entry = self
            .client_map
            .get_async(&id)
            .await
            .ok_or(AppError::NonexistentUser)?;

        entry.get_mut().status = new_status;

        // Cloning Vec<i32> is NOT cheap
        let friends_list = entry.get().friends.clone();

        // Drop so we don't deadlock by mistake
        drop(entry);

        for friend in friends_list {
            self.client_map
                .read_async(&friend, |_, v| {
                    let _ = v.sender.send(Message::ChangeStatus(new_status, id));
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
                    .sender
                    .send(Message::SendMessage(content, timestamp, sender_id));
            })
            .await;
    }

    pub async fn new_friendship(&self, user_id: i32, friend_id: i32) {
        let _ = self
            .client_map
            .entry_async(user_id)
            .await
            .and_modify(|v| v.friends.push(friend_id));

        let _ = self
            .client_map
            .entry_async(friend_id)
            .await
            .and_modify(|v| v.friends.push(user_id));
    }

    pub async fn get_user_info(&self, user_id: i32) -> Vec<i32> {
        log::debug!("Getting user {user_id} info");
        let entry = self.client_map.get_async(&user_id).await.unwrap();

        entry.get().friends.clone()
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
