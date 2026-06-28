use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct LinkButtonProps {
    pub children: Html,
    pub route: Route,
}

#[component(LinkButton)]
pub fn link_button(LinkButtonProps { children, route }: &LinkButtonProps) -> Html {
    html! {
        <Link<Route> to={ route.clone() } classes={ classes!("link-button") }>{ children }</Link<Route>>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct PushButtonProps {
    #[prop_or_default]
    pub children: Html,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

#[component(PushButton)]
pub fn push_button(props: &PushButtonProps) -> Html {
    html! {
        <button class={ classes!("push-button") } onclick={ &props.onclick } type={ "button" }>{ &props.children }</button>
    }
}
