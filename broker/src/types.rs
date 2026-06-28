use bytes::{BufMut, Bytes, BytesMut};
use chrono::{DateTime, Utc};
use instant_messenger_common::UserStatus;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    // new_status, user_id
    ChangeStatus(UserStatus, i32),
    // contents, timestamp, sender_id
    SendMessage(String, DateTime<Utc>, i32),
    Close,
}

impl Into<Bytes> for Message {
    fn into(self) -> Bytes {
        match self {
            Message::ChangeStatus(new_status, user_id) => {
                let mut bytes = BytesMut::with_capacity(
                    // Packet identifier
                    size_of::<u8>() +
                    // new_status
                    size_of::<u8>() +
                    // user_id
                    size_of::<i32>(),
                );

                bytes.put_u8(0x01);
                bytes.put_u8(new_status as u8);
                bytes.put_i32(user_id);

                bytes.into()
            }
            Message::SendMessage(content, timestamp, user_id) => {
                let mut bytes = BytesMut::with_capacity(
                    // Packet identifier
                    size_of::<u8>() +
                    // timestamp
                    size_of::<i64>() +
                    // user_id
                    size_of::<i32>() +
                    // content
                    content.as_bytes().len(),
                );

                bytes.put_u8(0x02);
                bytes.put_i64(timestamp.timestamp());
                bytes.put_i32(user_id);
                bytes.put_slice(content.as_bytes());

                bytes.into()
            }
            _ => unreachable!("Other variants shouldn't be converted to bytes"),
        }
    }
}
