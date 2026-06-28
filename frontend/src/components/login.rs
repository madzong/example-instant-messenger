use crate::{
    Route,
    components::{Loading, PushButton, TextBox},
    services::{auth, storage},
    types::ErrActions,
};
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};
use yew_router::prelude::*;

#[derive(Clone, Default)]
struct LoginState {
    login_value: AttrValue,
    password_value: AttrValue,
    error_message: Option<AttrValue>,
    loading: bool,
}

#[component(Login)]
pub fn login() -> Html {
    let state = use_state(|| LoginState::default());
    let navigator = use_navigator().unwrap();

    if let Some(_) = storage::get_refresh_token() {
        navigator.replace(&Route::Chat);
    }

    let oninput_login = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();

            let mut new_state = (*state).clone();
            new_state.login_value = input.value().into();
            state.set(new_state);
        })
    };

    let oninput_password = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();

            let mut new_state = (*state).clone();
            new_state.password_value = input.value().into();
            state.set(new_state);
        })
    };

    let onclick_log_in = {
        let state = state.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.loading = true;
            state.set(new_state);

            {
                let state = state.clone();
                let navigator = navigator.clone();
                spawn_local(async move {
                    let login_res = auth::login(
                        state.login_value.to_string(),
                        state.password_value.to_string(),
                    )
                    .await;

                    let mut new_state = (*state).clone();

                    new_state.error_message = match login_res {
                        Err(e) => Some(format!("Error: {}", e).into()),
                        Ok(Err(ErrActions::PasswordInvalid)) => {
                            Some("Credentials invalid - try again.".into())
                        }
                        Ok(Err(ErrActions::Other)) => Some("An unknown error occurred.".into()),
                        Ok(Ok(_)) => {
                            navigator.replace(&Route::Chat);
                            return;
                        }
                        _ => unreachable!(),
                    };

                    new_state.loading = false;

                    state.set(new_state);
                });
            }
        })
    };

    html! {
        <main class={ classes!("login-page", "floating-host") }>
            <div class={ classes!("login-menu", "floating-window") }>
                <div class={ classes!("login-menu-left") }>
                    <h1>{ "Log in" }</h1>
                    <p>{ "Log into your account." }</p>
                    if let Some(msg) = &state.error_message {
                        <p class={ classes!("msg-error") }>{ msg }</p>
                    }
                </div>
                <div class={ classes!("login-menu-right") }>
                    <TextBox input_type={ "text" } text={ "Login" } classes={ classes!("full-width") } oninput={ oninput_login } />
                    <TextBox input_type={ "password" } text={ "Password" } classes={ classes!("full-width") } oninput={ oninput_password }  />
                    <div class={ classes!("login-menu-buttons") }>
                        <PushButton onclick={ onclick_log_in }>{ "Log in" }</PushButton>
                    </div>
                </div>
                if state.loading {
                    <div class={ classes!("floating-window-loader") }>
                        <Loading />
                    </div>
                }
            </div>
        </main>
    }
}
