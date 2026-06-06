use tokio::sync::mpsc;
use tokio::time::Instant;
use instant_messenger_common::{UserInfo, UserStatus};

#[derive(Debug, Clone)]
pub enum Message {
    // new_status, user_id
    ChangeStatus(UserStatus, u32),
    // contents, timestamp, sender_id
    SendMessage(String, Instant, u32),
}

pub struct UserData {
    pub friends: Vec<u32>,
    pub sender: mpsc::UnboundedSender<Message>,
    pub info: UserInfo,
}
