use std::{error::Error, fmt};

use chrono::{DateTime, Utc};
use instant_messenger_common::{UserInfo, UserMessage, UserStatus};
use tokio_postgres::{Error as PostgresError, NoTls};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum DBError {
    NotFound,
    NoResults,
    ConnectError,
    Other,
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DBError::NotFound => write!(f, "Record not found"),
            DBError::NoResults => write!(f, "Query returned 0 rows"),
            DBError::ConnectError => write!(f, "Database connection failure"),
            DBError::Other => write!(f, "Unknown error"),
        }
    }
}

impl std::error::Error for DBError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            _ => None,
        }
    }
}

impl From<PostgresError> for DBError {
    fn from(err: PostgresError) -> Self {
        log::error!("DB error: {err}");

        if err.as_db_error().is_none() && err.to_string().contains("row order was empty") {
            return DBError::NoResults;
        }

        if err.is_closed()
            || err
                .as_db_error()
                .map_or(false, |e| e.code().code().starts_with("08"))
        {
            return DBError::ConnectError;
        }

        if let Some(source) = err.source() {
            if source.is::<std::io::Error>() {
                return DBError::ConnectError;
            }
        }

        if let Some(db_error) = err.as_db_error() {
            match db_error.code().code() {
                "02000" => DBError::NoResults,
                _ => DBError::Other,
            }
        } else {
            DBError::Other
        }
    }
}

#[derive(Debug)]
pub struct DBManager {
    db: tokio_postgres::Client,
}

impl DBManager {
    pub async fn new(creds: DBCredentials) -> Result<Self, DBError> {
        let (db_client, db_connection) =
            tokio_postgres::connect(&creds.as_config_string(), NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = db_connection.await {
                log::error!("Database connection error: {}", e);
            }
        });

