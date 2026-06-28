use crate::Route;
use crate::components::{LinkButton, LoremShort};
use yew::prelude::*;

#[component(Home)]
pub fn home() -> Html {
    html! {
        <main class={ classes!("homepage", "floating-host") }>
            <div class={ classes!("homepage-menu", "floating-window") }>
                <h1>{ "Example Instant Messenger" }</h1>
                <p><LoremShort /></p>
                <div class={ classes!("login-register") }>
                    <LinkButton route={ Route::Login }>{ "Log in" }</LinkButton>
                    <LinkButton route={ Route::Register }>{ "Register" }</LinkButton>
                </div>
            </div>
        </main>
    }
}
