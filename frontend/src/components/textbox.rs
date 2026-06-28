use web_sys::{EventTarget, HtmlInputElement, wasm_bindgen::JsCast};
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct TextBoxProps {
    pub text: AttrValue,
    pub input_type: AttrValue,
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
    #[prop_or_default]
    pub classes: Classes,
}

#[component(TextBox)]
pub fn text_box(props: &TextBoxProps) -> Html {
    let is_focused = use_state(|| false);

    let _is_focused = is_focused.clone();
    let onfocus_input = Callback::from(move |_| {
        _is_focused.set(true);
    });

    let _is_focused = is_focused.clone();
    let onblur_input = Callback::from(move |e: FocusEvent| {
        let target: EventTarget = e.target().expect("Event should have a target");
        let input: &HtmlInputElement = target.dyn_ref().expect("Target should be <input>");

        if input.value().is_empty() {
            _is_focused.set(false);
        }
    });

    html! {
        <div class={ classes!("textbox", &props.classes) }>
            <input type={ &props.input_type } onfocus={ onfocus_input } onblur={ onblur_input } oninput = { &props.oninput } />
            <span class={ classes!("textbox-text", if *is_focused { "focused" } else { "unfocused" }) }>{ &props.text }</span>
        </div>
    }
}
