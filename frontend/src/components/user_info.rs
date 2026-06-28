use web_sys::console;
use yew::{platform::spawn_local, prelude::*};
use yew_router::prelude::*;

use crate::{
    Route,
    components::StatusDot,
    services::user,
    types::{ErrActions, UserStatus},
};

#[derive(Properties, PartialEq, Clone, Debug)]
pub struct UserInfoProps {
    pub user_id: Option<i32>,
    #[prop_or_default]
    pub on_logout: Callback<MouseEvent>,
}

#[derive(Clone, PartialEq, Debug, Default)]
struct UserInfoState {
    username: AttrValue,
    status: UserStatus,
    popup_visible: bool,
}

fn change_status(new_status: UserStatus, navigator: Navigator) {
    spawn_local(async move {
        match user::change_status(new_status).await {
            Ok(Ok(())) => {}
            Ok(Err(ErrActions::Relogin | ErrActions::Other)) => {
                navigator.replace(&Route::Login);
                return;
            }
            Ok(Err(e)) => console::error_1(&format!("Error: {:?}", e).into()),
            Err(e) => console::error_1(&format!("Error: {:?}", e).into()),
        }
    });
}

#[component(UserInfo)]
pub fn user_info(props: &UserInfoProps) -> Html {
    let state = use_state(UserInfoState::default);
    let navigator = use_navigator().unwrap();

    {
        let state = state.clone();
        let navigator = navigator.clone();

        use_effect_with(props.user_id, move |user_id| {
            if user_id.is_none() {
                return;
            }

            {
                let user_id = user_id.clone();
                spawn_local(async move {
                    let me_info =
                        user::get_user_info(user::UserIdentifier::ID(user_id.unwrap())).await;

                    let me_info = match me_info {
                        Ok(Ok(info)) => info,
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

                    let mut new_state = (*state).clone();
                    new_state.status = me_info.status;
                    new_state.username = me_info.name;
                    state.set(new_state);
                });
            }
        });
    }

    let toggle_popup = {
        let state = state.clone();

        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.popup_visible = !state.popup_visible;
            state.set(new_state);
        })
    };

    let status_change_online = {
        let state = state.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            change_status(UserStatus::ONLINE, navigator.clone());

            let mut new_state = (*state).clone();
            new_state.status = UserStatus::ONLINE;
            state.set(new_state);
        })
    };

    let status_change_offline = {
        let state = state.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            change_status(UserStatus::OFFLINE, navigator.clone());

            let mut new_state = (*state).clone();
            new_state.status = UserStatus::OFFLINE;
            state.set(new_state);
        })
    };

    let status_change_dnd = {
        let state = state.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            change_status(UserStatus::DND, navigator.clone());

            let mut new_state = (*state).clone();
            new_state.status = UserStatus::DND;
            state.set(new_state);
        })
    };

    html! {
        <div class={ classes!("user-info") }>
            <div class={ classes!("popup-container", if state.popup_visible { "shown" } else { "" }) }>
                <div class={ classes!("popup") }>
                    <div class={ classes!("popup-item") } onclick={ status_change_online }>
                        { "Set status to" }
                        <StatusDot status={ UserStatus::ONLINE } />
                        { "Online" }
                    </div>
                    <div class={ classes!("popup-item") } onclick={ status_change_offline }>
                        { "Set status to" }
                        <StatusDot status={ UserStatus::OFFLINE } />
                        { "Invisible" }
                    </div>
                    <div class={ classes!("popup-item") } onclick={ status_change_dnd }>
                        { "Set status to" }
                        <StatusDot status={ UserStatus::DND } />
                        { "Do not Disturb" }
                    </div>
                    <hr />
                    <div class={ classes!("popup-item", "item-red") } onclick={ &props.on_logout }>{ "Log out" }</div>
                </div>
            </div>

            <div class={ classes!("info-me") } onclick={ toggle_popup } >
                <p>{ &state.username }{ " " }<span class={ classes!("bold-text") }>{ "(Me)" }</span></p>
                <StatusDot status={ state.status } />
            </div>
        </div>
    }
}
