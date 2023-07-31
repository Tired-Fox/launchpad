use std::ops::Deref;

use launchpad_macros::{client, main};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, HtmlHeadElement};

#[main]
fn main() -> Result<(), JsValue> {
    // Get the current document
    let document = document();
    let body = body();
    let head = head();

    let title = document.create_element("title")?;
    title.set_inner_html("Rust Wasm Example");
    head.append_child(&title)?;

    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust!");

    let val2: Element = clone_element(&val);
    val2.set_inner_html(get_name());

    body.append_child(&val)?;
    body.append_child(&val2)?;

    Ok(())
}

client! {
    pub fn get_name() -> &'static str {
        "Zachary"
    }

    pub fn window() -> web_sys::Window {
        web_sys::window().expect("no global 'window' exists")
    }

    pub fn document() -> web_sys::Document {
        window().document().expect("should have a document on window")
    }

    pub fn body() -> web_sys::HtmlElement {
        document().body().expect("document should have a body")
    }

    pub fn head() -> web_sys::HtmlHeadElement {
        document().head().expect("document should have a body")
    }

    pub fn clone_element(element: &Element) -> Element {
        element.deref().clone_node().unwrap().dyn_into().unwrap()
    }
}
