extern crate tela_html;
use tela_html::{html, Props};

fn component(props: Props) -> String {
    println!("{:?}", props);
    html! {<div></div>}
}

fn main() {
    let error_code = 34;
    let attrs = [("hidden", "yes"), ("alt", "Wrapper")];

    let result = html! {
        <!-- "Some Comment" -->
        <div title="something" {..attrs}>
            <p>
                "Some text here"
            </p>
            <component>
                { error_code }
            </component>
        </div>
    };
    println!("{}", result);
}
