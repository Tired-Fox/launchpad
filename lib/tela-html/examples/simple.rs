use serde::{Deserialize, Serialize};
use tela_html::{html, Element, Prop, Props};

#[derive(Prop, Deserialize, Serialize)]
struct Complex<'a> {
    alt: &'a str,
    title: &'a str,
}

fn custom(props: Props) -> Element {
    println!("{:?}", props);
    let complex: Complex = props.fetch("data-complex").unwrap();
    html! {
        <p alt={complex.alt} title={complex.title}>{props.children()}</p>
    }
}

fn main() {
    let complex = std::collections::HashMap::from([("alt", "hello"), ("title", "world")]);
    let result = html! {
        <!DOCTYPE html>
        <!-- "Some comment" -->
        <div>"Hello, world!"</div>
        <custom data-complex={complex}>
            "children go here"
            " and here"
        </custom>
    };

    println!("{}", result);
}
