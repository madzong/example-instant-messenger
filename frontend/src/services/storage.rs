use chrono::{DateTime, Utc};
use wasm_cookies::{CookieOptions, cookies};
use web_sys::{HtmlDocument, wasm_bindgen::JsCast};

pub fn store_access_token(token: &str, expires: DateTime<Utc>) {
    store_cookie("access", token, expires);
}

pub fn store_refresh_token(token: &str, expires: DateTime<Utc>) {
    store_cookie("refresh", token, expires);
}

pub fn get_access_token() -> Option<String> {
    get_cookie("access")
}

pub fn get_refresh_token() -> Option<String> {
    get_cookie("refresh")
}

pub fn delete_tokens() {
    store_cookie("access", "", Utc::now());
    store_cookie("refresh", "", Utc::now());
}

fn format_date(date: DateTime<Utc>) -> String {
    date.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn store_cookie(name: &str, value: &str, expires: DateTime<Utc>) {
    let formatted_date = format_date(expires);

    let options = CookieOptions {
        expires: Some(formatted_date.into()),
        ..Default::default()
    };

    let cookie_str = cookies::set(name, value, &options);

    let window = web_sys::window().expect("There should be a window");
    let document = window.document().expect("There should be a document");
    let html_document: HtmlDocument = document
        .dyn_into()
        .expect("Document should also be HtmlDocument");

    html_document
        .set_cookie(&cookie_str)
        .expect("The cookie string is auto-formatted");
}

fn get_cookie_string() -> String {
    let window = web_sys::window().expect("There should be a window");
    let document = window.document().expect("There should be a document");
    let html_document: HtmlDocument = document
        .dyn_into()
        .expect("Document should also be HtmlDocument");

    html_document.cookie().unwrap_or_default()
}

fn get_cookie(name: &str) -> Option<String> {
    let raw_cookies = get_cookie_string();
    cookies::get(&raw_cookies, name).map(|c| c.expect("Cookie is invalid"))
}
