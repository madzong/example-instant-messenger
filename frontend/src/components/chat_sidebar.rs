use crate::{
    components::{ChatEntry, PushButton, TextBox, UserInfo},
    types,
};
use implicit_clone::unsync::IArray;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone, Debug)]
pub struct ChatSidebarProps {
    pub entries: IArray<types::UserInfo>,
    #[prop_or_default]
    pub user_clicked: Callback<i32>,
    #[prop_or_default]
    pub on_logout: Callback<MouseEvent>,
    pub me_id: Option<i32>,
    pub selected: Option<i32>,
    pub new_chat: Callback<(String, String)>,
}

#[derive(PartialEq, Clone, Debug, Default)]
struct ChatSidebarState {
    active: i32,
    new_chat_popout_active: bool,
    new_chat_popout_x: f64,
    new_chat_popout_y: f64,
    new_chat_username: AttrValue,
    new_chat_message: AttrValue,
}

#[component(ChatSidebar)]
pub fn chat_sidebar(props: &ChatSidebarProps) -> Html {
    let state = use_state(ChatSidebarState::default);

    {
        let state = state.clone();

        use_effect_with(props.selected, move |sel| {
            if sel.is_some() {
                let mut new_state = (*state).clone();
                new_state.active = sel.unwrap();
                state.set(new_state);
            }
        });
    }

    let user_clicked = {
        let state = state.clone();
        let props = props.clone();

        Callback::from(move |user_id| {
            let mut new_state = (*state).clone();
            new_state.active = user_id;
            state.set(new_state);

            props.user_clicked.emit(user_id);
        })
    };

    let new_chat = {
        let state = state.clone();

        Callback::from(move |e: MouseEvent| {
            let target: HtmlElement = e.target_unchecked_into();

            let rect = target.get_bounding_client_rect();
            let right = rect.right();
            let top = rect.top();

            let mut new_state = (*state).clone();
            new_state.new_chat_popout_x = right + 10.0;
            new_state.new_chat_popout_y = top;
            new_state.new_chat_popout_active = true;
            state.set(new_state);
        })
    };

    let new_chat_send = {
        let state = state.clone();
        let new_chat = props.new_chat.clone();

        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.new_chat_popout_active = false;
            state.set(new_state);

            new_chat.emit((
                state.new_chat_username.clone().to_string(),
                state.new_chat_message.clone().to_string(),
            ));
        })
    };

    let oninput_username = {
        let state = state.clone();

        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();

            let mut new_state = (*state).clone();
            new_state.new_chat_username = target.value().into();
            state.set(new_state);
        })
    };

    let oninput_message = {
        let state = state.clone();

        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();

            let mut new_state = (*state).clone();
            new_state.new_chat_message = target.value().into();
            state.set(new_state);
        })
    };

    let close_popout = {
        let state = state.clone();

        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.new_chat_popout_active = false;
            state.set(new_state);
        })
    };

    html! {
        <div class={ classes!("chat-sidebar") }>
            <div class={ classes!("new-chat-popout", if state.new_chat_popout_active { "shown" } else { "" }) } style={ format!("top:{}px;left:{}px", state.new_chat_popout_y, state.new_chat_popout_x) }>
                <span class={ classes!("close-button") } onclick={ close_popout }>{ "x" }</span>
                <TextBox text={ "Username" } input_type={ "text" } oninput={ oninput_username } />
                <TextBox text={ "Message" } input_type={ "text" } oninput={ oninput_message } />
                <PushButton onclick={ new_chat_send }>{ "Send" }</PushButton>
            </div>

            <div class={ classes!("sidebar-entries") }>
                <PushButton onclick={ new_chat }>{ "New chat" }</PushButton>
                {
                    props.entries
                        .iter()
                        .map(|u| html! { <ChatEntry user_id={ u.id } user_name={ &u.username } user_status={ u.status } onclick={ &user_clicked } is_selected={ state.active == u.id } /> })
                        .collect::<Html>()
                }
            </div>

            <UserInfo user_id={ props.me_id } on_logout={ &props.on_logout } />
        </div>
    }
}
