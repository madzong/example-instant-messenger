use yew::prelude::*;

#[component(Loading)]
pub fn loading() -> Html {
    html! {
        <div class={ classes!("loading-spinner") }></div>
    }
}
