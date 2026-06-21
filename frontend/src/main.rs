use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/register")]
    Register,
    #[at("/chat")]
    Chat,
}

#[component(Home)]
fn home() -> Html {
    html! {
        <div>
            <h1 class={ classes!("hello") }>{ "Hello, World!" }</h1>
            <Link<Route> to={Route::Chat}>{ "Go chat" }</Link<Route>>
        </div>
    }
}

#[component(Login)]
fn login() -> Html {
    html! {
        <h1>{ "Login page" }</h1>
    }
}

#[component(Register)]
fn register() -> Html {
    html! {
        <h1>{ "Register page" }</h1>
    }
}

#[component(Chat)]
fn chat() -> Html {
    html! {
        <h1>{ "Chat page" }</h1>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Login => html! { <Login /> },
        Route::Register => html! { <Register /> },
        Route::Chat => html! { <Chat /> },
    }
}

#[component(Main)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<Main>::new().render();
}
