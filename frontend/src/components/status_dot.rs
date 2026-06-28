use yew::prelude::*;

use crate::types::UserStatus;

#[derive(Properties, PartialEq, Clone, Copy, Debug)]
pub struct StatusDotProps {
    pub status: UserStatus,
}

#[component(StatusDot)]
pub fn status_dot(props: &StatusDotProps) -> Html {
    html! {
        <div class={ classes!(
            "status-dot",
            match props.status {
                UserStatus::OFFLINE => "offline",
                UserStatus::ONLINE => "online",
                UserStatus::DND => "dnd",
            }
        ) }></div>
    }
}
