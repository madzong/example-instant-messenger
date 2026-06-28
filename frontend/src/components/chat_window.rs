use crate::components::{Msg, MsgSide, PushButton};
use implicit_clone::unsync::IArray;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone, Debug)]
pub struct ChatWindowProps {
    #[prop_or_default]
    pub on_msg_sent: Callback<AttrValue>,
    pub on_top_reached: Callback<()>,
    pub messages: IArray<Msg>,
    pub recipient_name: AttrValue,
}

#[derive(Clone, Debug, Default)]
struct ChatWindowState {
    msg_input: AttrValue,
    updating: bool,
    window_height: i32,
}

#[component(ChatWindow)]
pub fn chat_window(props: &ChatWindowProps) -> Html {
    let state = use_state(ChatWindowState::default);
    let chat_window_ref = use_node_ref();

    let oninput = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let target: HtmlInputElement = e.target_unchecked_into();
            let value = target.value();

            let mut new_state = (*state).clone();
            new_state.msg_input = value.into();
            state.set(new_state);
        })
    };

    let on_send = {
        let state = state.clone();
        let props = props.clone();

        Callback::from(move |_| {
            let msg_content = state.msg_input.clone();

            if msg_content.is_empty() {
                return;
            }

            let mut new_state = (*state).clone();
            new_state.msg_input = String::new().into();
            new_state.updating = false;
            state.set(new_state);

            props.on_msg_sent.emit(msg_content);
        })
    };

    let onkeypress = {
        let on_send = on_send.clone();

        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                on_send.emit(MouseEvent::new("click").unwrap());
            }
        })
    };

    {
        let chat_window_ref = chat_window_ref.clone();
        let messages = props.messages.clone();
        let state = state.clone();

        use_effect_with(messages, move |_| {
            if state.updating {
                let mut new_state = (*state).clone();
                new_state.updating = false;
                state.set(new_state);

                if let Some(element) = chat_window_ref.cast::<HtmlElement>() {
                    element.set_scroll_top(element.scroll_height() - state.window_height);
                }

                return;
            }

            if let Some(element) = chat_window_ref.cast::<HtmlElement>() {
                element.set_scroll_top(element.scroll_height());
            }
        })
    }

    let onscroll = {
        let props = props.clone();
        let state = state.clone();
        let chat_window_ref = chat_window_ref.clone();

        Callback::from(move |e: Event| {
            if state.updating {
                return;
            }

            let element: HtmlElement = e.target_unchecked_into();

            if element.scroll_top() < 10 {
                let mut new_state = (*state).clone();
                new_state.updating = true;
                new_state.window_height = chat_window_ref
                    .cast::<HtmlElement>()
                    .unwrap()
                    .scroll_height();
                state.set(new_state);

                props.on_top_reached.emit(());
            }
        })
    };

    html! {
        <div class={ classes!("chat-window") }>
            <div class={ classes!("chat-messages") } ref={ chat_window_ref } onscroll={ onscroll }>
                for msg in props.messages.iter() {
                    <p>
                        <span class={ classes!("bold-text") }>
                            { "[" }
                            { match msg.side {
                                MsgSide::ME => "Me",
                                MsgSide::OTHER => &props.recipient_name,
                            } }
                            { "]:" }
                        </span>
                        { " " }
                        { &msg.content }
                        <span class={ classes!("msg-timestamp") }>
                            { "(" }
                            {
                                msg.timestamp.format("%d/%m/%Y %H:%M:%S").to_string()
                            }
                            { ")" }
                        </span>
                    </p>
                }
            </div>
            <div class={ classes!("chat-box") }>
                <input type="text" { oninput } { onkeypress } value={ state.msg_input.clone() } />
                <PushButton onclick={ on_send }>{ "Send" }</PushButton>
            </div>
        </div>
    }
}