        Ok(Self { db: db_client })
    }

    pub async fn get_user_friendships(
        &self,
        user_id: &UserIdentifier,
    ) -> Result<Vec<i32>, DBError> {
        let user_id = self.user_identifier_to_id(user_id).await?;

        let rows = self
            .db
            .query(
                "SELECT friendships.friend_id FROM friendships
                  WHERE friendships.user_id = $1",
                &[&user_id],
            )
            .await?;

        let friendships = rows.iter().map(|row| row.get("friend_id")).collect();

        Ok(friendships)
    }

    pub async fn get_user_friendships_full(
        &self,
        user_id: &UserIdentifier,
    ) -> Result<Vec<UserInfo>, DBError> {
        let user_id = self.user_identifier_to_id(user_id).await?;

        let rows = self
            .db
            .query(
                "SELECT friendships.friend_id, users.username, users.status
                   FROM friendships JOIN users ON friendships.friend_id = users.id
                  WHERE friendships.user_id = $1",
                &[&user_id],
            )
            .await?;

        let mut friendships = vec![];
        for row in rows {
            let user_info = UserInfo {
                id: row.get("friend_id"),
                username: row.get("username"),
                status: UserStatus::from_repr(row.get::<_, i32>("status") as u8)
                    .expect("Database entries should be well-formed"),
            };

            friendships.push(user_info);
        }

        Ok(friendships)
    }

    pub async fn get_messages_paginated(
        &self,
        user1_id: &UserIdentifier,
        user2_id: &UserIdentifier,
        limit: i64,
        page: i64,
    ) -> Result<MessagesPaginated, DBError> {
        let user1_id = self.user_identifier_to_id(user1_id).await?;
        let user2_id = self.user_identifier_to_id(user2_id).await?;

        let offset = limit * page;

        let rows = self
            .db
            .query(
                "SELECT *, COUNT(*) OVER() AS row_count
               FROM messages
              WHERE (sender_id = $1 AND receiver_id = $2) OR (receiver_id = $1 AND sender_id = $2)
              ORDER BY sent_at DESC
              LIMIT $3
             OFFSET $4",
                &[&user1_id, &user2_id, &limit, &offset],
            )
            .await?;

        if rows.len() == 0 {
            return Err(DBError::NotFound);
        }

        let rows_total: i64 = rows[0].get("row_count");

        let mut messages: Vec<UserMessage> = vec![];
        for row in rows {
            let sender: i32 = row.get("sender_id");
            let content: String = row.get("contents");
            let timestamp: DateTime<Utc> = row.get("sent_at");

            messages.push(UserMessage {
                content,
                timestamp,
                sender,
            });
        }

        Ok(MessagesPaginated {
            messages,
            rows_total,
        })
    }

    pub async fn user_identifier_to_id(&self, identifier: &UserIdentifier) -> Result<i32, DBError> {
        match identifier {
            UserIdentifier::ID(id) => Ok(*id),
            UserIdentifier::Username(name) => {
                let row = self
                    .db
                    .query_one("SELECT id FROM users WHERE username = $1", &[&name])
                    .await?;

                Ok(row.get("id"))
            }
        }
    }

    pub async fn user_identifier_to_name(
        &self,
        identifier: &UserIdentifier,
    ) -> Result<String, DBError> {
        match identifier {
            UserIdentifier::ID(id) => {
                let row = self
                    .db
                    .query_one("SELECT username FROM users WHERE id = $1", &[&id])
                    .await?;

                Ok(row.get("username"))
            }
            UserIdentifier::Username(name) => Ok(name.clone()),
        }
    }

    pub async fn set_user_status(
        &self,
        user_id: &UserIdentifier,
        new_status: UserStatus,
    ) -> Result<(), DBError> {
        let user_id = self.user_identifier_to_id(user_id).await?;

        self.db
            .execute(
                "UPDATE users SET status = $1 WHERE id = $2",
                &[&(new_status as i32), &user_id],
            )
            .await?;

        Ok(())
    }

    pub async fn get_user_info(&self, user_id: &UserIdentifier) -> Result<UserInfo, DBError> {
        let user_id = self.user_identifier_to_id(user_id).await?;

        let row = self
            .db
            .query_one(
                "SELECT username, status
               FROM users
              WHERE users.id = $1",
                &[&user_id],
            )
            .await?;

        let id = user_id;
        let username: String = row.get("username");
        let status = UserStatus::from_repr(row.get::<_, i32>("status") as u8)
            .expect("DB data should be clean");

        let user_info = UserInfo {
            id,
            username,
            status,
        };

        Ok(user_info)
    }

    pub async fn insert_message(
        &self,
        sender: &UserIdentifier,
        receiver: &UserIdentifier,
        timestamp: DateTime<Utc>,
        content: &str,
    ) -> Result<(), DBError> {
        let sender = self.user_identifier_to_id(sender).await?;
        let receiver = self.user_identifier_to_id(receiver).await?;

        self.db
            .execute(
                "INSERT INTO messages (sender_id, receiver_id, contents, sent_at)
                  VALUES          ($1,        $2,          $3,       $4)",
                &[&sender, &receiver, &content, &timestamp],
            )
            .await?;

        Ok(())
    }

    pub async fn is_user_friends_with(
        &self,
        user1_id: &UserIdentifier,
        user2_id: &UserIdentifier,
    ) -> Result<bool, DBError> {
        let user1_id = self.user_identifier_to_id(user1_id).await?;
        let user2_id = self.user_identifier_to_id(user2_id).await?;

        let row = self
            .db
            .query_one(
                "SELECT EXISTS(
                     SELECT 1 FROM friendships
                      WHERE (user_id = $1 AND friend_id = $2)
                )",
                &[&user1_id, &user2_id],
            )
            .await?;

        Ok(row.get(0))
    }

    pub async fn insert_friendship(
        &self,
        user1_id: &UserIdentifier,
        user2_id: &UserIdentifier,
    ) -> Result<(), DBError> {
        let user1_id = self.user_identifier_to_id(user1_id).await?;
        let user2_id = self.user_identifier_to_id(user2_id).await?;

        self.db
            .execute(
                "INSERT INTO friendships
                 VALUES ($1, $2), ($2, $1)",
                &[&user1_id, &user2_id],
            )
            .await?;

        Ok(())
    }

    pub async fn clean_stale_tokens(&self, user_id: i32) -> Result<(), DBError> {
        self.db
            .execute(
                "DELETE FROM refresh_tokens WHERE user_id = $1 AND expires_at < NOW()",
                &[&user_id],
            )
            .await?;

        Ok(())
    }

    pub async fn insert_refresh_token(
        &self,
        uuid: uuid::Uuid,
        user_id: i32,
        expiry: DateTime<Utc>,
    ) -> Result<(), DBError> {
        self.db
            .execute(
                "INSERT INTO refresh_tokens VALUES ($1, $2, $3)",
                &[&uuid, &user_id, &expiry],
            )
            .await?;

        Ok(())
    }

    pub async fn get_hash_and_id(&self, username: &str) -> Result<(i32, String), DBError> {
        let row = self
            .db
            .query_one(
                "SELECT passwords.hash, users.id
                   FROM users JOIN passwords ON users.password_id = passwords.id
                  WHERE users.username = $1",
                &[&username],
            )
            .await?;

        let user_id: i32 = row.get("id");
        let password_hash: String = row.get("hash");

        Ok((user_id, password_hash))
    }

    pub async fn does_user_exist(&self, username: &str) -> Result<bool, DBError> {
        let row = self
            .db
            .query_one(
                "SELECT EXISTS(
                    SELECT 1 FROM users WHERE username = $1
                )",
                &[&username],
            )
            .await?;

        Ok(row.get(0))
    }

    pub async fn insert_user(&self, username: &str, password_hash: &str) -> Result<i32, DBError> {
        let row = self
            .db
            .query_one(
                "INSERT INTO passwords (hash) VALUES ($1) RETURNING id",
                &[&password_hash],
            )
            .await?;

        let password_id: i32 = row.get(0);

        let row = self
            .db
            .query_one(
                "INSERT INTO users (username, password_id)
                                          VALUES       ($1,       $2) 
                                       RETURNING id",
                &[&username, &password_id],
            )
            .await?;

        let user_id: i32 = row.get(0);

        Ok(user_id)
    }

    pub async fn cleanup(&self) {
        // Cleanup should remain errorless
        let _ = self.db.query("UPDATE users SET status = 0", &[]).await;
    }
}

#[derive(Debug, Clone)]
pub struct MessagesPaginated {
    pub messages: Vec<UserMessage>,
    pub rows_total: i64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum UserIdentifier {
    Username(String),
    ID(i32),
}

#[derive(Clone, Debug)]
pub struct DBCredentials {
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub db_host: String,
}

impl DBCredentials {
    pub fn as_config_string(&self) -> String {
        format!(
            "user={} password={} dbname={} host={}",
            self.db_user, self.db_password, self.db_name, self.db_host
        )
    }
}
