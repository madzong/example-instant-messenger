use components::{Chat, Home, Login, Register};
use yew::prelude::*;
use yew_router::prelude::*;

pub mod components;
pub mod services;
pub mod types;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/register")]
    Register,
    #[at("/chat")]
    Chat,
}

impl Route {
    fn switch(self) -> Html {
        match self {
            Route::Home => html! { <Home /> },
            Route::Login => html! { <Login /> },
            Route::Register => html! { <Register /> },
            Route::Chat => html! { <Chat /> },
        }
    }
}

#[component(Main)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Route::switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<Main>::new().render();
}
