use chrono::{DateTime, Utc, serde::ts_seconds};
use futures_util::{FutureExt, SinkExt, StreamExt, select};
use gloo_net::websocket::{Message, futures::WebSocket};
use implicit_clone::{ImplicitClone, unsync::IArray};
use serde::Deserialize;
use web_sys::console;
use yew::{
    platform::{pinned::mpsc, spawn_local},
    prelude::*,
};
use yew_router::prelude::*;

use crate::{
    Route,
    components::{ChatSidebar, ChatWindow},
    services::{
        auth, storage,
        user::{self, UserIdentifier},
    },
    types::{ErrActions, UserInfo, WSMessage},
};

#[derive(Clone, Debug, ImplicitClone, PartialEq, Deserialize)]
pub struct Msg {
    pub content: AttrValue,
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub sender: i32,
}

#[derive(Clone, Debug, Default)]
struct ChatState {
    active_chat: Option<i32>,
    recipient_name: Option<AttrValue>,
    chats: IArray<UserInfo>,
    me_id: Option<i32>,
    chat_messages: IArray<Msg>,
    chat_page: i32,
    chat_rows: i64,
    no_overwrite: bool,
}

#[component(Chat)]
pub fn chat() -> Html {
    let state = use_state(ChatState::default);
    let navigator = use_navigator().unwrap();
    let (tx, rx) = mpsc::unbounded::<bool>();
    let tx_ref = use_mut_ref(|| tx);

    {
        let state = state.clone();
        let navigator = navigator.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let token = match storage::get_access_token() {
                    Some(t) => t,
                    None => {
                        match auth::check_token().await {
                            Ok(_) => {}
                            Err(ErrActions::Relogin | ErrActions::Other) => {
                                navigator.replace(&Route::Login);
                                return;
                            }
                            _ => unreachable!(),
                        }

                        storage::get_access_token().unwrap()
                    }
                };

                let friends = match user::get_contacts().await {
                    // We know that token exists and is valid
                    Ok(v) => v.unwrap(),
                    Err(e) => {
                        console::error_1(&format!("An error occurred while getting contacts: {e}").into());
                        return;
                    }
                };

                let me_info = match user::get_user_info(None).await {
                    Ok(v) => v.unwrap(),
                    Err(e) => {
                        console::error_1(&format!("An error occurred while getting user info: {e}").into());
                        return;
                    }
                };

                let mut new_state = (*state).clone();
                new_state.chats = friends.into();
                new_state.me_id = Some(me_info.id);
                state.set(new_state);

                let ws = WebSocket::open(&format!("ws://ws.localhost/?token={}", token))
                    .expect("Failed to open websocket connection");

                let (mut write, read) = ws.split();

                let mut read = read.fuse();
                let mut rx = Box::pin(rx).fuse();

                loop {
                    select! {
                        msg = read.next().fuse() => {
                            let msg = if let Some(msg) = msg { msg } else { break; };

                            match msg {
                                Ok(Message::Bytes(buf)) => {
                                    let message = match WSMessage::from_bytes(&buf) {
                                        Some(message) => message,
                                        None => {
                                            console::error_1(&"Received malformed packet".into());
                                            continue;
                                        }
                                    };

                                    match message {
                                        WSMessage::ChangeStatus(new_status, user_id) => {
                                            let mut new_state = (*state).clone();
                                            let mut chats = new_state.chats.to_vec();
                                            for chat in chats.iter_mut() {
                                                if chat.id == user_id {
                                                    chat.status = new_status
                                                }
                                            }
                                            new_state.chats = chats.into();
                                            state.set(new_state);
                                        }
                                        WSMessage::SendMessage(timestamp, user_id, content) => {
                                            let mut exists = false;
                                            for c in state.chats.iter() {
                                                if c.id == user_id {
                                                    exists = true;
                                                }
                                            }

                                            if !exists {
                                                let mut new_state = (*state).clone();
                                                let mut new_chats = state.chats.to_vec();
                                                new_chats.push(user::get_user_info(Some(UserIdentifier::ID(user_id))).await.unwrap().unwrap());
                                                new_state.chats = new_chats.into();
                                                state.set(new_state);
                                            }

                                            if state.active_chat.is_some() && user_id == state.active_chat.unwrap() {
                                                let mut new_state = (*state).clone();
                                                let mut new_messages = new_state.chat_messages.to_vec();
                                                new_messages.push(Msg { content: content.clone().into(), timestamp, sender: state.active_chat.unwrap() });
                                                new_state.chat_messages = new_messages.into();
                                                state.set(new_state);
                                            }
                                        }
                                    }
                                }
                                Err(err) => {
                                    console::error_1(&format!("Websocket error: {:?}", err).into());
                                }
                                _ => {}
                            }
                        }

                        msg = rx.next() => {
                            if let Some(_) = msg {
                                break;
                            }
                        }
                    }
                }

                let _ = write.close().await;
            });
        });
    }

    let on_logout = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            storage::delete_tokens();
            let tx = tx_ref.borrow_mut();
            let _ = tx.send_now(true);
            navigator.replace(&Route::Login);
        })
    };

    let change_chat_window = {
        let state = state.clone();
        let navigator = navigator.clone();
        Callback::from(move |user_id| {
            let mut new_state = (*state).clone();
            new_state.chat_page = 0;
            new_state.chat_messages = [].into();
            new_state.active_chat = Some(user_id);
            for user in state.chats.iter() {
                if user.id == user_id {
                    new_state.recipient_name = Some(user.username.clone());
                }
            }
            state.set(new_state);

            if state.no_overwrite {
                let mut new_state = (*state).clone();
                new_state.no_overwrite = false;
                state.set(new_state);
                return;
            }

            {
                let navigator = navigator.clone();
                let state = state.clone();

                spawn_local(async move {
                    let (messages, chat_rows) = match user::get_messages(user_id, 0).await {
                        Ok(Ok(messages)) => (
                            messages
                                .messages
                                .iter()
                                .rev()
                                .map(|m| m.to_owned())
                                .collect::<Vec<Msg>>(),
                            messages.row_count,
                        ),
                        Ok(Err(ErrActions::Relogin | ErrActions::Other)) => {
                            navigator.replace(&Route::Login);
                            return;
                        }
                        Ok(Err(e)) => {
                            console::error_1(&format!("Error: {:?}", e).into());
                            return;
                        }
                        Err(e) => {
                            console::error_1(&format!("Error: {:?}", e).into());
                            return;
                        }
                    };

                    let mut new_state = (*state).clone();
                    new_state.chat_messages = messages.into();
                    new_state.chat_rows = chat_rows;
                    state.set(new_state);
                });
            }
        })
    };

    let on_msg_sent = {
        let state = state.clone();
        let navigator = navigator.clone();

        Callback::from(move |content: AttrValue| {
            let mut new_state = (*state).clone();
            let mut new_messages = new_state.chat_messages.to_vec();
            new_messages.push(Msg {
                content: content.clone(),
                timestamp: Utc::now(),
                sender: state.me_id.unwrap(),
            });
            new_state.chat_messages = new_messages.into();
            state.set(new_state);

            let state = state.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                match user::send_message(state.active_chat.unwrap(), content.to_string()).await {
                    Ok(Ok(())) => (),
                    Ok(Err(ErrActions::Relogin | ErrActions::Other)) => {
                        navigator.replace(&Route::Login);
                        return;
                    }
                    Err(e) => {
                        console::error_1(
                            &format!("Error occurred fetching user data: {:?}", e).into(),
                        );
                        return;
                    }
                    _ => unreachable!(),
                };
            });
        })
    };

    let on_top_reached = {
        let navigator = navigator.clone();
        let state = state.clone();

        Callback::from(move |_| {
            let navigator = navigator.clone();
            let state = state.clone();

            spawn_local(async move {
                if state.chat_rows as i32 <= (state.chat_page + 1) * 100 {
                    return;
                }

                let chat_page = state.chat_page + 1;

                let mut messages: Vec<Msg> =
                    match user::get_messages(state.active_chat.unwrap(), chat_page).await {
                        Ok(Ok(messages)) => messages.messages,
                        Ok(Err(ErrActions::Relogin | ErrActions::Other)) => {
                            navigator.replace(&Route::Login);
                            return;
                        }
                        Ok(Err(e)) => {
                            console::error_1(&format!("Error: {:?}", e).into());
                            return;
                        }
                        Err(e) => {
                            console::error_1(&format!("Error: {:?}", e).into());
                            return;
                        }
                    };

                let mut new_state = (*state).clone();
                let mut new_messages = state
                    .chat_messages
                    .iter()
                    .rev()
                    .map(|m| m.clone())
                    .collect::<Vec<_>>();
                new_messages.append(&mut messages);
                new_messages.reverse();
                new_state.chat_messages = new_messages.into();
                new_state.chat_page = chat_page;
                state.set(new_state);
            });
        })
    };

    let new_chat = {
        let state = state.clone();
        let navigator = navigator.clone();
        let on_msg_sent = on_msg_sent.clone();
        let change_chat_window = change_chat_window.clone();

        Callback::from(move |(receiver_name, content): (_, String)| {
            let state = state.clone();
            let navigator = navigator.clone();
            let on_msg_sent = on_msg_sent.clone();
            let change_chat_window = change_chat_window.clone();

            spawn_local(async move {
                let user_info =
                    match user::get_user_info(Some(UserIdentifier::Username(receiver_name))).await {
                        Ok(Ok(user_info)) => user_info,
                        Ok(Err(ErrActions::Relogin | ErrActions::Other)) => {
                            navigator.replace(&Route::Login);
                            return;
                        }
                        Ok(Err(e)) => {
                            console::error_1(&format!("Error: {:?}", e).into());
                            return;
                        }
                        Err(e) => {
                            console::error_1(&format!("Error: {:?}", e).into());
                            return;
                        }
                    };

                for c in state.chats.iter() {
                    if c.id == user_info.id {
                        change_chat_window.emit(user_info.id);
                        on_msg_sent.emit(content.into());
                        return;
                    }
                }

                let mut new_state = (*state).clone();
                let mut new_chats = state.chats.to_vec();
                new_chats.push(user_info.clone());
                new_state.chats = new_chats.into();
                new_state.no_overwrite = true;
                state.set(new_state);

                change_chat_window.emit(user_info.id);
                on_msg_sent.emit(content.into());
            });
        })
    };

    html! {
        <main class={ classes!("chat-main") }>
            <ChatSidebar entries={ state.chats.clone() } { on_logout } me_id={ state.me_id.clone() } user_clicked={ change_chat_window } { new_chat } selected={ state.active_chat } />
            if state.active_chat.is_some() {
                <ChatWindow { on_msg_sent } messages={ state.chat_messages.clone() } recipient_name={ state.recipient_name.as_ref().unwrap().clone() } { on_top_reached } me_id={ state.me_id.clone() } />
            }
        </main>
    }
}
