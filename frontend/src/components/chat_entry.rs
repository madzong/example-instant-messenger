use yew::prelude::*;

use crate::{components::StatusDot, types::UserStatus};

#[derive(Properties, PartialEq, Debug, Clone)]
pub struct ChatEntryProps {
    pub user_id: i32,
    pub user_name: AttrValue,
    pub user_status: UserStatus,
    #[prop_or_default]
    pub onclick: Callback<i32>,
    pub is_selected: bool,
}

#[component(ChatEntry)]
pub fn chat_entry(props: &ChatEntryProps) -> Html {
    let onclick = {
        let props = props.clone();
        Callback::from(move |_| props.onclick.emit(props.user_id))
    };

    html! {
        <div class={ classes!("chat-entry", if props.is_selected { "active" } else { "" }) } { onclick }>
            <p>{ &props.user_name }</p>
            <StatusDot status={ props.user_status } />
        </div>
    }
}
